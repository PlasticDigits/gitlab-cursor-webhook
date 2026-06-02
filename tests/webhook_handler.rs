// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use gitlab_cursor_webhook::{config::Config, routes};
use http_body_util::BodyExt;
use reqwest::Client;
use serde_json::Value;
use tower::ServiceExt;

fn test_config(cursor_url: &str) -> Arc<Config> {
    Arc::new(Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        cursor_webhook_url: cursor_url.to_string(),
        cursor_token: "crsr_test_token".to_string(),
        gitlab_webhook_secret: None,
        allowed_users: ["plasticdigits", "brouie"]
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>(),
    })
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("json")
}

#[tokio::test]
async fn health_returns_ok() {
    let app = routes::router(routes::AppState {
        config: test_config("http://127.0.0.1:1/unused"),
        client: Client::new(),
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_json(response).await, serde_json::json!({ "status": "ok" }));
}

#[tokio::test]
async fn approval_webhook_is_skipped() {
    let app = routes::router(routes::AppState {
        config: test_config("http://127.0.0.1:1/unused"),
        client: Client::new(),
    });

    let body = include_str!("fixtures/mr_approval.json");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhook")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await,
        serde_json::json!({ "status": "skipped" })
    );
}

#[tokio::test]
async fn open_webhook_forwards_to_cursor() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock cursor");
    let addr = listener.local_addr().unwrap();
    let cursor_url = format!("http://{addr}/hook");

    tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new().route(
                "/hook",
                axum::routing::post(|| async { (StatusCode::ACCEPTED, "ok") }),
            ),
        )
        .await
        .expect("mock cursor server");
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let app = routes::router(routes::AppState {
        config: test_config(&cursor_url),
        client: Client::new(),
    });

    let body = include_str!("fixtures/mr_open.json");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhook")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}
