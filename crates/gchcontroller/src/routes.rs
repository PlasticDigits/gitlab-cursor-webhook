// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::time::Instant;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use gch_core::{
    db::Database,
    dedup::{commit_key, issue_key, DedupCache},
    job_api::{JobListResponse, JobSummary},
    server_ipv4_from_tfstate,
    filter::{
        is_mr_label_only_update, removed_agent_label_tags, select_issue_tag, should_forward,
        should_forward_issue, should_forward_mr_labels, GitLabIssueWebhook, GitLabMrWebhook,
        SkipReason, WebhookEnvelope,
    },
    prompt::{render_prompt, PromptContext},
    tag::MR_SECURITY_TAG,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use standardwebhooks::Webhook;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::admission::ProvisionAdmission;
use crate::config::ControllerConfig;
use crate::jobs::{JobRecord, JobStore};
use crate::provision::{provision_job, ProvisionRequest};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ControllerConfig>,
    pub db: Arc<Database>,
    pub jobs: Arc<JobStore>,
    pub admission: Arc<ProvisionAdmission>,
    pub dedup: Arc<DedupCache>,
    pub issue_dedup: Arc<DedupCache>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/webhook", post(webhook))
        .route("/api/jobs/{job_id}", get(get_job))
        .route("/api/jobs/{job_id}/heartbeat", post(heartbeat))
        .route("/api/jobs/{job_id}/status", post(job_status))
        .route("/api/jobs/{job_id}/complete", post(complete_job))
        .route("/api/admin/jobs", get(admin_list_jobs))
        .route("/api/admin/jobs/{job_id}", get(admin_get_job))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

fn verify_gitlab_webhook(db: &Database, headers: &HeaderMap, body: &[u8]) -> bool {
    let tokens = match db.list_project_signing_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            error!(error = %e, "failed to load project signing tokens");
            return false;
        }
    };
    if tokens.is_empty() {
        return false;
    }
    tokens.iter().any(|token| {
        Webhook::new(token)
            .ok()
            .is_some_and(|wh| wh.verify(body, headers).is_ok())
    })
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .map(str::to_string)
}

#[derive(Debug, Deserialize)]
struct AdminJobsQuery {
    active: Option<bool>,
}

fn verify_admin(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(expected) = &state.config.admin_token else {
        return false;
    };
    bearer_token(headers).as_deref() == Some(expected.as_str())
}

fn admin_auth_failed(state: &AppState) -> Response {
    if state.config.admin_token.is_none() {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "admin_api_disabled",
                "message": "set GCH_ADMIN_TOKEN on the controller to enable job listing"
            })),
        )
            .into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

fn job_to_summary(job: &JobRecord, jobs_dir: &std::path::Path, detailed: bool) -> JobSummary {
    let age_secs = Utc::now()
        .signed_duration_since(job.created_at)
        .num_seconds()
        .max(0) as u64;
    let last_heartbeat_secs_ago = job
        .last_heartbeat
        .map(|hb| Instant::now().duration_since(hb).as_secs());
    let server_ipv4 = server_ipv4_from_tfstate(&job.terraform_dir)
        .or_else(|| server_ipv4_from_tfstate(&jobs_dir.join(job.job_id.to_string())));

    JobSummary {
        job_id: job.job_id.to_string(),
        status: job.status.as_str().to_string(),
        phase: job.phase.clone(),
        status_message: job.status_message.clone(),
        project: job.project_gitlab_path.clone(),
        tag: job.tag.to_string(),
        iid: job.iid,
        object_kind: job.object_kind.clone(),
        model: job.model.clone(),
        created_at: job.created_at.to_rfc3339(),
        completed_at: job.completed_at.map(|t| t.to_rfc3339()),
        last_heartbeat_secs_ago,
        age_secs,
        server_ipv4,
        git_ref: detailed.then(|| job.git_ref.clone()).flatten(),
        workspace_path: detailed.then(|| job.workspace_path.clone()),
    }
}

