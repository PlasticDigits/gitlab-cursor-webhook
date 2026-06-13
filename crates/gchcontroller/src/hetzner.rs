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
    next_page: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct SshKey {
    id: u64,
    name: String,
    fingerprint: String,
}

#[derive(Debug, Deserialize)]
struct SshKeyListResponse {
    ssh_keys: Vec<SshKey>,
    meta: Meta,
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

/// Normalize SSH key fingerprints for comparison (strip colons/spaces, lowercase).
pub fn normalize_fingerprint(fingerprint: &str) -> String {
    fingerprint
        .chars()
        .filter(|c| *c != ':' && !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

pub fn is_numeric_ssh_key_id(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_digit())
}

/// List all SSH keys in the Hetzner project.
async fn list_ssh_keys(token: &str) -> Result<Vec<SshKey>, HetznerError> {
    let client = Client::new();
    let mut keys = Vec::new();
    let mut page = 1u32;

    loop {
        let page_str = page.to_string();
        let response = client
            .get("https://api.hetzner.cloud/v1/ssh_keys")
            .bearer_auth(token)
            .query(&[("per_page", "50"), ("page", page_str.as_str())])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(HetznerError::Api(format!("{status}: {body}")));
        }

        let list: SshKeyListResponse = response.json().await?;
        keys.extend(list.ssh_keys);

        match list.meta.pagination.next_page {
            Some(next) => page = next,
            None => break,
        }
    }

    Ok(keys)
}

/// Resolve `GCH_SSH_KEY_IDS` entries to numeric Hetzner SSH key IDs.
///
/// Each entry may be a numeric ID, key name (e.g. `admin-ceramic`), or fingerprint
/// (with or without colons).
pub async fn resolve_ssh_key_ids(
    token: &str,
    refs: &[String],
) -> Result<Vec<String>, HetznerError> {
    if refs.is_empty() {
        return Ok(vec![]);
    }

    let keys = list_ssh_keys(token).await?;
    let mut resolved = Vec::with_capacity(refs.len());

    for reference in refs {
        if is_numeric_ssh_key_id(reference) {
            resolved.push(reference.clone());
            continue;
        }

        let normalized_fp = normalize_fingerprint(reference);
        let matched = keys.iter().find(|key| {
            key.name == *reference
                || normalize_fingerprint(&key.fingerprint) == normalized_fp
        });

        match matched {
            Some(key) => resolved.push(key.id.to_string()),
            None => {
                return Err(HetznerError::Api(format!(
                    "ssh key not found in Hetzner project: {reference} \
                     (expected numeric id, key name, or fingerprint)"
                )));
            }
        }
    }

    Ok(resolved)
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

    #[test]
    fn normalize_fingerprint_strips_colons() {
        assert_eq!(
            normalize_fingerprint("A0:6A:F8:BF:2D:C4:14:6B:C6:68:8B:39:49:38:A2:B4"),
            "a06af8bf2dc4146bc6688b394938a2b4"
        );
    }

    #[test]
    fn numeric_ssh_key_id() {
        assert!(is_numeric_ssh_key_id("12345"));
        assert!(!is_numeric_ssh_key_id("admin-ceramic"));
        assert!(!is_numeric_ssh_key_id(""));
    }
}
