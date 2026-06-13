// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use gch_core::db::ResolvedTag;
use tokio::sync::{Mutex, watch};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tokio::process::Command;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::cloud_init::{render_cloud_init, CloudInitParams};
use crate::config::ControllerConfig;
use crate::jobs::{JobRecord, JobStatus, JobStore};
use crate::queue::{self, promote_queued_jobs};

#[derive(Debug, thiserror::Error)]
pub enum ProvisionError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cloud-init render error: {0}")]
    CloudInit(#[from] crate::cloud_init::CloudInitError),
    #[error("terraform failed: {0}")]
    Terraform(String),
    #[error("provision cancelled")]
    Cancelled,
}

#[derive(Clone)]
pub struct ProvisionRequest {
    pub project_gitlab_path: String,
    pub tag: String,
    pub iid: u64,
    pub object_kind: String,
    pub prompt: String,
    pub resolved: ResolvedTag,
    pub git_ref: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProvisionResult {
    pub job_id: Uuid,
    pub runtime_token: String,
    pub queued: bool,
    pub retry_at: Option<DateTime<Utc>>,
}

/// Hetzner server type + location attempts when placement fails (sold out, etc.).
const PLACEMENT_FALLBACKS: &[(&str, &str)] = &[
    ("cx33", "nbg1"),
    ("cx33", "fsn1"),
    ("cx33", "hel1"),
    ("cpx32", "nbg1"),
    ("cpx32", "fsn1"),
    ("cpx32", "hel1"),
    ("cpx41", "hil"),
];

struct TerraformWorkspace<'a> {
    module_dir: &'a Path,
    job_id: &'a Uuid,
    req: &'a ProvisionRequest,
    cloud_init: &'a str,
    firewall_id: &'a str,
    ssh_key_ids: &'a [String],
}

struct ProvisionTaskRegistry {
    cancel_txs: Mutex<HashMap<Uuid, watch::Sender<bool>>>,
    tasks: Mutex<HashMap<Uuid, JoinHandle<()>>>,
}

impl ProvisionTaskRegistry {
    fn new() -> Self {
        Self {
            cancel_txs: Mutex::new(HashMap::new()),
            tasks: Mutex::new(HashMap::new()),
        }
    }

    async fn register(&self, job_id: Uuid) -> watch::Receiver<bool> {
        let (tx, rx) = watch::channel(false);
        self.cancel_txs.lock().await.insert(job_id, tx);
        rx
    }

    async fn store_task(&self, job_id: Uuid, handle: JoinHandle<()>) {
        self.tasks.lock().await.insert(job_id, handle);
    }

    async fn cancel_and_wait(&self, job_id: Uuid) {
        if let Some(tx) = self.cancel_txs.lock().await.remove(&job_id) {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.tasks.lock().await.remove(&job_id) {
            let _ = tokio::time::timeout(Duration::from_secs(120), handle).await;
        }
        self.cancel_txs.lock().await.remove(&job_id);
        sleep(Duration::from_secs(2)).await;
    }

    async fn unregister(&self, job_id: Uuid) {
        self.cancel_txs.lock().await.remove(&job_id);
        self.tasks.lock().await.remove(&job_id);
    }
}

fn provision_tasks() -> &'static ProvisionTaskRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<ProvisionTaskRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ProvisionTaskRegistry::new)
}