async fn admin_list_jobs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminJobsQuery>,
) -> Response {
    if !verify_admin(&state, &headers) {
        return admin_auth_failed(&state);
    }

    let active_only = query.active.unwrap_or(false);
    let jobs = state.jobs.list(active_only).await;
    let summaries: Vec<JobSummary> = jobs
        .iter()
        .map(|job| job_to_summary(job, &state.config.jobs_dir, false))
        .collect();

    Json(JobListResponse { jobs: summaries }).into_response()
}

async fn admin_get_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    if !verify_admin(&state, &headers) {
        return admin_auth_failed(&state);
    }

    let Some(job) = state.jobs.get(job_id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    Json(job_to_summary(&job, &state.config.jobs_dir, true)).into_response()
}

async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !verify_gitlab_webhook(&state.db, &headers, &body) {
        warn!("gitlab webhook rejected: invalid signature or no signing tokens configured");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let envelope: WebhookEnvelope = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, "failed to parse gitlab webhook envelope");
            return ok_response(json!({ "status": "skipped" }));
        }
    };

    match envelope.object_kind.as_str() {
        "merge_request" => handle_merge_request(&state, &body).await,
        "issue" => handle_issue(&state, &body).await,
        object_kind => {
            info!(
                object_kind,
                skipped_reason = SkipReason::UnsupportedObjectKind.as_str(),
                provisioned = false,
                "webhook skipped"
            );
            ok_response(json!({ "status": "skipped" }))
        }
    }
}

async fn handle_merge_request(state: &AppState, body: &Bytes) -> Response {
    let payload: GitLabMrWebhook = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "failed to parse merge request webhook payload");
            return ok_response(json!({ "status": "skipped" }));
        }
    };

    let action = payload.object_attributes.action.clone();
    let iid = payload.object_attributes.iid;
    let username = payload.user.username.clone();

    match should_forward_mr_labels(&payload, &state.config.allowed_users) {
        Ok(tags) => {
            if let Some(tag) = select_issue_tag(&tags) {
                return handle_mr_labeled_agent(state, payload, tag, &action, iid, &username).await;
            }
        }
        Err(SkipReason::LabelNotTriggered) | Err(SkipReason::LabelReserved { .. }) => {}
        Err(reason) => {
            log_skip(&action, iid, &username, None, &reason);
            return ok_response(json!({ "status": "skipped" }));
        }
    }

    if is_mr_label_only_update(&payload) {
        log_skip(
            &action,
            iid,
            &username,
            None,
            &SkipReason::LabelNotTriggered,
        );
        return ok_response(json!({ "status": "skipped" }));
    }

    if let Err(reason) = should_forward(&payload, &state.config.allowed_users) {
        log_skip(&action, iid, &username, None, &reason);
        return ok_response(json!({ "status": "skipped" }));
    }

    let tag = MR_SECURITY_TAG;
    let Some(resolved) = state.db.resolve_tag(&payload.project, tag).ok().flatten() else {
        log_skip(
            &action,
            iid,
            &username,
            None,
            &SkipReason::ProjectNotConfigured,
        );
        return ok_response(json!({ "status": "skipped" }));
    };

    if let Some(key) = commit_key(&payload, tag) {
        if state.dedup.is_duplicate(&key) {
            log_skip(&action, iid, &username, None, &SkipReason::Duplicate);
            return ok_response(json!({ "status": "skipped" }));
        }
    }

    let ctx = mr_prompt_context(&payload);
    let prompt = render_prompt(&resolved.prompt_template, &ctx);
    let git_ref = payload
        .object_attributes
        .last_commit
        .as_ref()
        .map(|c| c.id.clone());

    provision_result(
        state,
        ProvisionRequest {
            project_gitlab_path: resolved.project.gitlab_path.clone(),
            tag: tag.to_string(),
            iid,
            object_kind: "merge_request".to_string(),
            prompt,
            resolved,
            git_ref,
        },
        &action,
        iid,
        &username,
        None,
    )
    .await
}

