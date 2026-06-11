// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use gch_core::tag::WebhookTag;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Provisioning,
    Running,
    Completed,
    Failed,
    Destroying,
}

impl JobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Provisioning => "provisioning",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Destroying => "destroying",
        }
    }

    pub const fn is_active(self) -> bool {
        matches!(self, Self::Provisioning | Self::Running)
    }
}

#[derive(Debug, Clone)]
pub struct JobRecord {
    pub job_id: Uuid,
    pub token_hash: String,
    pub project_gitlab_path: String,
    pub tag: WebhookTag,
    pub iid: u64,
    pub object_kind: String,
    pub prompt: String,
    pub model: String,
    pub workspace_path: String,
    pub git_ref: Option<String>,
    pub status: JobStatus,
    pub phase: Option<String>,
    pub status_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_heartbeat: Option<Instant>,
    pub completed_at: Option<DateTime<Utc>>,
    pub terraform_dir: PathBuf,
    pub server_id: Option<i64>,
}

pub struct JobStore {
    jobs: RwLock<HashMap<Uuid, JobRecord>>,
}

impl JobStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            jobs: RwLock::new(HashMap::new()),
        })
    }

    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn count_active(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|j| {
                matches!(
                    j.status,
                    JobStatus::Provisioning | JobStatus::Running
                )
            })
            .count()
    }

    pub async fn insert(&self, job: JobRecord) {
        self.jobs.write().await.insert(job.job_id, job);
    }

    pub async fn get(&self, job_id: Uuid) -> Option<JobRecord> {
        self.jobs.read().await.get(&job_id).cloned()
    }

    pub async fn verify_token(&self, job_id: Uuid, token: &str) -> Option<JobRecord> {
        let hash = Self::hash_token(token);
        let jobs = self.jobs.read().await;
        jobs.get(&job_id).and_then(|j| {
            if j.token_hash == hash {
                Some(j.clone())
            } else {
                None
            }
        })
    }

    pub async fn update_heartbeat(&self, job_id: Uuid) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.last_heartbeat = Some(Instant::now());
            if job.status == JobStatus::Provisioning {
                job.status = JobStatus::Running;
            }
            true
        } else {
            false
        }
    }

    pub async fn update_status(
        &self,
        job_id: Uuid,
        phase: Option<String>,
        message: Option<String>,
    ) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.phase = phase;
            job.status_message = message;
            true
        } else {
            false
        }
    }

    pub async fn mark_complete(
        &self,
        job_id: Uuid,
        success: bool,
        message: Option<String>,
    ) -> Option<JobRecord> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = if success {
                JobStatus::Completed
            } else {
                JobStatus::Failed
            };
            job.status_message = message;
            job.completed_at = Some(Utc::now());
            Some(job.clone())
        } else {
            None
        }
    }

    pub async fn mark_destroying(&self, job_id: Uuid) -> Option<JobRecord> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Destroying;
            Some(job.clone())
        } else {
            None
        }
    }

    pub async fn remove(&self, job_id: Uuid) -> Option<JobRecord> {
        self.jobs.write().await.remove(&job_id)
    }

    pub async fn list_for_reaper(&self) -> Vec<JobRecord> {
        self.jobs.read().await.values().cloned().collect()
    }

    pub async fn list(&self, active_only: bool) -> Vec<JobRecord> {
        let jobs = self.jobs.read().await;
        let mut list: Vec<JobRecord> = jobs
            .values()
            .filter(|j| !active_only || j.status.is_active())
            .cloned()
            .collect();
        list.sort_by_key(|j| std::cmp::Reverse(j.created_at));
        list
    }

    pub async fn list_all(&self) -> Vec<JobRecord> {
        self.list(false).await
    }

    pub fn job_timeout(settings_secs: u64) -> Duration {
        Duration::from_secs(settings_secs.max(60))
    }

    pub fn heartbeat_stale(settings_secs: u64) -> Duration {
        Duration::from_secs(settings_secs.max(30))
    }

    pub fn provisioning_timeout(settings_secs: u64) -> Duration {
        Duration::from_secs(settings_secs.max(60))
    }
}

impl Default for JobStore {
    fn default() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
        }
    }
}
