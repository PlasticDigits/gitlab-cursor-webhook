// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use gitlab_cursor_webhook::{config::Config, config::ProjectCursorConfig, routes};
use http_body_util::BodyExt;
use reqwest::Client;
use serde_json::Value;
use tower::ServiceExt;

fn test_config(project_webhooks: HashMap<String, ProjectCursorConfig>) -> Arc<Config> {
    Arc::new(Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        gitlab_webhook_secret: None,
        allowed_users: ["plasticdigits", "brouie"]
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>(),
        project_webhooks,
        dedup_ttl_secs: 86_400,
    })
}

fn test_state(cursor_url: &str) -> routes::AppState {
    let project_webhooks = [(
        "group/example-project".to_string(),
        ProjectCursorConfig {
            webhook_url: cursor_url.to_string(),
            token: "crsr_test_token".to_string(),
        },
    )]
    .into_iter()
    .collect();
    let config = test_config(project_webhooks);
    routes::AppState {
        config: config.clone(),
        client: Client::new(),
        dedup: Arc::new(gitlab_cursor_webhook::dedup::DedupCache::new(
            config.dedup_ttl_secs,
        )),
    }
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
    let app = routes::router(test_state("http://127.0.0.1:1/unused"));

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
    let app = routes::router(test_state("http://127.0.0.1:1/unused"));

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

    let app = routes::router(test_state(&cursor_url));

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

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await,
        serde_json::json!({ "status": "forwarded", "cursor_status": 202 })
    );
}

#[tokio::test]
async fn cursor_client_error_still_returns_ok_to_gitlab() {
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
                axum::routing::post(|| async { StatusCode::BAD_REQUEST }),
            ),
        )
        .await
        .expect("mock cursor server");
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let app = routes::router(test_state(&cursor_url));
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

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await,
        serde_json::json!({ "status": "forward_failed", "cursor_status": 400 })
    );
}

#[tokio::test]
async fn issue_hook_is_skipped_with_ok() {
    let app = routes::router(test_state("http://127.0.0.1:1/unused"));
    let body = include_str!("fixtures/issue_hook.json");
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
async fn note_hook_is_skipped_with_ok() {
    let app = routes::router(test_state("http://127.0.0.1:1/unused"));
    let body = include_str!("fixtures/note_hook.json");
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
async fn duplicate_open_webhook_is_skipped() {
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

    let app = routes::router(test_state(&cursor_url));
    let body = include_str!("fixtures/mr_open.json");

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhook")
                .header("content-type", "application/json")
                .header("X-Gitlab-Event-UUID", "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(
        response_json(first).await,
        serde_json::json!({ "status": "forwarded", "cursor_status": 202 })
    );

    let second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhook")
                .header("content-type", "application/json")
                .header("X-Gitlab-Event-UUID", "ffffffff-bbbb-cccc-dddd-eeeeeeeeeeee")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(
        response_json(second).await,
        serde_json::json!({ "status": "skipped" })
    );
}

#[tokio::test]
async fn mapped_project_uses_project_webhook_url() {
    let default_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind default mock cursor");
    let mapped_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mapped mock cursor");
    let default_addr = default_listener.local_addr().unwrap();
    let mapped_addr = mapped_listener.local_addr().unwrap();
    let default_url = format!("http://{default_addr}/default");
    let mapped_url = format!("http://{mapped_addr}/mapped");

    let (default_hit, mapped_hit) = (std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)), std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)));
    let default_counter = default_hit.clone();
    let mapped_counter = mapped_hit.clone();

    tokio::spawn(async move {
        axum::serve(
            default_listener,
            axum::Router::new().route(
                "/default",
                axum::routing::post(move || {
                    let counter = default_counter.clone();
                    async move {
                        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        (StatusCode::ACCEPTED, "ok")
                    }
                }),
            ),
        )
        .await
        .expect("default mock cursor server");
    });

    tokio::spawn(async move {
        axum::serve(
            mapped_listener,
            axum::Router::new().route(
                "/mapped",
                axum::routing::post(move || {
                    let counter = mapped_counter.clone();
                    async move {
                        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        (StatusCode::ACCEPTED, "ok")
                    }
                }),
            ),
        )
        .await
        .expect("mapped mock cursor server");
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let config = Arc::new(Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        gitlab_webhook_secret: None,
        allowed_users: ["plasticdigits", "brouie"]
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>(),
        project_webhooks: [
            (
                "group/other-project".to_string(),
                ProjectCursorConfig {
                    webhook_url: mapped_url.clone(),
                    token: "crsr_mapped".to_string(),
                },
            ),
            (
                "group/example-project".to_string(),
                ProjectCursorConfig {
                    webhook_url: default_url.clone(),
                    token: "crsr_default".to_string(),
                },
            ),
        ]
        .into_iter()
        .collect(),
        dedup_ttl_secs: 86_400,
    });
    let state = routes::AppState {
        config: config.clone(),
        client: Client::new(),
        dedup: Arc::new(gitlab_cursor_webhook::dedup::DedupCache::new(
            config.dedup_ttl_secs,
        )),
    };
    let app = routes::router(state);

    let mut mapped_body: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/mr_open.json")).unwrap();
    mapped_body["project"]["id"] = serde_json::json!(200);
    mapped_body["project"]["path_with_namespace"] = serde_json::json!("group/other-project");
    mapped_body["project"]["name"] = serde_json::json!("other-project");
    let mapped_body = mapped_body.to_string();

    let mapped_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhook")
                .header("content-type", "application/json")
                .body(Body::from(mapped_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(mapped_response.status(), StatusCode::OK);
    assert_eq!(
        response_json(mapped_response).await,
        serde_json::json!({ "status": "forwarded", "cursor_status": 202 })
    );
    assert_eq!(mapped_hit.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(default_hit.load(std::sync::atomic::Ordering::SeqCst), 0);

    let default_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhook")
                .header("content-type", "application/json")
                .body(Body::from(include_str!("fixtures/mr_open.json")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(default_response.status(), StatusCode::OK);
    assert_eq!(
        response_json(default_response).await,
        serde_json::json!({ "status": "forwarded", "cursor_status": 202 })
    );
    assert_eq!(mapped_hit.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(default_hit.load(std::sync::atomic::Ordering::SeqCst), 1);
}
