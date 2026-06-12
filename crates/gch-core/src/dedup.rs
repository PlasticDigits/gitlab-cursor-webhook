// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::filter::{GitLabMrWebhook, Project};

/// Tracks forwarded keys so duplicate GitLab deliveries are skipped.
#[derive(Debug)]
pub struct DedupCache {
    ttl: Duration,
    seen: Mutex<HashMap<String, Instant>>,
}

impl DedupCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            ttl: Duration::from_secs(ttl_secs.max(1)),
            seen: Mutex::new(HashMap::new()),
        }
    }

    /// Returns `true` if this MR commit was already forwarded within the TTL window.
    pub fn is_duplicate(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
        seen.retain(|_, at| now.duration_since(*at) < self.ttl);

        if seen.contains_key(key) {
            return true;
        }
        seen.insert(key.to_string(), now);
        false
    }
}

/// Per-issue, per-tag flow key — implement and verify dedupe independently.
pub fn issue_key(project: &Project, iid: u64, tag: &str) -> String {
    format!("{}:{}:{}", project.id, iid, tag)
}

/// `"{project_id}:{iid}:{commit_sha}:{tag}"` when `last_commit.id` is present.
pub fn commit_key(payload: &GitLabMrWebhook, tag: &str) -> Option<String> {
    let commit = payload
        .object_attributes
        .last_commit
        .as_ref()?
        .id
        .trim();
    if commit.is_empty() {
        return None;
    }
    Some(format!(
        "{}:{}:{}:{}",
        payload.project.id, payload.object_attributes.iid, commit, tag
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{GitLabMrWebhook, LastCommit, ObjectAttributes, Project, User};
    use std::thread;
    use std::time::Duration as StdDuration;

    fn sample(iid: u64, commit: &str) -> GitLabMrWebhook {
        GitLabMrWebhook {
            object_kind: "merge_request".to_string(),
            user: User {
                username: "alice".to_string(),
            },
            project: Project {
                id: 1,
                name: "p".to_string(),
                path_with_namespace: None,
            },
            labels: Vec::new(),
            changes: None,
            object_attributes: ObjectAttributes {
                action: "open".to_string(),
                oldrev: None,
                iid,
                title: "t".to_string(),
                description: None,
                url: "https://example/mr/34".to_string(),
                merge_commit_sha: None,
                source_project_id: None,
                target_project_id: None,
                last_commit: Some(LastCommit {
                    id: commit.to_string(),
                    message: None,
                    timestamp: None,
                    url: None,
                }),
            },
        }
    }

    #[test]
    fn same_commit_on_same_mr_is_duplicate() {
        let cache = DedupCache::new(3600);
        let key = commit_key(&sample(34, "abc123"), "security").unwrap();
        assert!(!cache.is_duplicate(&key));
        assert!(cache.is_duplicate(&key));
    }

    #[test]
    fn same_commit_different_mr_is_not_duplicate() {
        let cache = DedupCache::new(3600);
        let a = commit_key(&sample(34, "abc123"), "security").unwrap();
        let b = commit_key(&sample(35, "abc123"), "security").unwrap();
        assert!(!cache.is_duplicate(&a));
        assert!(!cache.is_duplicate(&b));
    }

    #[test]
    fn new_commit_on_same_mr_is_not_duplicate() {
        let cache = DedupCache::new(3600);
        let first = commit_key(&sample(34, "abc123"), "security").unwrap();
        let second = commit_key(&sample(34, "def456"), "security").unwrap();
        assert!(!cache.is_duplicate(&first));
        assert!(!cache.is_duplicate(&second));
    }

    #[test]
    fn same_commit_different_mr_tags_are_not_duplicate() {
        let cache = DedupCache::new(3600);
        let security = commit_key(&sample(34, "abc123"), "security").unwrap();
        let other = commit_key(&sample(34, "abc123"), "review").unwrap();
        assert!(!cache.is_duplicate(&security));
        assert!(!cache.is_duplicate(&other));
    }

    #[test]
    fn entry_expires_after_ttl() {
        let cache = DedupCache::new(1);
        let key = commit_key(&sample(34, "abc123"), "security").unwrap();
        assert!(!cache.is_duplicate(&key));
        assert!(cache.is_duplicate(&key));
        thread::sleep(StdDuration::from_millis(1100));
        assert!(!cache.is_duplicate(&key));
    }

    #[test]
    fn issue_key_includes_tag() {
        let project = Project {
            id: 42,
            name: "p".to_string(),
            path_with_namespace: None,
        };
        assert_eq!(issue_key(&project, 7, "verify"), "42:7:verify");
    }

    #[test]
    fn issue_dedup_is_per_tag() {
        let cache = DedupCache::new(900);
        let project = Project {
            id: 1,
            name: "p".to_string(),
            path_with_namespace: None,
        };
        let implement = issue_key(&project, 12, "implement");
        let verify = issue_key(&project, 12, "verify");
        assert!(!cache.is_duplicate(&implement));
        assert!(cache.is_duplicate(&implement));
        assert!(!cache.is_duplicate(&verify));
    }
}
