// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use gch_core::{db::Database, dedup::DedupCache};
use gchcontroller::{
    config::ControllerConfig,
    jobs::JobStore,
    routes::{self, AppState},
};
use gch_core::db::Settings;
use http_body_util::BodyExt;
use serde_json::Value;
use standardwebhooks::{
    Webhook, HEADER_WEBHOOK_ID, HEADER_WEBHOOK_SIGNATURE, HEADER_WEBHOOK_TIMESTAMP,
};
use tower::ServiceExt;

const TEST_SIGNING_TOKEN: &str = "whsec_C2FVsBQIhrscChlQIMV+b5sSYspob7oD";

fn sign_webhook(body: &[u8]) -> HeaderMap {
    let wh = Webhook::new(TEST_SIGNING_TOKEN).unwrap();
    let msg_id = "msg_integration_test";
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let signature = wh.sign(msg_id, timestamp, body).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(HEADER_WEBHOOK_ID, msg_id.parse().unwrap());
    headers.insert(HEADER_WEBHOOK_TIMESTAMP, timestamp.to_string().parse().unwrap());
    headers.insert(HEADER_WEBHOOK_SIGNATURE, signature.parse().unwrap());
    headers
}

fn signed_webhook_request(body: &str) -> Request<Body> {
    let bytes = body.as_bytes().to_vec();
    let signed = sign_webhook(&bytes);
    let mut builder = Request::builder()
        .method("POST")
        .uri("/webhook")
        .header("content-type", "application/json");
    for (name, value) in signed.iter() {
        builder = builder.header(name, value);
    }
    builder.body(Body::from(bytes)).unwrap()
}

fn setup_db() -> Arc<Database> {
    let db = Database::open_in_memory().expect("db");
    db.add_project(
        "group/example-project",
        "example-project",
        "/home/agent/workspace",
    )
    .expect("project");
    db.add_tag("example-project", "security", "snap-sec", None, None, None)
        .expect("tag");
    db.add_tag("example-project", "verify", "snap-verify", None, None, None)
        .expect("tag");
    db.add_tag("example-project", "implement", "snap-impl", None, None, None)
        .expect("tag");
    db.set_prompt("example-project", "security", "Review MR {{title}}")
        .expect("prompt");
    db.set_prompt("example-project", "verify", "Verify issue {{title}}")
        .expect("prompt");
    db.set_prompt("example-project", "implement", "Implement issue {{title}}")
        .expect("prompt");
    db.set_setting("controller_url", "http://127.0.0.1:8080")
        .expect("setting");
    db.set_setting("firewall_id", "fw-test").expect("setting");
    Arc::new(db)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn test_config() -> Arc<ControllerConfig> {
    let root = repo_root();
    Arc::new(ControllerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        gitlab_webhook: Webhook::new(TEST_SIGNING_TOKEN).unwrap(),
        allowed_users: ["plasticdigits", "brouie"]
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>(),
        dedup_ttl_secs: 86_400,
        issue_dedup_ttl_secs: 900,
        db_path: "/tmp/gch-test.db".into(),
        hcloud_token: "test".into(),
        firewall_id: "fw-test".into(),
        ssh_key_ids: vec![],
        controller_url: "http://127.0.0.1:8080".into(),
        cursor_api_key: "cursor-test".into(),
        gitlab_token: "gitlab-test".into(),
        jobs_dir: std::env::temp_dir().join("gch-test-jobs"),
        terraform_module_dir: root.join("terraform/modules/agent-vm"),
        cloud_init_template: root.join("templates/cloud_init.yaml.tpl"),
        provision_enabled: false,
        settings: Settings {
            controller_url: "http://127.0.0.1:8080".into(),
            firewall_id: "fw-test".into(),
            job_timeout_secs: 10_800,
            heartbeat_stale_secs: 300,
            max_concurrent_jobs: 10,
        },
    })
}

fn test_state() -> AppState {
    let config = test_config();
    AppState {
        config: config.clone(),
        db: setup_db(),
        jobs: JobStore::new(),
        dedup: Arc::new(DedupCache::new(config.dedup_ttl_secs)),
        issue_dedup: Arc::new(DedupCache::new(config.issue_dedup_ttl_secs)),
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
    let app = routes::router(test_state());
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
async fn unsigned_webhook_is_rejected() {
    let app = routes::router(test_state());
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
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn approval_webhook_is_skipped() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/mr_approval.json");
    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await,
        serde_json::json!({ "status": "skipped" })
    );
}

#[tokio::test]
async fn open_webhook_provisions_job() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/mr_open.json");
    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["status"], "provisioned");
    assert!(json["job_id"].is_string());
}

#[tokio::test]
async fn issue_hook_without_agent_label_is_skipped_with_ok() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/issue_hook.json");
    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await,
        serde_json::json!({ "status": "skipped" })
    );
}

#[tokio::test]
async fn issue_open_with_verify_label_provisions() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/issue_open_with_verify_label.json");
    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["status"], "provisioned");
}

#[tokio::test]
async fn duplicate_issue_webhook_is_skipped() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/issue_open_with_verify_label.json");

    let first = app
        .clone()
        .oneshot(signed_webhook_request(body))
        .await
        .unwrap();
    assert_eq!(response_json(first).await["status"], "provisioned");

    let second = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(
        response_json(second).await,
        serde_json::json!({ "status": "skipped" })
    );
}

#[tokio::test]
async fn issue_update_with_label_added_provisions_implement() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/issue_update_label_added.json");
    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(response_json(response).await["status"], "provisioned");
}

#[tokio::test]
async fn duplicate_open_webhook_is_skipped() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/mr_open.json");

    let first = app
        .clone()
        .oneshot(signed_webhook_request(body))
        .await
        .unwrap();
    assert_eq!(response_json(first).await["status"], "provisioned");

    let second = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(
        response_json(second).await,
        serde_json::json!({ "status": "skipped" })
    );
}

#[tokio::test]
async fn job_api_requires_bearer_token() {
    let state = test_state();
    let app = routes::router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/jobs/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