async fn handle_mr_labeled_agent(
    state: &AppState,
    payload: GitLabMrWebhook,
    tag: String,
    action: &str,
    iid: u64,
    username: &str,
) -> Response {
    let Some(resolved) = state
        .db
        .resolve_tag(&payload.project, &tag)
        .ok()
        .flatten()
    else {
        log_skip(
            action,
            iid,
            username,
            Some(&tag),
            &SkipReason::ProjectNotConfigured,
        );
        return ok_response(json!({ "status": "skipped" }));
    };

    let dedup_key = issue_key(&payload.project, iid, &tag);
    if state.issue_dedup.is_duplicate(&dedup_key) {
        log_skip(action, iid, username, Some(&tag), &SkipReason::Duplicate);
        return ok_response(json!({ "status": "skipped" }));
    }

    let ctx = mr_prompt_context(&payload);
    let prompt = render_prompt(&resolved.prompt_template, &ctx);
    let git_ref = payload
        .object_attributes
        .last_commit
        .as_ref()
        .map(|c| c.id.clone());

    provision_result(
        state,
        ProvisionRequest {
            project_gitlab_path: resolved.project.gitlab_path.clone(),
            tag: tag.clone(),
            iid,
            object_kind: "merge_request".to_string(),
            prompt,
            resolved,
            git_ref,
        },
        action,
        iid,
        username,
        Some(&tag),
    )
    .await
}

async fn handle_issue(state: &AppState, body: &Bytes) -> Response {
    let payload: GitLabIssueWebhook = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "failed to parse issue webhook payload");
            return ok_response(json!({ "status": "skipped" }));
        }
    };

    let action = payload.object_attributes.action.clone();
    let iid = payload.object_attributes.iid;
    let username = payload.user.username.clone();

    for tag in removed_agent_label_tags(&payload) {
        let key = issue_key(&payload.project, iid, &tag);
        state.issue_dedup.forget(&key);
    }

    let tags = match should_forward_issue(&payload, &state.config.allowed_users) {
        Ok(tags) => tags,
        Err(reason) => {
            log_skip(&action, iid, &username, None, &reason);
            return ok_response(json!({ "status": "skipped" }));
        }
    };

    let Some(tag) = state
        .db
        .resolve_issue_tag(&payload.project, &tags)
        .ok()
        .flatten()
    else {
        log_skip(
            &action,
            iid,
            &username,
            select_issue_tag(&tags).as_deref(),
            &SkipReason::ProjectNotConfigured,
        );
        return ok_response(json!({ "status": "skipped" }));
    };

    let dedup_key = issue_key(&payload.project, iid, &tag);
    if state.issue_dedup.is_duplicate(&dedup_key) {
        log_skip(&action, iid, &username, Some(&tag), &SkipReason::Duplicate);
        return ok_response(json!({ "status": "skipped" }));
    }

    let Some(resolved) = state.db.resolve_tag(&payload.project, &tag).ok().flatten() else {
        log_skip(
            &action,
            iid,
            &username,
            Some(&tag),
            &SkipReason::ProjectNotConfigured,
        );
        return ok_response(json!({ "status": "skipped" }));
    };

    let ctx = issue_prompt_context(&payload, &tag);
    let prompt = render_prompt(&resolved.prompt_template, &ctx);

    provision_result(
        state,
        ProvisionRequest {
            project_gitlab_path: resolved.project.gitlab_path.clone(),
            tag: tag.clone(),
            iid,
            object_kind: "issue".to_string(),
            prompt,
            resolved,
            git_ref: None,
        },
        &action,
        iid,
        &username,
        Some(&tag),
    )
    .await
}

