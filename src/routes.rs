// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{error, info, warn};

use crate::config::Config;
use crate::cursor::forward_to_cursor;
use crate::dedup::{commit_key, DedupCache};
use crate::filter::{should_forward, GitLabMrWebhook, SkipReason, WebhookEnvelope};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub client: Client,
    pub dedup: Arc<DedupCache>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/webhook", post(webhook))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

fn verify_gitlab_token(headers: &HeaderMap, secret: Option<&str>) -> bool {
    match secret {
        None => true,
        Some(expected) => headers
            .get("X-Gitlab-Token")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|token| token == expected),
    }
}

async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !verify_gitlab_token(&headers, state.config.gitlab_webhook_secret.as_deref()) {
        warn!("gitlab webhook rejected: invalid or missing X-Gitlab-Token");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let envelope: WebhookEnvelope = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, "failed to parse gitlab webhook envelope");
            return ok_response(json!({ "status": "skipped" }));
        }
    };

    if envelope.object_kind != "merge_request" {
        info!(
            object_kind = %envelope.object_kind,
            skipped_reason = SkipReason::NotMergeRequest.as_str(),
            forwarded = false,
            "webhook skipped"
        );
        return ok_response(json!({ "status": "skipped" }));
    }

    let payload: GitLabMrWebhook = match serde_json::from_slice(&body) {
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
        info!(
            action = %action,
            iid,
            username = %username,
            skipped_reason = reason.as_str(),
            forwarded = false,
            "webhook skipped"
        );
        return ok_response(json!({ "status": "skipped" }));
    }

    let Some(cursor) = state.config.cursor_config_for(&payload.project) else {
        info!(
            action = %action,
            iid,
            username = %username,
            project = payload.project.path_with_namespace.as_deref().unwrap_or(""),
            skipped_reason = SkipReason::ProjectNotConfigured.as_str(),
            forwarded = false,
            "webhook skipped"
        );
        return ok_response(json!({ "status": "skipped" }));
    };

    if let Some(key) = commit_key(&payload) {
        if state.dedup.is_duplicate(&key) {
            info!(
                action = %action,
                iid,
                username = %username,
                skipped_reason = SkipReason::Duplicate.as_str(),
                forwarded = false,
                commit_key = %key,
                "webhook skipped"
            );
            return ok_response(json!({ "status": "skipped" }));
        }
    }

    match forward_to_cursor(
        &state.client,
        &cursor.webhook_url,
        &cursor.token,
        &payload,
    )
    .await
    {
        Ok((status, _response_body)) => {
            let cursor_status = status.as_u16();
            if status.is_success() {
                info!(
                    action = %action,
                    iid,
                    username = %username,
                    forwarded = true,
                    cursor_status,
                    "webhook forwarded to cursor"
                );
                ok_response(json!({
                    "status": "forwarded",
                    "cursor_status": cursor_status,
                }))
            } else if status.is_server_error() {
                error!(
                    action = %action,
                    iid,
                    username = %username,
                    forwarded = true,
                    cursor_status,
                    "cursor returned server error"
                );
                ok_response(json!({
                    "status": "forward_failed",
                    "cursor_status": cursor_status,
                }))
            } else {
                warn!(
                    action = %action,
                    iid,
                    username = %username,
                    forwarded = true,
                    cursor_status,
                    "cursor rejected webhook"
                );
                ok_response(json!({
                    "status": "forward_failed",
                    "cursor_status": cursor_status,
                }))
            }
        }
        Err(e) => {
            error!(
                action = %action,
                iid,
                username = %username,
                forwarded = true,
                error = %e,
                "failed to forward webhook to cursor"
            );
            ok_response(json!({
                "status": "forward_failed",
                "error": "failed to reach cursor webhook",
            }))
        }
    }
}

fn ok_response(body: Value) -> Response {
    (StatusCode::OK, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gitlab_token_check_skipped_when_secret_unset() {
        let headers = HeaderMap::new();
        assert!(verify_gitlab_token(&headers, None));
    }

    #[test]
    fn gitlab_token_check_requires_match() {
        let mut headers = HeaderMap::new();
        assert!(!verify_gitlab_token(&headers, Some("secret")));

        headers.insert("X-Gitlab-Token", "wrong".parse().unwrap());
        assert!(!verify_gitlab_token(&headers, Some("secret")));

        headers.insert("X-Gitlab-Token", "secret".parse().unwrap());
        assert!(verify_gitlab_token(&headers, Some("secret")));
    }
}