pub async fn provision_job(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
    req: ProvisionRequest,
) -> Result<ProvisionResult, ProvisionError> {
    let job_id = Uuid::new_v4();
    let runtime_token = Uuid::new_v4().to_string();
    let token_hash = JobStore::hash_token(&runtime_token);

    let terraform_dir = config.jobs_dir.join(job_id.to_string());
    fs::create_dir_all(&terraform_dir)?;

    let defer = queue::should_defer_provisioning(config, jobs).await;
    let retry_at = defer.then(|| queue::next_retry_at(config));

    let job = JobRecord {
        job_id,
        token_hash,
        project_gitlab_path: req.project_gitlab_path.clone(),
        tag: req.tag.clone(),
        iid: req.iid,
        object_kind: req.object_kind.clone(),
        prompt: req.prompt.clone(),
        model: req.resolved.tag.model.clone(),
        hetzner_snapshot_id: req.resolved.tag.hetzner_snapshot_id.clone(),
        workspace_path: req.resolved.project.workspace_path.clone(),
        git_ref: req.git_ref.clone(),
        status: if defer {
            JobStatus::Queued
        } else {
            JobStatus::Provisioning
        },
        phase: Some(if defer {
            "queued".to_string()
        } else {
            "terraform_apply".to_string()
        }),
        status_message: defer.then(|| {
            "waiting for capacity (concurrent jobs or hetzner server limit)".to_string()
        }),
        runtime_token: Some(runtime_token.clone()),
        retry_at,
        queue_attempts: if defer { 1 } else { 0 },
        created_at: Utc::now(),
        provisioning_started_at: if defer { None } else { Some(Utc::now()) },
        last_heartbeat: None,
        completed_at: None,
        terraform_dir: terraform_dir.clone(),
        server_id: None,
    };
    jobs.insert(job).await;

    if defer {
        info!(
            %job_id,
            retry_at = ?retry_at,
            "job queued, will retry provisioning when capacity is available"
        );
        return Ok(ProvisionResult {
            job_id,
            runtime_token,
            queued: true,
            retry_at,
        });
    }

    start_terraform_for_request(config, jobs, job_id, &req, &runtime_token, &terraform_dir)
        .await?;

    Ok(ProvisionResult {
        job_id,
        runtime_token,
        queued: false,
        retry_at: None,
    })
}

pub async fn start_terraform_for_job(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
    job: &JobRecord,
    runtime_token: &str,
) -> Result<(), ProvisionError> {
    let req = ProvisionRequest {
        project_gitlab_path: job.project_gitlab_path.clone(),
        tag: job.tag.clone(),
        iid: job.iid,
        object_kind: job.object_kind.clone(),
        prompt: job.prompt.clone(),
        resolved: ResolvedTag {
            project: gch_core::db::ProjectRecord {
                id: 0,
                gitlab_path: job.project_gitlab_path.clone(),
                name: String::new(),
                workspace_path: job.workspace_path.clone(),
                enabled: true,
                signing_token: None,
            },
            tag: gch_core::db::TagRecord {
                id: 0,
                project_id: 0,
                name: job.tag.clone(),
                hetzner_snapshot_id: job.hetzner_snapshot_id.clone(),
                server_type: String::new(),
                hetzner_location: String::new(),
                model: job.model.clone(),
                enabled: true,
            },
            prompt_template: String::new(),
        },
        git_ref: job.git_ref.clone(),
    };

    jobs.promote_to_provisioning(job.job_id).await;
    start_terraform_for_request(
        config,
        jobs,
        job.job_id,
        &req,
        runtime_token,
        &job.terraform_dir,
    )
    .await
}

