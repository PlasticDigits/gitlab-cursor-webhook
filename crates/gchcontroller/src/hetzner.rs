// SPDX-License-Identifier: AGPL-3.0-or-later

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum HetznerError {
    #[error("hetzner api request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("hetzner api error: {0}")]
    Api(String),
}

#[derive(Debug, Deserialize)]
struct ServerListResponse {
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Meta {
    pagination: Pagination,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    total_entries: usize,
}

/// Count servers in the Hetzner project via the Cloud API.
pub async fn count_servers(token: &str) -> Result<usize, HetznerError> {
    let client = Client::new();
    let response = client
        .get("https://api.hetzner.cloud/v1/servers")
        .bearer_auth(token)
        .query(&[("per_page", "1")])
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(HetznerError::Api(format!("{status}: {body}")));
    }

    let list: ServerListResponse = response.json().await?;
    Ok(list.meta.pagination.total_entries)
}

/// Returns true when we should defer provisioning because the account is near its server cap.
pub fn near_server_limit(count: usize, limit: u32, queue_threshold: u32) -> bool {
    count >= queue_threshold as usize || count >= limit as usize
}

/// Terraform/Hetzner errors that may clear after capacity frees up.
pub fn is_retryable_provision_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("resource_limit_exceeded")
        || lower.contains("server limit")
        || lower.contains("resource_unavailable")
        || lower.contains("no server type available")
        || lower.contains("location is not available")
}

pub async fn count_servers_best_effort(token: &str) -> Option<usize> {
    match count_servers(token).await {
        Ok(count) => Some(count),
        Err(e) => {
            warn!(error = %e, "failed to count hetzner servers, skipping limit check");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_limit_at_threshold() {
        assert!(near_server_limit(14, 15, 14));
        assert!(near_server_limit(15, 15, 14));
        assert!(!near_server_limit(13, 15, 14));
    }

    #[test]
    fn retryable_errors() {
        assert!(is_retryable_provision_error(
            "Error: resource_limit_exceeded: server limit reached"
        ));
        assert!(is_retryable_provision_error("Server limit reached"));
        assert!(!is_retryable_provision_error("invalid snapshot id"));
    }
}