async fn provision_result(
    state: &AppState,
    req: ProvisionRequest,
    action: &str,
    iid: u64,
    username: &str,
    agent: Option<&str>,
) -> Response {
    match provision_job(&state.config, &state.jobs, &state.admission, req).await {
        Ok(result) => {
            if result.queued {
                info!(
                    action = %action,
                    iid,
                    username = %username,
                    agent,
                    job_id = %result.job_id,
                    retry_at = ?result.retry_at,
                    provisioned = false,
                    "job queued, will retry when capacity is available"
                );
                ok_response(json!({
                    "status": "queued",
                    "job_id": result.job_id,
                    "retry_at": result.retry_at.map(|t| t.to_rfc3339()),
                }))
            } else {
                info!(
                    action = %action,
                    iid,
                    username = %username,
                    agent,
                    job_id = %result.job_id,
                    provisioned = true,
                    "job accepted, provisioning in background"
                );
                ok_response(json!({
                    "status": "accepted",
                    "job_id": result.job_id,
                }))
            }
        }
        Err(e) => {
            error!(
                action = %action,
                iid,
                username = %username,
                agent,
                error = %e,
                "provisioning failed"
            );
            ok_response(json!({
                "status": "provision_failed",
                "error": e.to_string(),
            }))
        }
    }
}

#[derive(Debug, Serialize)]
struct JobResponse {
    job_id: Uuid,
    prompt: String,
    model: String,
    workspace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_ref: Option<String>,
    tag: String,
    project: String,
    iid: u64,
    object_kind: String,
}

async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Some(job) = state.jobs.verify_token(job_id, &token).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    Json(JobResponse {
        job_id: job.job_id,
        prompt: job.prompt,
        model: job.model,
        workspace: job.workspace_path,
        git_ref: job.git_ref,
        tag: job.tag.to_string(),
        project: job.project_gitlab_path,
        iid: job.iid,
        object_kind: job.object_kind,
    })
    .into_response()
}

