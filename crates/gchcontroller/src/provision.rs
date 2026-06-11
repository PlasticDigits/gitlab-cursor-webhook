// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use chrono::Utc;
use gch_core::db::ResolvedTag;
use gch_core::tag::WebhookTag;
use tokio::process::Command;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::cloud_init::{render_cloud_init, CloudInitParams};
use crate::config::ControllerConfig;
use crate::jobs::{JobRecord, JobStatus, JobStore};

#[derive(Debug, thiserror::Error)]
pub enum ProvisionError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cloud-init render error: {0}")]
    CloudInit(#[from] crate::cloud_init::CloudInitError),
    #[error("terraform failed: {0}")]
    Terraform(String),
    #[error("max concurrent jobs reached")]
    MaxConcurrentJobs,
}

pub struct ProvisionRequest {
    pub project_gitlab_path: String,
    pub tag: WebhookTag,
    pub iid: u64,
    pub object_kind: String,
    pub prompt: String,
    pub resolved: ResolvedTag,
    pub git_ref: Option<String>,
}

pub async fn provision_job(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
    req: ProvisionRequest,
) -> Result<(Uuid, String), ProvisionError> {
    let active = jobs.count_active().await;
    if active >= config.settings.max_concurrent_jobs as usize {
        return Err(ProvisionError::MaxConcurrentJobs);
    }

    let job_id = Uuid::new_v4();
    let runtime_token = Uuid::new_v4().to_string();
    let token_hash = JobStore::hash_token(&runtime_token);

    let terraform_dir = config.jobs_dir.join(job_id.to_string());
    fs::create_dir_all(&terraform_dir)?;

    let cloud_init_script = format!(
        "{}/gch-cloud-init.sh",
        req.resolved.project.workspace_path.trim_end_matches('/')
    );

    let cloud_init = render_cloud_init(
        &config.cloud_init_template,
        &CloudInitParams {
            job_id,
            controller_url: &config.controller_url,
            runtime_token: &runtime_token,
            cursor_api_key: &config.cursor_api_key,
            gitlab_token: &config.gitlab_token,
            cloud_init_script: &cloud_init_script,
        },
    )?;

    write_terraform_workspace(
        &terraform_dir,
        &config.terraform_module_dir,
        &job_id,
        &req,
        &cloud_init,
        &config.firewall_id,
        &config.ssh_key_ids,
    )?;

    let job = JobRecord {
        job_id,
        token_hash,
        project_gitlab_path: req.project_gitlab_path.clone(),
        tag: req.tag,
        iid: req.iid,
        object_kind: req.object_kind,
        prompt: req.prompt,
        model: req.resolved.tag.model.clone(),
        workspace_path: req.resolved.project.workspace_path.clone(),
        git_ref: req.git_ref,
        status: JobStatus::Provisioning,
        phase: Some("terraform_apply".to_string()),
        status_message: None,
        created_at: Utc::now(),
        last_heartbeat: None,
        completed_at: None,
        terraform_dir: terraform_dir.clone(),
        server_id: None,
    };
    jobs.insert(job).await;

    if config.provision_enabled {
        // Return before terraform finishes — GitLab webhook delivery times out (~10s).
        let jobs = Arc::clone(jobs);
        let terraform_dir = terraform_dir.clone();
        tokio::spawn(async move {
            match run_terraform(&terraform_dir, "apply", true).await {
                Ok(()) => {
                    info!(%job_id, "terraform apply succeeded");
                }
                Err(e) => {
                    error!(%job_id, error = %e, "terraform apply failed");
                    jobs.remove(job_id).await;
                    let _ = destroy_workspace(&terraform_dir).await;
                }
            }
        });
    } else {
        warn!(%job_id, "GCH_PROVISION_ENABLED=false, skipping terraform apply");
    }

    Ok((job_id, runtime_token))
}

