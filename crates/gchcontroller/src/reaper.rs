// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use gch_core::db::Settings;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

use crate::config::ControllerConfig;
use crate::jobs::{JobStatus, JobStore};
use crate::provision::destroy_job;

pub fn spawn_reaper(config: Arc<ControllerConfig>, jobs: Arc<JobStore>, settings: Settings) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            if let Err(e) = reap_once(&config, &jobs, &settings).await {
                warn!(error = %e, "reaper cycle error");
            }
        }
    });
}

async fn reap_once(
    config: &ControllerConfig,
    jobs: &Arc<JobStore>,
    settings: &Settings,
) -> Result<(), crate::provision::ProvisionError> {
    let now = std::time::Instant::now();
    let timeout = JobStore::job_timeout(settings.job_timeout_secs);
    let stale = JobStore::heartbeat_stale(settings.heartbeat_stale_secs);
    let provisioning_timeout =
        JobStore::provisioning_timeout(settings.provisioning_timeout_secs);

    for job in jobs.list_for_reaper().await {
        let job_age = chrono::Utc::now()
            .signed_duration_since(job.created_at)
            .to_std()
            .unwrap_or(timeout);

        let should_destroy = match job.status {
            JobStatus::Queued => false,
            JobStatus::Destroying => {
                info!(job_id = %job.job_id, reason = "retry_destroy", "reaping job");
                true
            }
            JobStatus::Completed | JobStatus::Failed => true,
            JobStatus::Provisioning => {
                let provision_age = job
                    .provisioning_started_at
                    .map(|started| {
                        chrono::Utc::now()
                            .signed_duration_since(started)
                            .to_std()
                            .unwrap_or(provisioning_timeout)
                    })
                    .unwrap_or(provisioning_timeout);
                if provision_age >= provisioning_timeout {
                    info!(job_id = %job.job_id, reason = "provisioning_timeout", "reaping job");
                    true
                } else {
                    false
                }
            }
            JobStatus::Running => {
                if job_age >= timeout {
                    info!(job_id = %job.job_id, reason = "max_lifetime", "reaping job");
                    true
                } else if let Some(hb) = job.last_heartbeat {
                    if now.duration_since(hb) >= stale {
                        info!(job_id = %job.job_id, reason = "stale_heartbeat", "reaping job");
                        true
                    } else {
                        false
                    }
                } else if job_age >= stale {
                    info!(job_id = %job.job_id, reason = "no_heartbeat", "reaping job");
                    true
                } else {
                    false
                }
            }
        };

        if should_destroy {
            destroy_job(config, Arc::clone(jobs), job.job_id).await?;
        }
    }

    Ok(())
}
