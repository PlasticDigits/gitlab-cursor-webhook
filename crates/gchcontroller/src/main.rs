// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use gch_core::dedup::DedupCache;
use gchcontroller::{config::ControllerConfig, jobs::JobStore, queue, reaper, routes};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "gchcontroller=info".into()),
        )
        .init();

    let (mut config, db) = match ControllerConfig::from_env() {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("configuration error: {e}");
            std::process::exit(1);
        }
    };

    match gchcontroller::hetzner::resolve_ssh_key_ids(
        &config.hcloud_token,
        &config.ssh_key_refs,
    )
    .await
    {
        Ok(ids) => {
            if ids.is_empty() {
                tracing::warn!(
                    "GCH_SSH_KEY_IDS / GCH_SSH_KEY_NAME not set — agent VMs will be created \
                     without SSH keys (Hetzner emails a root password instead)"
                );
            } else {
                tracing::info!(ssh_key_ids = ?ids, "resolved Hetzner SSH keys for provisioning");
            }
            config.ssh_key_ids = ids;
        }
        Err(e) => {
            eprintln!("failed to resolve Hetzner SSH keys: {e}");
            std::process::exit(1);
        }
    }

    let settings = config.settings.clone();
    let config = Arc::new(config);
    let db = Arc::new(db);
    let jobs = JobStore::new();

    std::fs::create_dir_all(&config.jobs_dir).ok();

    reaper::spawn_reaper(config.clone(), jobs.clone(), settings);
    queue::spawn_queue_worker(config.clone(), jobs.clone());

    tracing::info!(listen_addr = %config.listen_addr, "starting gchcontroller");

    let state = routes::AppState {
        config: config.clone(),
        db,
        jobs,
        dedup: Arc::new(DedupCache::new(config.dedup_ttl_secs)),
        issue_dedup: Arc::new(DedupCache::new(config.issue_dedup_ttl_secs)),
    };

    let app = routes::router(state).layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(config.listen_addr)
        .await
        .expect("failed to bind listen address");

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}