async fn start_terraform_for_request(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
    job_id: Uuid,
    req: &ProvisionRequest,
    runtime_token: &str,
    terraform_dir: &Path,
) -> Result<(), ProvisionError> {
    let cloud_init_script = format!(
        "{}/gch-cloud-init.sh",
        req.resolved.project.workspace_path.trim_end_matches('/')
    );

    let cloud_init = render_cloud_init(
        &config.cloud_init_template,
        &CloudInitParams {
            job_id,
            controller_url: &config.controller_url,
            runtime_token,
            cursor_api_key: &config.cursor_api_key,
            gitlab_token: &config.gitlab_token,
            cloud_init_script: &cloud_init_script,
        },
    )?;

    if config.provision_enabled {
        let jobs = Arc::clone(jobs);
        let terraform_dir = terraform_dir.to_path_buf();
        let module_dir = config.terraform_module_dir.clone();
        let firewall_id = config.firewall_id.clone();
        let ssh_key_ids = config.ssh_key_ids.clone();
        let provision_req = req.clone();
        let retry_secs = config.provision_queue_retry_secs;
        let cancel_rx = provision_tasks().register(job_id).await;
        let handle = tokio::spawn(async move {
            let workspace = TerraformWorkspace {
                module_dir: &module_dir,
                job_id: &job_id,
                req: &provision_req,
                cloud_init: &cloud_init,
                firewall_id: &firewall_id,
                ssh_key_ids: &ssh_key_ids,
            };
            let result =
                apply_with_placement_fallbacks(&terraform_dir, &workspace, job_id, cancel_rx)
                    .await;
            provision_tasks().unregister(job_id).await;
            match result {
                Ok((server_type, location)) => {
                    if job_was_cancelled(&jobs, job_id).await {
                        warn!(
                            %job_id,
                            %server_type,
                            %location,
                            "terraform apply completed after job was cancelled; destroying orphan resources"
                        );
                        if let Err(e) = destroy_workspace(&terraform_dir).await {
                            error!(%job_id, error = %e, "failed to destroy orphan resources after late apply");
                        }
                        return;
                    }
                    info!(
                        %job_id,
                        %server_type,
                        %location,
                        "terraform apply succeeded"
                    );
                }
                Err(ProvisionError::Cancelled) => {
                    info!(%job_id, "terraform apply cancelled");
                }
                Err(e) => {
                    error!(%job_id, error = %e, "terraform apply failed after all placement fallbacks");
                    let retry_at =
                        Utc::now() + chrono::Duration::seconds(retry_secs as i64);
                    if jobs
                        .mark_queued(
                            job_id,
                            &format!("placement_failed: {e}"),
                            retry_at,
                        )
                        .await
                        .is_some()
                    {
                        info!(
                            %job_id,
                            %retry_at,
                            "job re-queued after placement failure"
                        );
                    } else {
                        let _ = destroy_workspace(&terraform_dir).await;
                    }
                }
            }
        });
        provision_tasks().store_task(job_id, handle).await;
    } else {
        warn!(%job_id, "GCH_PROVISION_ENABLED=false, skipping terraform apply");
    }

    Ok(())
}

pub async fn destroy_job(
    config: &ControllerConfig,
    jobs: Arc<JobStore>,
    job_id: Uuid,
) -> Result<(), ProvisionError> {
    let job = match jobs.get(job_id).await {
        Some(j) if j.status == JobStatus::Destroying => j,
        _ => match jobs.mark_destroying(job_id).await {
            Some(j) => j,
            None => return Ok(()),
        },
    };

    provision_tasks().cancel_and_wait(job_id).await;
    wait_for_state_lock_release(&job.terraform_dir, Duration::from_secs(15)).await;

    if let Err(e) = destroy_workspace(&job.terraform_dir).await {
        error!(%job_id, error = %e, "terraform destroy failed; will retry");
        return Ok(());
    }

    jobs.remove(job_id).await;
    info!(%job_id, "job destroyed and removed from memory");

    if let Err(e) = promote_queued_jobs(config, &jobs).await {
        warn!(error = %e, "failed to promote queued job after destroy");
    }

    Ok(())
}

async fn job_was_cancelled(jobs: &JobStore, job_id: Uuid) -> bool {
    match jobs.get(job_id).await {
        None => true,
        Some(job) => job.status == JobStatus::Destroying,
    }
}

