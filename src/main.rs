// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use gitlab_cursor_webhook::{config::Config, routes};
use reqwest::Client;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gitlab_cursor_webhook=info".into()),
        )
        .init();

    let config = match Config::from_env() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("configuration error: {e}");
            std::process::exit(1);
        }
    };

    tracing::info!(listen_addr = %config.listen_addr, "starting gitlab-cursor-webhook");

    let state = routes::AppState {
        config: config.clone(),
        client: Client::new(),
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
