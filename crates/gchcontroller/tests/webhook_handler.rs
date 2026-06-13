// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use gchcontroller::jobs::{JobRecord, JobStatus};

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
    db.set_project_signing_token("group/example-project", TEST_SIGNING_TOKEN)
        .expect("signing token");
    db.add_tag("example-project", "security", "snap-sec", None, None, None)
        .expect("tag");
    db.add_tag("example-project", "verify", "snap-verify", None, None, None)
        .expect("tag");
    db.add_tag("example-project", "implement", "snap-impl", None, None, None)
        .expect("tag");
    db.add_tag("example-project", "fix_conflicts", "snap-fix", None, None, None)
        .expect("tag");
    db.set_prompt("example-project", "security", "Review MR {{title}}")
        .expect("prompt");
    db.set_prompt("example-project", "fix_conflicts", "Fix conflicts {{title}}")
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
        allowed_users: ["plasticdigits", "brouie"]
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>(),
        dedup_ttl_secs: 86_400,
        issue_dedup_ttl_secs: 900,
        db_path: "/tmp/gch-test.db".into(),
        hcloud_token: "test".into(),
        firewall_id: "fw-test".into(),
        ssh_key_refs: vec![],
        ssh_key_ids: vec![],
        controller_url: "http://127.0.0.1:8080".into(),
        cursor_api_key: "cursor-test".into(),
        gitlab_token: "gitlab-test".into(),
        jobs_dir: std::env::temp_dir().join("gch-test-jobs"),
        terraform_module_dir: root.join("terraform/modules/agent-vm"),
        cloud_init_template: root.join("templates/cloud_init.yaml.tpl"),
        provision_enabled: false,
        admin_token: Some("admin-test-token".into()),
        hetzner_server_limit: 15,
        hetzner_server_queue_threshold: 14,
        provision_queue_retry_secs: 1800,
        settings: Settings {
            controller_url: "http://127.0.0.1:8080".into(),
            firewall_id: "fw-test".into(),
            job_timeout_secs: 10_800,
            heartbeat_stale_secs: 300,
            provisioning_timeout_secs: 900,
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
    assert_eq!(json["status"], "accepted");
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
    assert_eq!(json["status"], "accepted");
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
    assert_eq!(response_json(first).await["status"], "accepted");

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
    assert_eq!(response_json(response).await["status"], "accepted");
}

#[tokio::test]
async fn implement_then_verify_on_same_issue_both_provision() {
    let app = routes::router(test_state());
    let implement = include_str!("fixtures/issue_update_label_added.json");
    let verify = include_str!("fixtures/issue_update_verify_label_added.json");

    let first = app
        .clone()
        .oneshot(signed_webhook_request(implement))
        .await
        .unwrap();
    assert_eq!(response_json(first).await["status"], "accepted");

    let second = app.oneshot(signed_webhook_request(verify)).await.unwrap();
    assert_eq!(response_json(second).await["status"], "accepted");
}

#[tokio::test]
async fn mr_update_with_fix_conflicts_label_provisions() {
    let app = routes::router(test_state());
    let body = include_str!("fixtures/mr_update_fix_conflicts_label.json");
    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(response_json(response).await["status"], "accepted");
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
    assert_eq!(response_json(first).await["status"], "accepted");

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

#[tokio::test]
async fn admin_jobs_lists_in_memory_jobs() {
    let state = test_state();
    let job_id = uuid::Uuid::new_v4();
    state
        .jobs
        .insert(JobRecord {
            job_id,
            token_hash: "hash".into(),
            project_gitlab_path: "group/example-project".into(),
            tag: "security".into(),
            iid: 42,
            object_kind: "merge_request".into(),
            prompt: "review".into(),
            model: "composer-2.5".into(),
            hetzner_snapshot_id: "snap-sec".into(),
            workspace_path: "/home/agent/workspace".into(),
            git_ref: Some("main".into()),
            status: JobStatus::Running,
            phase: Some("agent".into()),
            status_message: None,
            runtime_token: None,
            retry_at: None,
            queue_attempts: 0,
            created_at: Utc::now(),
            provisioning_started_at: Some(Utc::now()),
            last_heartbeat: None,
            completed_at: None,
            terraform_dir: state.config.jobs_dir.join(job_id.to_string()),
            server_id: None,
        })
        .await;

    let app = routes::router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/jobs")
                .header("Authorization", "Bearer admin-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["jobs"].as_array().unwrap().len(), 1);
    assert_eq!(json["jobs"][0]["job_id"], job_id.to_string());
    assert_eq!(json["jobs"][0]["status"], "running");
}

#[tokio::test]
async fn issue_open_with_gap_analysis_label_provisions() {
    let state = test_state();
    state
        .db
        .add_tag("example-project", "gap_analysis", "snap-gap", None, None, None)
        .expect("tag");
    state
        .db
        .set_prompt("example-project", "gap_analysis", "Gap analysis {{title}}")
        .expect("prompt");

    let body = r#"{
        "object_kind": "issue",
        "user": { "username": "plasticdigits" },
        "project": {
            "id": 1,
            "name": "example-project",
            "path_with_namespace": "group/example-project"
        },
        "labels": [{ "title": "agent:gap_analysis" }],
        "object_attributes": {
            "action": "open",
            "iid": 99,
            "title": "Run gap analysis",
            "description": "Scope notes",
            "url": "https://gitlab.example/group/example-project/-/issues/99"
        }
    }"#;

    let app = routes::router(state);
    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(response_json(response).await["status"], "accepted");
}

#[tokio::test]
async fn issue_open_with_reserved_security_label_is_skipped() {
    let app = routes::router(test_state());
    let body = r#"{
        "object_kind": "issue",
        "user": { "username": "plasticdigits" },
        "project": {
            "id": 1,
            "name": "example-project",
            "path_with_namespace": "group/example-project"
        },
        "labels": [{ "title": "agent:security" }],
        "object_attributes": {
            "action": "open",
            "iid": 100,
            "title": "Security audit issue",
            "url": "https://gitlab.example/group/example-project/-/issues/100"
        }
    }"#;

    let response = app.oneshot(signed_webhook_request(body)).await.unwrap();
    assert_eq!(
        response_json(response).await,
        serde_json::json!({ "status": "skipped" })
    );
}

#[tokio::test]
async fn admin_jobs_requires_token() {
    let state = test_state();
    let app = routes::router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