async fn apply_with_placement_fallbacks(
    terraform_dir: &Path,
    workspace: &TerraformWorkspace<'_>,
    job_id: Uuid,
    mut cancel_rx: watch::Receiver<bool>,
) -> Result<(&'static str, &'static str), ProvisionError> {
    let mut last_err: Option<ProvisionError> = None;
    let mut initialized = false;

    for (server_type, location) in PLACEMENT_FALLBACKS {
        if *cancel_rx.borrow() {
            return Err(ProvisionError::Cancelled);
        }

        write_terraform_workspace(terraform_dir, workspace, server_type, location)?;

        if !initialized {
            run_terraform(terraform_dir, "init", false, job_id, Some(&mut cancel_rx), None)
                .await?;
            initialized = true;
        }

        match run_terraform(
            terraform_dir,
            "apply",
            true,
            job_id,
            Some(&mut cancel_rx),
            None,
        )
        .await {
            Ok(()) => return Ok((server_type, location)),
            Err(ProvisionError::Cancelled) => return Err(ProvisionError::Cancelled),
            Err(e) => {
                warn!(
                    job_id = %workspace.job_id,
                    %server_type,
                    %location,
                    error = %e,
                    "placement failed, trying next fallback"
                );
                last_err = Some(e);
                let _ = run_terraform(
                    terraform_dir,
                    "destroy",
                    true,
                    job_id,
                    Some(&mut cancel_rx),
                    None,
                )
                .await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        ProvisionError::Terraform("no placement fallbacks configured".to_string())
    }))
}

async fn destroy_workspace(terraform_dir: &Path) -> Result<(), ProvisionError> {
    if !terraform_dir.join("main.tf").exists() {
        fs::remove_dir_all(terraform_dir).ok();
        return Ok(());
    }

    const MAX_ATTEMPTS: u32 = 5;
    for attempt in 0..MAX_ATTEMPTS {
        match run_terraform(terraform_dir, "destroy", true, Uuid::nil(), None, None).await {
            Ok(()) => {
                fs::remove_dir_all(terraform_dir).ok();
                return Ok(());
            }
            Err(e) if is_state_lock_error(&e) && attempt + 1 < MAX_ATTEMPTS => {
                warn!(
                    dir = %terraform_dir.display(),
                    attempt = attempt + 1,
                    error = %e,
                    "terraform destroy blocked by state lock; retrying"
                );
                let _ = try_force_unlock(terraform_dir).await;
                wait_for_state_lock_release(terraform_dir, Duration::from_secs(2)).await;
                let backoff = Duration::from_millis(500 * (attempt as u64 + 1));
                sleep(backoff).await;
            }
            Err(e) => return Err(e),
        }
    }

    Err(ProvisionError::Terraform(
        "destroy failed after state-lock retries".to_string(),
    ))
}

fn is_state_lock_error(err: &ProvisionError) -> bool {
    match err {
        ProvisionError::Terraform(msg) => {
            msg.contains("Error acquiring the state lock")
                || msg.contains("resource temporarily unavailable")
        }
        _ => false,
    }
}

