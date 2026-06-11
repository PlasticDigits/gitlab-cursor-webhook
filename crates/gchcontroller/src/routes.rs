// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use gch_core::{
    db::Database,
    dedup::{commit_key, issue_key, DedupCache},
    filter::{
        should_forward, should_forward_issue, GitLabIssueWebhook,
        GitLabMrWebhook, IssueAgent, Project, SkipReason, WebhookEnvelope,
    },
    prompt::{render_prompt, PromptContext},
    tag::WebhookTag,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::ControllerConfig;
use crate::jobs::JobStore;
use crate::provision::{provision_job, ProvisionError, ProvisionRequest};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ControllerConfig>,
    pub db: Arc<Database>,
    pub jobs: Arc<JobStore>,
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
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .map(str::to_string)
}

async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(e) = state.config.gitlab_webhook.verify(&body, &headers) {
        warn!(error = %e, "gitlab webhook rejected: invalid signature");
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

    if let Err(reason) = should_forward(&payload, &state.config.allowed_users) {
        log_skip(&action, iid, &username, None, &reason);
        return ok_response(json!({ "status": "skipped" }));
    }

    let tag = WebhookTag::Security;
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

    if let Some(key) = commit_key(&payload) {
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
            tag,
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

    let agents = match should_forward_issue(&payload, &state.config.allowed_users) {
        Ok(agents) => agents,
        Err(reason) => {
            log_skip(&action, iid, &username, None, &reason);
            return ok_response(json!({ "status": "skipped" }));
        }
    };

    let Some(agent) = resolve_issue_agent(&agents, &state.db, &payload.project) else {
        log_skip(
            &action,
            iid,
            &username,
            None,
            &SkipReason::ProjectNotConfigured,
        );
        return ok_response(json!({ "status": "skipped" }));
    };

    let tag = WebhookTag::from_issue_agent(agent);
    let dedup_key = issue_key(&payload.project, iid);
    if state.issue_dedup.is_duplicate(&dedup_key) {
        log_skip(&action, iid, &username, Some(agent.as_str()), &SkipReason::Duplicate);
        return ok_response(json!({ "status": "skipped" }));
    }

    let Some(resolved) = state.db.resolve_tag(&payload.project, tag).ok().flatten() else {
        log_skip(
            &action,
            iid,
            &username,
            Some(agent.as_str()),
            &SkipReason::ProjectNotConfigured,
        );
        return ok_response(json!({ "status": "skipped" }));
    };

    let ctx = issue_prompt_context(&payload, agent);
    let prompt = render_prompt(&resolved.prompt_template, &ctx);

    provision_result(
        state,
        ProvisionRequest {
            project_gitlab_path: resolved.project.gitlab_path.clone(),
            tag,
            iid,
            object_kind: "issue".to_string(),
            prompt,
            resolved,
            git_ref: None,
        },
        &action,
        iid,
        &username,
        Some(agent.as_str()),
    )
    .await
}

fn resolve_issue_agent(
    agents: &[IssueAgent],
    db: &Database,
    project: &Project,
) -> Option<IssueAgent> {
    for agent in [IssueAgent::Implement, IssueAgent::Verify] {
        if !agents.contains(&agent) {
            continue;
        }
        let tag = WebhookTag::from_issue_agent(agent);
        if db.resolve_tag(project, tag).ok().flatten().is_some() {
            return Some(agent);
        }
    }
    None
}

async fn provision_result(
    state: &AppState,
    req: ProvisionRequest,
    action: &str,
    iid: u64,
    username: &str,
    agent: Option<&str>,
) -> Response {
    match provision_job(&state.config, &state.jobs, req).await {
        Ok((job_id, _token)) => {
            info!(
                action = %action,
                iid,
                username = %username,
                agent,
                job_id = %job_id,
                provisioned = true,
                "job provisioned"
            );
            ok_response(json!({
                "status": "provisioned",
                "job_id": job_id,
            }))
        }
        Err(ProvisionError::MaxConcurrentJobs) => {
            warn!(
                action = %action,
                iid,
                username = %username,
                agent,
                "max concurrent jobs reached"
            );
            ok_response(json!({
                "status": "provision_failed",
                "error": "max_concurrent_jobs",
            }))
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
    if state.jobs.verify_token(job_id, &token).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let success = body.status.eq_ignore_ascii_case("success");
    let msg = body
        .message
        .or_else(|| body.exit_code.map(|c| format!("exit_code={c}")));

    if state.jobs.mark_complete(job_id, success, msg).await.is_some() {
        // Reaper will destroy; trigger async destroy for faster cleanup
        let jobs = state.jobs.clone();
        tokio::spawn(async move {
            let _ = crate::provision::destroy_job(&jobs, job_id).await;
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

fn issue_prompt_context(payload: &GitLabIssueWebhook, agent: IssueAgent) -> PromptContext {
    let mut ctx = PromptContext::default();
    ctx.insert("event_type", &payload.object_attributes.action);
    ctx.insert("agent", agent.as_str());
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
