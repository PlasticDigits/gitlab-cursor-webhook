// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::tag::{is_issue_label_reserved, tag_from_agent_label, LABEL_AGENT_PREFIX};

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
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub changes: Option<MrChanges>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct MrChanges {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    UnsupportedObjectKind,
    ActionFiltered { action: String },
    UserNotAllowed { username: String },
    ProjectNotConfigured,
    Duplicate,
    LabelNotTriggered,
    LabelReserved { tag: String },
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
            SkipReason::LabelReserved { .. } => "label_reserved",
        }
    }
}

pub fn user_is_allowed(allowed_users: &HashSet<String>, username: &str) -> bool {
    allowed_users.contains(username)
        || allowed_users
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(username))
}

fn has_label(labels: &[Label], target: &str) -> bool {
    labels.iter().any(|label| label.title == target)
}

fn label_newly_added(previous: &[Label], current: &[Label], target: &str) -> bool {
    !has_label(previous, target) && has_label(current, target)
}

/// Agent tag names removed on an issue `update` (for clearing per-issue dedup).
pub fn removed_agent_label_tags(payload: &GitLabIssueWebhook) -> Vec<String> {
    if payload.object_attributes.action != "update" {
        return Vec::new();
    }
    let Some(label_change) = payload
        .changes
        .as_ref()
        .and_then(|changes| changes.labels.as_ref())
    else {
        return Vec::new();
    };

    let mut tags = Vec::new();
    for label in &label_change.previous {
        let Some(tag) = tag_from_agent_label(&label.title) else {
            continue;
        };
        if !has_label(&label_change.current, &label.title) && !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags
}

fn collect_triggered_agent_labels(labels: &[Label]) -> (Vec<String>, bool) {
    let mut tags = Vec::new();
    let mut saw_reserved = false;

    for label in labels {
        if let Some(tag) = tag_from_agent_label(&label.title) {
            if !tags.contains(&tag) {
                tags.push(tag);
            }
            continue;
        }
        if let Some(suffix) = label.title.strip_prefix(LABEL_AGENT_PREFIX) {
            if is_issue_label_reserved(suffix) {
                saw_reserved = true;
            }
        }
    }

    (tags, saw_reserved)
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

    if !user_is_allowed(allowed_users, &payload.user.username) {
        return Err(SkipReason::UserNotAllowed {
            username: payload.user.username.clone(),
        });
    }

    Ok(())
}

fn mr_label_change(payload: &GitLabMrWebhook) -> Option<&LabelChange> {
    payload
        .changes
        .as_ref()
        .and_then(|changes| changes.labels.as_ref())
}

/// Label-only MR update (no new commits) — security review must not run.
pub fn is_mr_label_only_update(payload: &GitLabMrWebhook) -> bool {
    payload.object_attributes.action == "update"
        && payload
            .object_attributes
            .oldrev
            .as_ref()
            .is_none_or(|rev| rev.is_empty())
        && mr_label_change(payload).is_some()
}

/// Returns MR tag names from `agent:{tag}` labels (e.g. `agent:fix_conflicts`).
///
/// Forwards on `open` when the label is present, or on `update` when the label was just added.
/// Reserved tags (e.g. `agent:security`) are ignored — use the default MR security flow instead.
pub fn should_forward_mr_labels(
    payload: &GitLabMrWebhook,
    allowed_users: &HashSet<String>,
) -> Result<Vec<String>, SkipReason> {
    if payload.object_kind != "merge_request" {
        return Err(SkipReason::UnsupportedObjectKind);
    }

    if !user_is_allowed(allowed_users, &payload.user.username) {
        return Err(SkipReason::UserNotAllowed {
            username: payload.user.username.clone(),
        });
    }

    let (mut tags, saw_reserved) = match payload.object_attributes.action.as_str() {
        "open" => collect_triggered_agent_labels(&payload.labels),
        "update" => {
            if let Some(label_change) = mr_label_change(payload) {
                let added: Vec<Label> = label_change
                    .current
                    .iter()
                    .filter(|label| {
                        label.title.starts_with(LABEL_AGENT_PREFIX)
                            && label_newly_added(
                                &label_change.previous,
                                &label_change.current,
                                &label.title,
                            )
                    })
                    .cloned()
                    .collect();
                collect_triggered_agent_labels(&added)
            } else {
                (Vec::new(), false)
            }
        }
        _ => (Vec::new(), false),
    };

    tags.sort();
    tags.dedup();

    if tags.is_empty() {
        if saw_reserved {
            return Err(SkipReason::LabelReserved {
                tag: crate::tag::MR_SECURITY_TAG.to_string(),
            });
        }
        return Err(SkipReason::LabelNotTriggered);
    }

    Ok(tags)
}

/// Returns issue tag names that should receive a Cursor webhook.
///
/// Forwards when an `agent:{tag}` label is present on a newly opened issue,
/// or when such a label was just added on an `update` (`previous` lacks it, `current` has it).
/// Reserved tags (e.g. `agent:security`) are ignored; `security` is MR-only.
pub fn should_forward_issue(
    payload: &GitLabIssueWebhook,
    allowed_users: &HashSet<String>,
) -> Result<Vec<String>, SkipReason> {
    if payload.object_kind != "issue" {
        return Err(SkipReason::UnsupportedObjectKind);
    }

    if !user_is_allowed(allowed_users, &payload.user.username) {
        return Err(SkipReason::UserNotAllowed {
            username: payload.user.username.clone(),
        });
    }

    let (mut tags, saw_reserved) = match payload.object_attributes.action.as_str() {
        "open" => collect_triggered_agent_labels(&payload.labels),
        "update" => {
            if let Some(label_change) = payload
                .changes
                .as_ref()
                .and_then(|changes| changes.labels.as_ref())
            {
                let added: Vec<Label> = label_change
                    .current
                    .iter()
                    .filter(|label| {
                        label.title.starts_with(LABEL_AGENT_PREFIX)
                            && label_newly_added(
                                &label_change.previous,
                                &label_change.current,
                                &label.title,
                            )
                    })
                    .cloned()
                    .collect();
                collect_triggered_agent_labels(&added)
            } else {
                (Vec::new(), false)
            }
        }
        _ => (Vec::new(), false),
    };

    tags.sort();
    tags.dedup();

    if tags.is_empty() {
        if saw_reserved {
            return Err(SkipReason::LabelReserved {
                tag: crate::tag::MR_SECURITY_TAG.to_string(),
            });
        }
        return Err(SkipReason::LabelNotTriggered);
    }

    Ok(tags)
}

/// When multiple tags would fire on the same issue event, pick one so they never run together.
pub fn select_issue_tag(tags: &[String]) -> Option<String> {
    crate::tag::order_issue_tags(tags).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::LABEL_AGENT_PREFIX;

    fn agent_label(tag: &str) -> Label {
        Label {
            title: format!("{LABEL_AGENT_PREFIX}{tag}"),
        }
    }

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
            labels: Vec::new(),
            changes: None,
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
        payload.labels = vec![agent_label("verify")];
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec!["verify".to_string()]
        );
    }

    #[test]
    fn issue_open_with_implement_label_forwards() {
        let mut payload = base_issue("open");
        payload.labels = vec![agent_label("implement")];
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec!["implement".to_string()]
        );
    }

    #[test]
    fn issue_open_with_custom_agent_label_forwards() {
        let mut payload = base_issue("open");
        payload.labels = vec![agent_label("gap_analysis")];
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec!["gap_analysis".to_string()]
        );
    }

    #[test]
    fn issue_open_with_reserved_security_label_skips() {
        let mut payload = base_issue("open");
        payload.labels = vec![agent_label("security")];
        let err = should_forward_issue(&payload, &allowlist()).unwrap_err();
        assert!(matches!(err, SkipReason::LabelReserved { .. }));
    }

    #[test]
    fn issue_open_with_security_and_verify_selects_verify() {
        let mut payload = base_issue("open");
        payload.labels = vec![agent_label("security"), agent_label("verify")];
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec!["verify".to_string()]
        );
    }

    #[test]
    fn issue_open_with_both_labels_selects_implement() {
        let mut payload = base_issue("open");
        payload.labels = vec![agent_label("verify"), agent_label("implement")];
        let tags = should_forward_issue(&payload, &allowlist()).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(select_issue_tag(&tags).as_deref(), Some("implement"));
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
                current: vec![agent_label("verify")],
            }),
        });
        assert_eq!(
            should_forward_issue(&payload, &allowlist()).unwrap(),
            vec!["verify".to_string()]
        );
    }

    #[test]
    fn issue_update_with_label_already_present_skips() {
        let mut payload = base_issue("update");
        payload.changes = Some(IssueChanges {
            labels: Some(LabelChange {
                previous: vec![agent_label("verify")],
                current: vec![agent_label("verify")],
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
    fn removed_agent_label_tags_detects_verify_removal() {
        let mut payload = base_issue("update");
        payload.changes = Some(IssueChanges {
            labels: Some(LabelChange {
                previous: vec![agent_label("verify")],
                current: vec![],
            }),
        });
        assert_eq!(
            removed_agent_label_tags(&payload),
            vec!["verify".to_string()]
        );
    }

    #[test]
    fn issue_allowlist_denies_unknown_user() {
        let mut payload = base_issue("open");
        payload.labels = vec![agent_label("verify")];
        payload.user.username = "stranger".to_string();
        let err = should_forward_issue(&payload, &allowlist()).unwrap_err();
        assert_eq!(
            err,
            SkipReason::UserNotAllowed {
                username: "stranger".to_string()
            }
        );
    }

    #[test]
    fn mr_open_with_fix_conflicts_label_forwards() {
        let mut payload = base_payload("open");
        payload.labels = vec![agent_label("fix_conflicts")];
        assert_eq!(
            should_forward_mr_labels(&payload, &allowlist()).unwrap(),
            vec!["fix_conflicts".to_string()]
        );
    }

    #[test]
    fn mr_update_with_fix_conflicts_label_added_forwards() {
        let mut payload = base_payload("update");
        payload.changes = Some(MrChanges {
            labels: Some(LabelChange {
                previous: vec![],
                current: vec![agent_label("fix_conflicts")],
            }),
        });
        assert_eq!(
            should_forward_mr_labels(&payload, &allowlist()).unwrap(),
            vec!["fix_conflicts".to_string()]
        );
    }

    #[test]
    fn mr_open_without_agent_labels_skips() {
        let payload = base_payload("open");
        let err = should_forward_mr_labels(&payload, &allowlist()).unwrap_err();
        assert_eq!(err, SkipReason::LabelNotTriggered);
    }

    #[test]
    fn mr_label_webhook_deserializes_and_forwards() {
        let body = include_str!("../../../tests/fixtures/mr_update_fix_conflicts_label.json");
        let payload: GitLabMrWebhook = serde_json::from_str(body).expect("deserialize");
        assert!(is_mr_label_only_update(&payload));
        assert_eq!(
            should_forward_mr_labels(&payload, &allowlist()).unwrap(),
            vec!["fix_conflicts".to_string()]
        );
    }

    #[test]
    fn user_is_allowed_case_insensitive() {
        let allowed = allowlist();
        assert!(user_is_allowed(&allowed, "PlasticDigits"));
        assert!(user_is_allowed(&allowed, "PLASTICDIGITS"));
    }
}
