// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::Instant;

use gch_core::db::Settings;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

use crate::jobs::{JobStatus, JobStore};
use crate::provision::destroy_job;

pub fn spawn_reaper(jobs: Arc<JobStore>, settings: Settings) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            if let Err(e) = reap_once(&jobs, &settings).await {
                warn!(error = %e, "reaper cycle error");
            }
        }
    });
}

async fn reap_once(jobs: &JobStore, settings: &Settings) -> Result<(), crate::provision::ProvisionError> {
    let now = Instant::now();
    let timeout = JobStore::job_timeout(settings.job_timeout_secs);
    let stale = JobStore::heartbeat_stale(settings.heartbeat_stale_secs);
    let provisioning_timeout =
        JobStore::provisioning_timeout(settings.provisioning_timeout_secs);

    for job in jobs.list_for_reaper().await {
        if job.status == JobStatus::Destroying {
            continue;
        }

        let job_age = chrono::Utc::now()
            .signed_duration_since(job.created_at)
            .to_std()
            .unwrap_or(timeout);

        let should_destroy = match job.status {
            JobStatus::Completed | JobStatus::Failed => true,
            JobStatus::Provisioning => {
                if job_age >= provisioning_timeout {
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
            JobStatus::Destroying => false,
        };

        if should_destroy {
            destroy_job(jobs, job.job_id).await?;
        }
    }

    Ok(())
}
