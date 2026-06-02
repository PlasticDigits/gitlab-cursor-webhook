// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{body::Bytes, http::StatusCode};
use reqwest::Client;
use serde::Serialize;
use thiserror::Error;

use crate::config::Config;
use crate::filter::GitLabMrWebhook;

#[derive(Debug, Serialize)]
pub struct CursorPayload<'a> {
    pub event_type: &'a str,
    pub username: &'a str,
    pub project_name: &'a str,
    pub web_url: &'a str,
    pub description: &'a str,
    pub iid: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_commit_sha: Option<&'a str>,
    pub title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit: Option<&'a crate::filter::LastCommit>,
}

impl<'a> CursorPayload<'a> {
    pub fn from_gitlab(payload: &'a GitLabMrWebhook) -> Self {
        let attrs = &payload.object_attributes;
        Self {
            event_type: &attrs.action,
            username: &payload.user.username,
            project_name: &payload.project.name,
            web_url: &attrs.url,
            description: attrs.description.as_deref().unwrap_or(""),
            iid: attrs.iid,
            merge_commit_sha: attrs.merge_commit_sha.as_deref(),
            title: &attrs.title,
            last_commit: attrs.last_commit.as_ref(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CursorForwardError {
    #[error("cursor request failed: {0}")]
    Request(#[from] reqwest::Error),
}

pub async fn forward_to_cursor(
    client: &Client,
    config: &Config,
    payload: &GitLabMrWebhook,
) -> Result<(StatusCode, Bytes), CursorForwardError> {
    let body = CursorPayload::from_gitlab(payload);
    let response = client
        .post(&config.cursor_webhook_url)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!("Bearer {}", config.cursor_token),
        )
        .json(&body)
        .send()
        .await?;

    let status =
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let bytes = response.bytes().await?;
    Ok((status, bytes))
}
