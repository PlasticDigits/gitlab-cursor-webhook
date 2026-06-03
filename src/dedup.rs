// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::filter::GitLabMrWebhook;

/// Tracks MR commits already forwarded so duplicate GitLab deliveries are skipped.
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

/// `"{iid}:{commit_sha}"` when `last_commit.id` is present.
pub fn commit_key(payload: &GitLabMrWebhook) -> Option<String> {
    let commit = payload
        .object_attributes
        .last_commit
        .as_ref()?
        .id
        .trim();
    if commit.is_empty() {
        return None;
    }
    Some(format!("{}:{}", payload.object_attributes.iid, commit))
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
                name: "p".to_string(),
            },
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
        let key = commit_key(&sample(34, "abc123")).unwrap();
        assert!(!cache.is_duplicate(&key));
        assert!(cache.is_duplicate(&key));
    }

    #[test]
    fn same_commit_different_mr_is_not_duplicate() {
        let cache = DedupCache::new(3600);
        let a = commit_key(&sample(34, "abc123")).unwrap();
        let b = commit_key(&sample(35, "abc123")).unwrap();
        assert!(!cache.is_duplicate(&a));
        assert!(!cache.is_duplicate(&b));
    }

    #[test]
    fn new_commit_on_same_mr_is_not_duplicate() {
        let cache = DedupCache::new(3600);
        let first = commit_key(&sample(34, "abc123")).unwrap();
        let second = commit_key(&sample(34, "def456")).unwrap();
        assert!(!cache.is_duplicate(&first));
        assert!(!cache.is_duplicate(&second));
    }

    #[test]
    fn entry_expires_after_ttl() {
        let cache = DedupCache::new(1);
        let key = commit_key(&sample(34, "abc123")).unwrap();
        assert!(!cache.is_duplicate(&key));
        assert!(cache.is_duplicate(&key));
        thread::sleep(StdDuration::from_millis(1100));
        assert!(!cache.is_duplicate(&key));
    }
}
