// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Job snapshot returned by the controller admin API and consumed by gchconfig.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobSummary {
    pub job_id: String,
    pub status: String,
    pub phase: Option<String>,
    pub status_message: Option<String>,
    pub project: String,
    pub tag: String,
    pub iid: u64,
    pub object_kind: String,
    pub model: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub last_heartbeat_secs_ago: Option<u64>,
    pub age_secs: u64,
    pub server_ipv4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobListResponse {
    pub jobs: Vec<JobSummary>,
}

/// Metadata parsed from a per-job Terraform workspace on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskJobWorkspace {
    pub job_id: String,
    pub tag: Option<String>,
    pub project: Option<String>,
    pub iid: Option<String>,
    pub server_ipv4: Option<String>,
}

pub fn server_ipv4_from_tfstate(terraform_dir: &Path) -> Option<String> {
    let state_path = terraform_dir.join("terraform.tfstate");
    let content = std::fs::read_to_string(state_path).ok()?;
    let state: serde_json::Value = serde_json::from_str(&content).ok()?;
    state
        .get("resources")?
        .as_array()?
        .iter()
        .find(|r| r.get("type").and_then(|t| t.as_str()) == Some("hcloud_server"))?
        .get("instances")?
        .as_array()?
        .first()?
        .get("attributes")?
        .get("ipv4_address")?
        .as_str()
        .map(str::to_string)
}

fn capture_quoted_value(main_tf: &str, key: &str) -> Option<String> {
    let key_pos = main_tf.find(key)?;
    let after_key = &main_tf[key_pos + key.len()..];
    let eq_pos = after_key.find('=')?;
    let after_eq = after_key[eq_pos + 1..].trim_start();
    let rest = after_eq.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn parse_disk_workspace(terraform_dir: &Path) -> Option<DiskJobWorkspace> {
    let job_id = terraform_dir.file_name()?.to_str()?.to_string();
    if uuid::Uuid::parse_str(&job_id).is_err() {
        return None;
    }

    let main_tf = std::fs::read_to_string(terraform_dir.join("main.tf")).ok()?;
    let server_ipv4 = server_ipv4_from_tfstate(terraform_dir);

    Some(DiskJobWorkspace {
        job_id,
        tag: capture_quoted_value(&main_tf, "label_tag"),
        project: capture_quoted_value(&main_tf, "label_project"),
        iid: capture_quoted_value(&main_tf, "label_iid"),
        server_ipv4,
    })
}

pub fn list_disk_workspaces(jobs_dir: &Path) -> Vec<DiskJobWorkspace> {
    let Ok(entries) = std::fs::read_dir(jobs_dir) else {
        return Vec::new();
    };

    let mut workspaces: Vec<DiskJobWorkspace> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| parse_disk_workspace(&e.path()))
        .collect();

    workspaces.sort_by(|a, b| b.job_id.cmp(&a.job_id));
    workspaces
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_server_ipv4_from_tfstate() {
        let dir = std::env::temp_dir().join(format!("gch-tfstate-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("terraform.tfstate"),
            r#"{
                "resources": [{
                    "type": "hcloud_server",
                    "instances": [{ "attributes": { "ipv4_address": "203.0.113.10" } }]
                }]
            }"#,
        )
        .unwrap();

        assert_eq!(
            server_ipv4_from_tfstate(&dir).as_deref(),
            Some("203.0.113.10")
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn parses_labels_from_main_tf() {
        let job_id = uuid::Uuid::new_v4();
        let dir = std::env::temp_dir().join(job_id.to_string());
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("main.tf"),
            format!(
                r#"
module "agent" {{
  label_tag     = "security"
  label_project = "plasticdigits-cl8y-dex-terraclassic"
  label_iid     = "7"
  job_id        = "{job_id}"
}}
"#
            ),
        )
        .unwrap();

        let parsed = parse_disk_workspace(&dir).expect("workspace");
        assert_eq!(parsed.job_id, job_id.to_string());
        assert_eq!(parsed.tag.as_deref(), Some("security"));
        assert_eq!(
            parsed.project.as_deref(),
            Some("plasticdigits-cl8y-dex-terraclassic")
        );
        assert_eq!(parsed.iid.as_deref(), Some("7"));
        fs::remove_dir_all(dir).ok();
    }
}
