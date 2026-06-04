// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Minimal GitLab webhook header used before full MR deserialization.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct WebhookEnvelope {
    pub object_kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GitLabMrWebhook {
    pub object_kind: String,
    pub user: User,
    pub project: Project,
    pub object_attributes: ObjectAttributes,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub username: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Project {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub path_with_namespace: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LastCommit {
    pub id: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ObjectAttributes {
    pub action: String,
    #[serde(default)]
    pub oldrev: Option<String>,
    pub iid: u64,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub url: String,
    #[serde(default)]
    pub merge_commit_sha: Option<String>,
    #[serde(default)]
    pub source_project_id: Option<u64>,
    #[serde(default)]
    pub target_project_id: Option<u64>,
    #[serde(default)]
    pub last_commit: Option<LastCommit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    NotMergeRequest,
    ActionFiltered { action: String },
    ForkMr,
    UserNotAllowed { username: String },
    ProjectNotConfigured,
    Duplicate,
}

impl SkipReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkipReason::NotMergeRequest => "not_merge_request",
            SkipReason::ActionFiltered { .. } => "action_filtered",
            SkipReason::ForkMr => "fork_mr",
            SkipReason::UserNotAllowed { .. } => "user_not_allowed",
            SkipReason::ProjectNotConfigured => "project_not_configured",
            SkipReason::Duplicate => "duplicate",
        }
    }
}

/// Returns `Ok(())` when the payload should be forwarded to Cursor.
pub fn should_forward(
    payload: &GitLabMrWebhook,
    allowed_users: &HashSet<String>,
) -> Result<(), SkipReason> {
    if payload.object_kind != "merge_request" {
        return Err(SkipReason::NotMergeRequest);
    }

    let action = payload.object_attributes.action.as_str();
    let forwardable_action = match action {
        "open" => true,
        "update" => payload
            .object_attributes
            .oldrev
            .as_ref()
            .is_some_and(|rev| !rev.is_empty()),
        _ => false,
    };
    if !forwardable_action {
        return Err(SkipReason::ActionFiltered {
            action: payload.object_attributes.action.clone(),
        });
    }

    let attrs = &payload.object_attributes;
    if let (Some(source), Some(target)) = (attrs.source_project_id, attrs.target_project_id) {
        if source != target {
            return Err(SkipReason::ForkMr);
        }
    }

    if !allowed_users.contains(&payload.user.username) {
        return Err(SkipReason::UserNotAllowed {
            username: payload.user.username.clone(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_payload(action: &str) -> GitLabMrWebhook {
        GitLabMrWebhook {
            object_kind: "merge_request".to_string(),
            user: User {
                username: "plasticdigits".to_string(),
            },
            project: Project {
                id: 1,
                name: "test-project".to_string(),
                path_with_namespace: Some("group/test-project".to_string()),
            },
            object_attributes: ObjectAttributes {
                action: action.to_string(),
                oldrev: None,
                iid: 42,
                title: "Test MR".to_string(),
                description: Some("A description".to_string()),
                url: "https://gitlab.example/project/-/merge_requests/42".to_string(),
                merge_commit_sha: None,
                source_project_id: Some(1),
                target_project_id: Some(1),
                last_commit: Some(LastCommit {
                    id: "abc123".to_string(),
                    message: Some("commit msg".to_string()),
                    timestamp: None,
                    url: None,
                }),
            },
        }
    }

    fn allowlist() -> HashSet<String> {
        ["plasticdigits", "brouie"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn open_same_project_forwards() {
        let payload = base_payload("open");
        assert!(should_forward(&payload, &allowlist()).is_ok());
    }

    #[test]
    fn update_with_oldrev_forwards() {
        let mut payload = base_payload("update");
        payload.object_attributes.oldrev = Some("deadbeef".to_string());
        assert!(should_forward(&payload, &allowlist()).is_ok());
    }

    #[test]
    fn update_without_oldrev_skips() {
        let payload = base_payload("update");
        let err = should_forward(&payload, &allowlist()).unwrap_err();
        assert!(matches!(err, SkipReason::ActionFiltered { .. }));
    }

    #[test]
    fn approval_skips() {
        let payload = base_payload("approval");
        let err = should_forward(&payload, &allowlist()).unwrap_err();
        assert!(matches!(err, SkipReason::ActionFiltered { .. }));
    }

    #[test]
    fn approved_skips() {
        let payload = base_payload("approved");
        let err = should_forward(&payload, &allowlist()).unwrap_err();
        assert!(matches!(err, SkipReason::ActionFiltered { .. }));
    }

    #[test]
    fn fork_mr_skips() {
        let mut payload = base_payload("open");
        payload.object_attributes.source_project_id = Some(2);
        payload.object_attributes.target_project_id = Some(1);
        let err = should_forward(&payload, &allowlist()).unwrap_err();
        assert_eq!(err, SkipReason::ForkMr);
    }

    #[test]
    fn allowlist_allows_known_user() {
        let payload = base_payload("open");
        assert!(should_forward(&payload, &allowlist()).is_ok());
    }

    #[test]
    fn allowlist_denies_unknown_user() {
        let mut payload = base_payload("open");
        payload.user.username = "stranger".to_string();
        let err = should_forward(&payload, &allowlist()).unwrap_err();
        assert_eq!(
            err,
            SkipReason::UserNotAllowed {
                username: "stranger".to_string()
            }
        );
    }

    #[test]
    fn non_merge_request_skips() {
        let mut payload = base_payload("open");
        payload.object_kind = "push".to_string();
        let err = should_forward(&payload, &allowlist()).unwrap_err();
        assert_eq!(err, SkipReason::NotMergeRequest);
    }
}
