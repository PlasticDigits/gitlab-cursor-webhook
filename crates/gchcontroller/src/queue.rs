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
        let mut ticker = interval(TokioDuration::from_secs(30));
        loop {
            ticker.tick().await;
            if let Err(e) = process_ready_jobs(&config, &jobs).await {
                warn!(error = %e, "provision queue cycle error");
            }
        }
    });
}

async fn process_ready_jobs(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
) -> Result<(), ProvisionError> {
    let now = Utc::now();
    let ready = jobs.list_ready_for_retry(now).await;
    if ready.is_empty() {
        return Ok(());
    }

    for job in ready {
        if should_defer_provisioning(config, jobs).await {
            let retry_at = now + Duration::seconds(config.provision_queue_retry_secs as i64);
            jobs.mark_queued(job.job_id, "capacity_unavailable", retry_at)
                .await;
            continue;
        }

        let Some(runtime_token) = job.runtime_token.clone() else {
            warn!(job_id = %job.job_id, "queued job missing runtime token, dropping");
            jobs.remove(job.job_id).await;
            continue;
        };

        info!(
            job_id = %job.job_id,
            queue_attempts = job.queue_attempts,
            "retrying queued provision"
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
    Utc::now() + Duration::seconds(config.provision_queue_retry_secs as i64)
}
