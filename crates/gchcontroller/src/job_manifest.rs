// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::jobs::{JobRecord, JobStatus};

pub const MANIFEST_FILENAME: &str = "job.manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobManifest {
    job_id: Uuid,
    token_hash: String,
    project_gitlab_path: String,
    tag: String,
    iid: u64,
    object_kind: String,
    prompt: String,
    model: String,
    hetzner_snapshot_id: String,
    workspace_path: String,
    git_ref: Option<String>,
    status: String,
    phase: Option<String>,
    status_message: Option<String>,
    runtime_token: Option<String>,
    retry_at: Option<DateTime<Utc>>,
    queue_attempts: u32,
    created_at: DateTime<Utc>,
    provisioning_started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    server_id: Option<i64>,
}

impl From<&JobRecord> for JobManifest {
    fn from(job: &JobRecord) -> Self {
        Self {
            job_id: job.job_id,
            token_hash: job.token_hash.clone(),
            project_gitlab_path: job.project_gitlab_path.clone(),
            tag: job.tag.clone(),
            iid: job.iid,
            object_kind: job.object_kind.clone(),
            prompt: job.prompt.clone(),
            model: job.model.clone(),
            hetzner_snapshot_id: job.hetzner_snapshot_id.clone(),
            workspace_path: job.workspace_path.clone(),
            git_ref: job.git_ref.clone(),
            status: job.status.as_str().to_string(),
            phase: job.phase.clone(),
            status_message: job.status_message.clone(),
            runtime_token: job.runtime_token.clone(),
            retry_at: job.retry_at,
            queue_attempts: job.queue_attempts,
            created_at: job.created_at,
            provisioning_started_at: job.provisioning_started_at,
            completed_at: job.completed_at,
            server_id: job.server_id,
        }
    }
}

impl TryFrom<JobManifest> for JobRecord {
    type Error = String;

    fn try_from(manifest: JobManifest) -> Result<Self, Self::Error> {
        let status = match manifest.status.as_str() {
            "queued" => JobStatus::Queued,
            "provisioning" => JobStatus::Provisioning,
            "running" => JobStatus::Running,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "destroying" => JobStatus::Destroying,
            other => return Err(format!("unknown job status in manifest: {other}")),
        };

        Ok(JobRecord {
            job_id: manifest.job_id,
            token_hash: manifest.token_hash,
            project_gitlab_path: manifest.project_gitlab_path,
            tag: manifest.tag,
            iid: manifest.iid,
            object_kind: manifest.object_kind,
            prompt: manifest.prompt,
            model: manifest.model,
            hetzner_snapshot_id: manifest.hetzner_snapshot_id,
            workspace_path: manifest.workspace_path,
            git_ref: manifest.git_ref,
            status,
            phase: manifest.phase,
            status_message: manifest.status_message,
            runtime_token: manifest.runtime_token,
            retry_at: manifest.retry_at,
            queue_attempts: manifest.queue_attempts,
            created_at: manifest.created_at,
            provisioning_started_at: manifest.provisioning_started_at,
            last_heartbeat: None,
            completed_at: manifest.completed_at,
            terraform_dir: PathBuf::new(), // filled by caller
            server_id: manifest.server_id,
        })
    }
}

pub fn write_manifest(job: &JobRecord) {
    let path = job.terraform_dir.join(MANIFEST_FILENAME);
    let manifest = JobManifest::from(job);
    let json = match serde_json::to_string_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            warn!(job_id = %job.job_id, error = %e, "failed to serialize job manifest");
            return;
        }
    };
    if let Err(e) = fs::write(&path, json) {
        warn!(job_id = %job.job_id, path = %path.display(), error = %e, "failed to write job manifest");
    }
}

pub fn remove_manifest(terraform_dir: &Path) {
    let path = terraform_dir.join(MANIFEST_FILENAME);
    if path.exists() {
        if let Err(e) = fs::remove_file(&path) {
            warn!(path = %path.display(), error = %e, "failed to remove job manifest");
        }
    }
}

/// Reload queued jobs from disk after a controller restart.
pub fn recover_queued_jobs(jobs_dir: &Path) -> Vec<JobRecord> {
    let Ok(entries) = fs::read_dir(jobs_dir) else {
        return Vec::new();
    };

    let mut recovered = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let terraform_dir = entry.path();
        if !terraform_dir.is_dir() {
            continue;
        }
        let manifest_path = terraform_dir.join(MANIFEST_FILENAME);
        if !manifest_path.exists() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        let manifest: JobManifest = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(e) => {
                warn!(path = %manifest_path.display(), error = %e, "invalid job manifest");
                continue;
            }
        };
        if manifest.status != "queued" {
            continue;
        }
        if manifest.runtime_token.as_deref().is_none_or(str::is_empty) {
            warn!(job_id = %manifest.job_id, "skipping queued manifest without runtime token");
            continue;
        }
        let mut job: JobRecord = match manifest.try_into() {
            Ok(j) => j,
            Err(e) => {
                warn!(path = %manifest_path.display(), error = %e, "unsupported job manifest");
                continue;
            }
        };
        job.terraform_dir = terraform_dir;
        info!(
            job_id = %job.job_id,
            tag = %job.tag,
            iid = job.iid,
            "recovered queued job from disk"
        );
        recovered.push(job);
    }

    recovered.sort_by_key(|j| j.created_at);
    recovered
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_job(terraform_dir: PathBuf) -> JobRecord {
        JobRecord {
            job_id: Uuid::new_v4(),
            token_hash: "hash".into(),
            project_gitlab_path: "group/project".into(),
            tag: "verify".into(),
            iid: 311,
            object_kind: "issue".into(),
            prompt: "verify".into(),
            model: "composer-2.5".into(),
            hetzner_snapshot_id: "snap".into(),
            workspace_path: "/home/agent/workspace".into(),
            git_ref: None,
            status: JobStatus::Queued,
            phase: Some("queued".into()),
            status_message: Some("waiting".into()),
            runtime_token: Some("runtime-token".into()),
            retry_at: Some(Utc::now()),
            queue_attempts: 1,
            created_at: Utc::now(),
            provisioning_started_at: None,
            last_heartbeat: None,
            completed_at: None,
            terraform_dir,
            server_id: None,
        }
    }

    #[test]
    fn manifest_roundtrip_and_recovery() {
        let dir = std::env::temp_dir().join(format!("gch-manifest-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let job = sample_job(dir.clone());
        write_manifest(&job);

        let jobs = recover_queued_jobs(dir.parent().unwrap());
        let recovered = jobs
            .into_iter()
            .find(|j| j.job_id == job.job_id)
            .expect("recovered job");
        assert_eq!(recovered.tag, "verify");
        assert_eq!(recovered.iid, 311);
        assert_eq!(recovered.status, JobStatus::Queued);
        assert_eq!(recovered.runtime_token.as_deref(), Some("runtime-token"));

        remove_manifest(&dir);
        fs::remove_dir_all(dir).ok();
    }
}