pub async fn destroy_job(
    jobs: &JobStore,
    job_id: Uuid,
) -> Result<(), ProvisionError> {
    let job = jobs.mark_destroying(job_id).await;
    let Some(job) = job else {
        return Ok(());
    };

    if let Err(e) = destroy_workspace(&job.terraform_dir).await {
        error!(%job_id, error = %e, "terraform destroy failed");
    }

    jobs.remove(job_id).await;
    info!(%job_id, "job destroyed and removed from memory");
    Ok(())
}

async fn destroy_workspace(terraform_dir: &Path) -> Result<(), ProvisionError> {
    if terraform_dir.join("main.tf").exists() {
        run_terraform(terraform_dir, "destroy", true).await?;
    }
    fs::remove_dir_all(terraform_dir).ok();
    Ok(())
}

async fn run_terraform(
    dir: &Path,
    action: &str,
    auto_approve: bool,
) -> Result<(), ProvisionError> {
    let hcloud_token = std::env::var("HCLOUD_TOKEN").unwrap_or_default();

    let init = Command::new("terraform")
        .args(["init", "-input=false", "-no-color"])
        .current_dir(dir)
        .env("HCLOUD_TOKEN", &hcloud_token)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !init.status.success() {
        let stderr = String::from_utf8_lossy(&init.stderr);
        return Err(ProvisionError::Terraform(format!("init: {stderr}")));
    }

    let mut args = vec![action, "-input=false", "-no-color"];
    if auto_approve {
        args.push("-auto-approve");
    }

    let out = Command::new("terraform")
        .args(&args)
        .current_dir(dir)
        .env("TF_IN_AUTOMATION", "1")
        .env("HCLOUD_TOKEN", &hcloud_token)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(ProvisionError::Terraform(format!("{action}: {stderr}")));
    }

    Ok(())
}

fn write_terraform_workspace(
    dir: &Path,
    module_dir: &Path,
    job_id: &Uuid,
    req: &ProvisionRequest,
    cloud_init: &str,
    firewall_id: &str,
    ssh_key_ids: &[String],
) -> Result<(), std::io::Error> {
    let module_abs = fs::canonicalize(module_dir).unwrap_or_else(|_| module_dir.to_path_buf());

    let ssh_keys = if ssh_key_ids.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            ssh_key_ids
                .iter()
                .map(|id| format!("\"{id}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    // Hetzner label values allow only [a-zA-Z0-9_.-] — no RFC3339 colons/plus signs.
    let created_at = Utc::now().timestamp().to_string();

    let main_tf = format!(
        r#"
terraform {{
  required_providers {{
    hcloud = {{
      source  = "hetznercloud/hcloud"
      version = "~> 1.49"
    }}
  }}
}}

provider "hcloud" {{}}

module "agent" {{
  source = "{module_abs}"

  job_id         = "{job_id}"
  server_type    = "{server_type}"
  snapshot_id    = "{snapshot_id}"
  location       = "{location}"
  firewall_id    = "{firewall_id}"
  ssh_key_ids    = {ssh_keys}
  cloud_init     = <<-EOT
{cloud_init}
EOT

  label_tag     = "{tag}"
  label_project = "{project}"
  label_iid     = "{iid}"
  label_created = "{created_at}"
}}
"#,
        module_abs = module_abs.display(),
        job_id = job_id,
        server_type = req.resolved.tag.server_type,
        snapshot_id = req.resolved.tag.hetzner_snapshot_id,
        location = req.resolved.tag.hetzner_location,
        firewall_id = firewall_id,
        ssh_keys = ssh_keys,
        cloud_init = cloud_init,
        tag = req.tag,
        project = req.project_gitlab_path,
        iid = req.iid,
        created_at = created_at,
    );

    fs::write(dir.join("main.tf"), main_tf)?;

    // Also write cloud-init as file for debugging
    fs::write(dir.join("cloud_init.yaml"), cloud_init)?;

    Ok(())
}

pub async fn destroy_job_by_dir(dir: &Path) -> Result<(), ProvisionError> {
    destroy_workspace(dir).await
}