async fn heartbeat(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if state.jobs.verify_token(job_id, &token).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if state.jobs.update_heartbeat(job_id).await {
        ok_response(json!({ "status": "ok" }))
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[derive(Debug, Deserialize)]
struct StatusBody {
    phase: Option<String>,
    message: Option<String>,
}

async fn job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<StatusBody>,
) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if state.jobs.verify_token(job_id, &token).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if state
        .jobs
        .update_status(job_id, body.phase, body.message)
        .await
    {
        ok_response(json!({ "status": "ok" }))
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[derive(Debug, Deserialize)]
struct CompleteBody {
    status: String,
    #[serde(default)]
    exit_code: Option<i32>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    idle_kind: Option<String>,
    #[serde(default)]
    last_stream_event: Option<String>,
    #[serde(default)]
    idle_secs: Option<u64>,
    #[serde(default)]
    max_secs: Option<u64>,
    #[serde(default)]
    in_flight_tools: Option<u64>,
}

async fn complete_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<CompleteBody>,
) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(job) = state.jobs.verify_token(job_id, &token).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let success = body.status.eq_ignore_ascii_case("success");
    let msg = body.message.clone().or_else(|| {
        body.exit_code.map(|c| format!("exit_code={c}"))
    });

    info!(
        job_id = %job_id,
        iid = job.iid,
        tag = %job.tag,
        project = %job.project_gitlab_path,
        object_kind = %job.object_kind,
        success,
        exit_code = ?body.exit_code,
        reason = body.reason.as_deref().unwrap_or("unknown"),
        idle_kind = ?body.idle_kind,
        last_stream_event = ?body.last_stream_event,
        idle_secs = ?body.idle_secs,
        max_secs = ?body.max_secs,
        in_flight_tools = ?body.in_flight_tools,
        message = ?body.message,
        "job complete"
    );

    if state.jobs.mark_complete(job_id, success, msg).await.is_some() {
        // Reaper will destroy; trigger async destroy for faster cleanup
        let config = state.config.clone();
        let jobs = state.jobs.clone();
        tokio::spawn(async move {
            let _ = crate::provision::destroy_job(&config, jobs, job_id).await;
        });
        ok_response(json!({ "status": "ok" }))
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

fn log_skip(action: &str, iid: u64, username: &str, agent: Option<&str>, reason: &SkipReason) {
    info!(
        action = %action,
        iid,
        username = %username,
        agent,
        skipped_reason = reason.as_str(),
        provisioned = false,
        "webhook skipped"
    );
}

fn mr_prompt_context(payload: &GitLabMrWebhook) -> PromptContext {
    let mut ctx = PromptContext::default();
    ctx.insert("event_type", &payload.object_attributes.action);
    ctx.insert("username", &payload.user.username);
    ctx.insert("project_name", &payload.project.name);
    ctx.insert("web_url", &payload.object_attributes.url);
    ctx.insert(
        "description",
        payload.object_attributes.description.as_deref().unwrap_or(""),
    );
    ctx.insert("iid", payload.object_attributes.iid.to_string());
    ctx.insert("title", &payload.object_attributes.title);
    if let Some(c) = &payload.object_attributes.last_commit {
        ctx.insert("last_commit_id", &c.id);
        ctx.insert("last_commit_message", c.message.as_deref().unwrap_or(""));
    }
    ctx
}

fn issue_prompt_context(payload: &GitLabIssueWebhook, tag: &str) -> PromptContext {
    let mut ctx = PromptContext::default();
    ctx.insert("event_type", &payload.object_attributes.action);
    ctx.insert("tag", tag);
    ctx.insert("agent", tag);
    ctx.insert("username", &payload.user.username);
    ctx.insert("project_name", &payload.project.name);
    ctx.insert(
        "web_url",
        payload.object_attributes.url.as_deref().unwrap_or(""),
    );
    ctx.insert(
        "description",
        payload.object_attributes.description.as_deref().unwrap_or(""),
    );
    ctx.insert("iid", payload.object_attributes.iid.to_string());
    ctx.insert("title", &payload.object_attributes.title);
    let labels: Vec<&str> = payload.labels.iter().map(|l| l.title.as_str()).collect();
    ctx.insert("labels", labels.join(", "));
    ctx
}

fn ok_response(body: Value) -> Response {
    (StatusCode::OK, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use standardwebhooks::{
        Webhook, HEADER_WEBHOOK_ID, HEADER_WEBHOOK_SIGNATURE, HEADER_WEBHOOK_TIMESTAMP,
    };

    const TEST_SIGNING_TOKEN: &str = "whsec_C2FVsBQIhrscChlQIMV+b5sSYspob7oD";

    fn signed_headers(body: &[u8]) -> HeaderMap {
        let wh = Webhook::new(TEST_SIGNING_TOKEN).unwrap();
        let msg_id = "msg_test_webhook";
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

    #[test]
    fn webhook_signature_valid() {
        let body = br#"{"object_kind":"merge_request"}"#;
        let wh = Webhook::new(TEST_SIGNING_TOKEN).unwrap();
        assert!(wh.verify(body, &signed_headers(body)).is_ok());
    }

    #[test]
    fn webhook_signature_rejects_missing_headers() {
        let body = br#"{"object_kind":"merge_request"}"#;
        let wh = Webhook::new(TEST_SIGNING_TOKEN).unwrap();
        assert!(wh.verify(body, &HeaderMap::new()).is_err());
    }

    #[test]
    fn webhook_signature_rejects_tampered_body() {
        let body = br#"{"object_kind":"merge_request"}"#;
        let wh = Webhook::new(TEST_SIGNING_TOKEN).unwrap();
        let tampered = br#"{"object_kind":"issue"}"#;
        assert!(wh.verify(tampered, &signed_headers(body)).is_err());
    }
}
