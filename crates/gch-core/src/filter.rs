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

pub const LABEL_AGENT_VERIFY: &str = "agent:verify";
pub const LABEL_AGENT_IMPLEMENT: &str = "agent:implement";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Label {
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LabelChange {
    pub previous: Vec<Label>,
    pub current: Vec<Label>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct IssueChanges {
    #[serde(default)]
    pub labels: Option<LabelChange>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IssueObjectAttributes {
    pub action: String,
    pub iid: u64,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GitLabIssueWebhook {
    pub object_kind: String,
    pub user: User,
    pub project: Project,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub changes: Option<IssueChanges>,
    pub object_attributes: IssueObjectAttributes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueAgent {
    Verify,
    Implement,
}

impl IssueAgent {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueAgent::Verify => "verify",
            IssueAgent::Implement => "implement",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            IssueAgent::Verify => LABEL_AGENT_VERIFY,
            IssueAgent::Implement => LABEL_AGENT_IMPLEMENT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    UnsupportedObjectKind,
    ActionFiltered { action: String },
    UserNotAllowed { username: String },
    ProjectNotConfigured,
    Duplicate,
    LabelNotTriggered,
}

impl SkipReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkipReason::UnsupportedObjectKind => "unsupported_object_kind",
            SkipReason::ActionFiltered { .. } => "action_filtered",
            SkipReason::UserNotAllowed { .. } => "user_not_allowed",
            SkipReason::ProjectNotConfigured => "project_not_configured",
            SkipReason::Duplicate => "duplicate",
            SkipReason::LabelNotTriggered => "label_not_triggered",
        }
    }
}

fn has_label(labels: &[Label], target: &str) -> bool {
    labels.iter().any(|label| label.title == target)
}

fn label_newly_added(previous: &[Label], current: &[Label], target: &str) -> bool {
    !has_label(previous, target) && has_label(current, target)
}

/// Returns `Ok(())` when the payload should be forwarded to Cursor.
pub fn should_forward(
    payload: &GitLabMrWebhook,
    allowed_users: &HashSet<String>,
) -> Result<(), SkipReason> {
    if payload.object_kind != "merge_request" {
        return Err(SkipReason::UnsupportedObjectKind);
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

    if !allowed_users.contains(&payload.user.username) {
        return Err(SkipReason::UserNotAllowed {
            username: payload.user.username.clone(),
        });
    }

    Ok(())
}

/// Returns the issue agents that should receive a Cursor webhook.
///
/// Forwards when `agent:verify` or `agent:implement` is present on a newly opened issue,
/// or when either label was just added on an `update` (`previous` lacks it, `current` has it).
pub fn should_forward_issue(
    payload: &GitLabIssueWebhook,
    allowed_users: &HashSet<String>,
) -> Result<Vec<IssueAgent>, SkipReason> {
    if payload.object_kind != "issue" {
        return Err(SkipReason::UnsupportedObjectKind);
    }

    if !allowed_users.contains(&payload.user.username) {
        return Err(SkipReason::UserNotAllowed {
            username: payload.user.username.clone(),
        });
    }

    let mut agents = Vec::new();
    match payload.object_attributes.action.as_str() {
        "open" => {
            if has_label(&payload.labels, LABEL_AGENT_VERIFY) {
                agents.push(IssueAgent::Verify);
            }
            if has_label(&payload.labels, LABEL_AGENT_IMPLEMENT) {
                agents.push(IssueAgent::Implement);
            }
        }
        "update" => {
            if let Some(label_change) = payload
                .changes
                .as_ref()
                .and_then(|changes| changes.labels.as_ref())
            {
                if label_newly_added(
                    &label_change.previous,
                    &label_change.current,
                    LABEL_AGENT_VERIFY,
                ) {
                    agents.push(IssueAgent::Verify);
                }
                if label_newly_added(
                    &label_change.previous,
                    &label_change.current,
                    LABEL_AGENT_IMPLEMENT,
                ) {
                    agents.push(IssueAgent::Implement);
                }
            }
        }
        _ => {}
    }

    if agents.is_empty() {
        return Err(SkipReason::LabelNotTriggered);
    }

    Ok(agents)
}

/// When both agents would fire on the same issue event, pick one so they never run together.
/// Implement takes priority over verify (typical implement-then-verify workflow).
pub fn select_issue_agent(agents: &[IssueAgent]) -> Option<IssueAgent> {
    if agents.contains(&IssueAgent::Implement) {
        Some(IssueAgent::Implement)
    } else if agents.contains(&IssueAgent::Verify) {
        Some(IssueAgent::Verify)
    } else {
        None
    }
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
    fn fork_mr_forwards() {
        let mut payload = base_payload("open");
        payload.object_attributes.source_project_id = Some(2);
        payload.object_attributes.target_project_id = Some(1);
        assert!(should_forward(&payload, &allowlist()).is_ok());
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
        assert_eq!(err, SkipReason::UnsupportedObjectKind);
    }

    fn base_issue(action: &str) -> GitLabIssueWebhook {
        GitLabIssueWebhook {
            object_kind: "issue".to_string(),
            user: User {
                username: "plasticdigits".to_string(),
            },
            project: Project {
                id: 1,
                name: "test-project".to_string(),
                path_with_namespace: Some("group/test-project".to_string()),
            },
            labels: Vec::new(),
            changes: None,
            object_attributes: IssueObjectAttributes {
                action: action.to_string(),
                iid: 12,
                title: "Test issue".to_string(),
                description: Some("Details".to_string()),
                url: Some("https://gitlab.example/group/test-project/-/issues/12".to_string()),
            },
        }
    }

    #[test]
    fn issue_open_with_verify_label_forwards() {
        let mut payload = base_issue("open");
        payload.labels = vec![Label {
            title: LABEL_AGENT_VERIFY.to_string(),
        }];
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec![IssueAgent::Verify]
        );
    }

    #[test]
    fn issue_open_with_implement_label_forwards() {
        let mut payload = base_issue("open");
        payload.labels = vec![Label {
            title: LABEL_AGENT_IMPLEMENT.to_string(),
        }];
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec![IssueAgent::Implement]
        );
    }

    #[test]
    fn issue_open_with_both_labels_selects_implement() {
        let mut payload = base_issue("open");
        payload.labels = vec![
            Label {
                title: LABEL_AGENT_VERIFY.to_string(),
            },
            Label {
                title: LABEL_AGENT_IMPLEMENT.to_string(),
            },
        ];
        let agents = should_forward_issue(&payload, &allowlist()).unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(select_issue_agent(&agents), Some(IssueAgent::Implement));
    }

    #[test]
    fn issue_open_without_agent_labels_skips() {
        let payload = base_issue("open");
        let err = should_forward_issue(&payload, &allowlist()).unwrap_err();
        assert_eq!(err, SkipReason::LabelNotTriggered);
    }

    #[test]
    fn issue_update_with_label_added_forwards() {
        let mut payload = base_issue("update");
        payload.changes = Some(IssueChanges {
            labels: Some(LabelChange {
                previous: vec![],
                current: vec![Label {
                    title: LABEL_AGENT_VERIFY.to_string(),
                }],
            }),
        });
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec![IssueAgent::Verify]
        );
    }

    #[test]
    fn issue_update_with_label_already_present_skips() {
        let mut payload = base_issue("update");
        payload.changes = Some(IssueChanges {
            labels: Some(LabelChange {
                previous: vec![Label {
                    title: LABEL_AGENT_VERIFY.to_string(),
                }],
                current: vec![Label {
                    title: LABEL_AGENT_VERIFY.to_string(),
                }],
            }),
        });
        let err = should_forward_issue(&payload, &allowlist()).unwrap_err();
        assert_eq!(err, SkipReason::LabelNotTriggered);
    }

    #[test]
    fn issue_update_without_label_change_skips() {
        let mut payload = base_issue("update");
        payload.changes = Some(IssueChanges { labels: None });
        let err = should_forward_issue(&payload, &allowlist()).unwrap_err();
        assert_eq!(err, SkipReason::LabelNotTriggered);
    }

    #[test]
    fn issue_allowlist_denies_unknown_user() {
        let mut payload = base_issue("open");
        payload.labels = vec![Label {
            title: LABEL_AGENT_VERIFY.to_string(),
        }];
        payload.user.username = "stranger".to_string();
        let err = should_forward_issue(&payload, &allowlist()).unwrap_err();
        assert_eq!(
            err,
            SkipReason::UserNotAllowed {
                username: "stranger".to_string()
            }
        );
    }
}