fn read_lock_id(terraform_dir: &Path) -> Option<String> {
    let lock_path = terraform_dir.join(".terraform.tfstate.lock.info");
    let content = fs::read_to_string(lock_path).ok()?;
    let info: serde_json::Value = serde_json::from_str(&content).ok()?;
    info.get("ID")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

async fn try_force_unlock(terraform_dir: &Path) -> Result<(), ProvisionError> {
    let Some(lock_id) = read_lock_id(terraform_dir) else {
        return Ok(());
    };
    info!(
        dir = %terraform_dir.display(),
        lock_id = %lock_id,
        "forcing terraform state unlock after cancelled apply"
    );
    run_terraform(
        terraform_dir,
        "force-unlock",
        false,
        Uuid::nil(),
        None,
        Some(&["-force", &lock_id]),
    )
    .await
}

async fn wait_for_state_lock_release(terraform_dir: &Path, timeout: Duration) {
    let lock_path = terraform_dir.join(".terraform.tfstate.lock.info");
    if !lock_path.exists() {
        return;
    }
    let deadline = Instant::now() + timeout;
    while lock_path.exists() && Instant::now() < deadline {
        sleep(Duration::from_millis(250)).await;
    }
}

async fn run_terraform(
    dir: &Path,
    action: &str,
    auto_approve: bool,
    job_id: Uuid,
    mut cancel_rx: Option<&mut watch::Receiver<bool>>,
    extra_args: Option<&[&str]>,
) -> Result<(), ProvisionError> {
    let hcloud_token = std::env::var("HCLOUD_TOKEN").unwrap_or_default();

    if action == "apply" || action == "destroy" {
        let init = run_terraform_child(
            dir,
            &["init", "-input=false", "-no-color"],
            &hcloud_token,
            job_id,
            cancel_rx.as_deref_mut(),
        )
        .await?;
        if !init.success() {
            return Err(ProvisionError::Terraform(format!(
                "init: {}",
                init.stderr
            )));
        }
    }

    let mut args = vec![action, "-input=false", "-no-color"];
    if let Some(extra) = extra_args {
        args.extend_from_slice(extra);
    }
    if auto_approve {
        args.push("-auto-approve");
    }

    let out =
        run_terraform_child(dir, &args, &hcloud_token, job_id, cancel_rx.as_deref_mut()).await?;
    if !out.success() {
        return Err(ProvisionError::Terraform(format!("{action}: {}", out.stderr)));
    }

    Ok(())
}

struct TerraformOutput {
    success: bool,
    stderr: String,
}

impl TerraformOutput {
    fn success(&self) -> bool {
        self.success
    }
}

async fn run_terraform_child(
    dir: &Path,
    args: &[&str],
    hcloud_token: &str,
    _job_id: Uuid,
    cancel_rx: Option<&mut watch::Receiver<bool>>,
) -> Result<TerraformOutput, ProvisionError> {
    let child = Command::new("terraform")
        .args(args)
        .current_dir(dir)
        .env("TF_IN_AUTOMATION", "1")
        .env("HCLOUD_TOKEN", hcloud_token)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id();
    let mut child_task = tokio::spawn(async move { child.wait_with_output().await });

    let output = match cancel_rx {
        Some(cancel_rx) => tokio::select! {
            biased;
            _ = async {
                loop {
                    if cancel_rx.changed().await.is_err() {
                        // Sender gone without signalling cancel — let terraform finish.
                        std::future::pending::<()>().await;
                    }
                    if *cancel_rx.borrow() {
                        return;
                    }
                }
            } => {
                if let Some(pid) = pid {
                    let _ = Command::new("kill")
                        .arg(pid.to_string())
                        .status()
                        .await;
                }
                let _ = (&mut child_task).await;
                return Err(ProvisionError::Cancelled);
            }
            out = &mut child_task => {
                out.map_err(|e| ProvisionError::Terraform(format!("terraform task: {e}")))??
            }
        },
        None => (&mut child_task)
            .await
            .map_err(|e| ProvisionError::Terraform(format!("terraform task: {e}")))??,
    };

    Ok(TerraformOutput {
        success: output.status.success(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn write_terraform_workspace(
    dir: &Path,
    workspace: &TerraformWorkspace<'_>,
    server_type: &str,
    location: &str,
) -> Result<(), std::io::Error> {
    let module_abs =
        fs::canonicalize(workspace.module_dir).unwrap_or_else(|_| workspace.module_dir.to_path_buf());

    let ssh_keys = if workspace.ssh_key_ids.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            workspace
                .ssh_key_ids
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
        job_id = workspace.job_id,
        server_type = server_type,
        snapshot_id = workspace.req.resolved.tag.hetzner_snapshot_id,
        location = location,
        firewall_id = workspace.firewall_id,
        ssh_keys = ssh_keys,
        cloud_init = workspace.cloud_init,
        tag = workspace.req.tag,
        project = workspace.req.project_gitlab_path,
        iid = workspace.req.iid,
        created_at = created_at,
    );

    fs::write(dir.join("main.tf"), main_tf)?;

    // Also write cloud-init as file for debugging
    fs::write(dir.join("cloud_init.yaml"), workspace.cloud_init)?;

    Ok(())
}

pub async fn destroy_job_by_dir(dir: &Path) -> Result<(), ProvisionError> {
    destroy_workspace(dir).await
}
