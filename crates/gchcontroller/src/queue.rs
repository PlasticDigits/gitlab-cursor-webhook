// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use chrono::{Duration, Utc};
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{info, warn};

use crate::config::ControllerConfig;
use crate::hetzner::{count_servers_best_effort, near_server_limit};
use crate::jobs::JobStore;
use crate::provision::{self, ProvisionError};

pub fn spawn_queue_worker(config: Arc<ControllerConfig>, jobs: Arc<JobStore>) {
    tokio::spawn(async move {
        let mut ticker = interval(TokioDuration::from_secs(config.provision_queue_poll_secs));
        loop {
            ticker.tick().await;
            if let Err(e) = process_ready_jobs(&config, &jobs).await {
                warn!(error = %e, "provision queue cycle error");
            }
        }
    });
}

/// Promote queued jobs in FIFO order until capacity is full or the queue is empty.
/// Called when a slot frees up (e.g. after destroy) so work does not wait for poll timers.
pub async fn promote_queued_jobs(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
) -> Result<(), ProvisionError> {
    promote_jobs(config, jobs, PromoteMode::Fifo).await
}

async fn process_ready_jobs(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
) -> Result<(), ProvisionError> {
    promote_jobs(config, jobs, PromoteMode::ReadyOnly).await
}

#[derive(Clone, Copy)]
enum PromoteMode {
    /// Oldest queued job whose `retry_at` has passed.
    ReadyOnly,
    /// Oldest queued job regardless of `retry_at` (wake-on-destroy).
    Fifo,
}

async fn promote_jobs(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
    mode: PromoteMode,
) -> Result<(), ProvisionError> {
    let now = Utc::now();

    loop {
        if should_defer_provisioning(config, jobs).await {
            break;
        }

        let job = match mode {
            PromoteMode::Fifo => jobs.oldest_queued().await,
            PromoteMode::ReadyOnly => jobs.list_ready_for_retry(now).await.into_iter().next(),
        };

        let Some(job) = job else {
            break;
        };

        let Some(runtime_token) = job.runtime_token.clone() else {
            warn!(job_id = %job.job_id, "queued job missing runtime token, dropping");
            jobs.remove(job.job_id).await;
            continue;
        };

        info!(
            job_id = %job.job_id,
            queue_attempts = job.queue_attempts,
            fifo = matches!(mode, PromoteMode::Fifo),
            "promoting queued provision"
        );

        provision::start_terraform_for_job(config, jobs, &job, &runtime_token).await?;
    }

    Ok(())
}

pub async fn should_defer_provisioning(config: &ControllerConfig, jobs: &JobStore) -> bool {
    let active = jobs.count_active().await;
    if active >= config.settings.max_concurrent_jobs as usize {
        return true;
    }

    if let Some(count) = count_servers_best_effort(&config.hcloud_token).await {
        return near_server_limit(
            count,
            config.hetzner_server_limit,
            config.hetzner_server_queue_threshold,
        );
    }

    false
}

pub fn next_retry_at(config: &ControllerConfig) -> chrono::DateTime<Utc> {
    Utc::now() + Duration::seconds(config.provision_queue_poll_secs as i64)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use chrono::{Duration, Utc};
    use gch_core::db::Settings;
    use uuid::Uuid;

    use super::*;
    use crate::config::ControllerConfig;
    use crate::jobs::{JobRecord, JobStatus, JobStore};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn test_config(max_concurrent: u32) -> ControllerConfig {
        let root = repo_root();
        ControllerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            allowed_users: HashSet::new(),
            dedup_ttl_secs: 60,
            issue_dedup_ttl_secs: 60,
            db_path: PathBuf::from("/tmp/gch-queue-test.db"),
            hcloud_token: String::new(),
            firewall_id: String::new(),
            ssh_key_refs: vec![],
            ssh_key_ids: vec![],
            controller_url: String::new(),
            cursor_api_key: String::new(),
            gitlab_token: String::new(),
            jobs_dir: PathBuf::from("/tmp/gch-queue-jobs"),
            terraform_module_dir: root.join("terraform/modules/agent-vm"),
            cloud_init_template: root.join("templates/cloud_init.yaml.tpl"),
            provision_enabled: false,
            admin_token: None,
            hetzner_server_limit: 15,
            hetzner_server_queue_threshold: 14,
            provision_queue_poll_secs: 30,
            provision_queue_retry_secs: 1800,
            settings: Settings {
                controller_url: String::new(),
                firewall_id: String::new(),
                job_timeout_secs: 3600,
                heartbeat_stale_secs: 300,
                provisioning_timeout_secs: 900,
                max_concurrent_jobs: max_concurrent.into(),
            },
        }
    }

    fn queued_job(created_offset_secs: i64, retry_in_secs: i64) -> JobRecord {
        let now = Utc::now();
        JobRecord {
            job_id: Uuid::new_v4(),
            token_hash: String::new(),
            project_gitlab_path: "group/project".into(),
            tag: "verify".into(),
            iid: 1,
            object_kind: "issue".into(),
            prompt: "test".into(),
            model: "model".into(),
            hetzner_snapshot_id: "snap".into(),
            workspace_path: "/workspace".into(),
            git_ref: None,
            status: JobStatus::Queued,
            phase: Some("queued".into()),
            status_message: None,
            runtime_token: Some("runtime-token".into()),
            retry_at: Some(now + Duration::seconds(retry_in_secs)),
            queue_attempts: 1,
            created_at: now + Duration::seconds(created_offset_secs),
            provisioning_started_at: None,
            last_heartbeat: None,
            completed_at: None,
            terraform_dir: PathBuf::from("/tmp/job"),
            server_id: None,
        }
    }

    #[tokio::test]
    async fn oldest_queued_returns_fifo_order() {
        let jobs = JobStore::new();
        let first = queued_job(-20, 3600);
        let second = queued_job(-10, 3600);
        let third = queued_job(-5, 3600);
        jobs.insert(second.clone()).await;
        jobs.insert(third.clone()).await;
        jobs.insert(first.clone()).await;

        let oldest = jobs.oldest_queued().await.expect("queued job");
        assert_eq!(oldest.job_id, first.job_id);
    }

    #[tokio::test]
    async fn fifo_promote_skips_future_retry_at() {
        let config = test_config(10);
        let jobs = JobStore::new();
        let job = queued_job(0, 3600);
        jobs.insert(job.clone()).await;

        promote_queued_jobs(&config, &jobs).await.expect("promote");

        let updated = jobs.get(job.job_id).await.expect("job still exists");
        assert_eq!(updated.status, JobStatus::Provisioning);
    }

    #[tokio::test]
    async fn fifo_promote_stops_at_capacity() {
        let config = test_config(2);
        let jobs = JobStore::new();

        let running = JobRecord {
            status: JobStatus::Running,
            created_at: Utc::now() - Duration::seconds(30),
            ..queued_job(-30, 0)
        };
        jobs.insert(running).await;

        let first = queued_job(-20, 3600);
        let second = queued_job(-10, 3600);
        jobs.insert(first.clone()).await;
        jobs.insert(second.clone()).await;

        promote_queued_jobs(&config, &jobs).await.expect("promote");

        assert_eq!(
            jobs.get(first.job_id).await.expect("first").status,
            JobStatus::Provisioning
        );
        assert_eq!(
            jobs.get(second.job_id).await.expect("second").status,
            JobStatus::Queued
        );
    }
}
