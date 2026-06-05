// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::{HashMap, HashSet};
use std::env;
use std::net::SocketAddr;

use thiserror::Error;

use crate::filter::Project;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCursorConfig {
    pub webhook_url: String,
    pub token: String,
}

/// Which Cursor automation receives a forwarded webhook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookAgent {
    /// MR security review automation (`PROJECT_WEBHOOKS_SECURITY`).
    Security,
    /// Issue verify automation (`PROJECT_WEBHOOKS_VERIFY`, label `agent:verify`).
    Verify,
    /// Issue implement automation (`PROJECT_WEBHOOKS_IMPLEMENT`, label `agent:implement`).
    Implement,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub gitlab_webhook_secret: Option<String>,
    pub allowed_users: HashSet<String>,
    /// MR security review: Cursor webhook URL + token keyed by `path_with_namespace` or numeric `id`.
    pub project_webhooks_security: HashMap<String, ProjectCursorConfig>,
    /// Issue verify (`agent:verify`): same key format as security.
    pub project_webhooks_verify: HashMap<String, ProjectCursorConfig>,
    /// Issue implement (`agent:implement`): same key format as security.
    pub project_webhooks_implement: HashMap<String, ProjectCursorConfig>,
    /// How long to remember forwarded MR commit keys (bounds in-memory dedup cache).
    pub dedup_ttl_secs: u64,
    /// How long to remember forwarded issue keys; verify and implement share one key per issue.
    pub issue_dedup_ttl_secs: u64,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVar(&'static str),
    #[error("invalid PORT value: {0}")]
    InvalidPort(String),
    #[error("invalid project webhooks entry (expected path-or-id=url|crsr_token): {0}")]
    InvalidProjectWebhooks(String),
}

fn normalize_cursor_token(token: &str) -> Result<String, ConfigError> {
    let token = token.trim();
    let token = token
        .strip_prefix("Bearer ")
        .or_else(|| token.strip_prefix("bearer "))
        .unwrap_or(token)
        .trim();
    if token.is_empty() {
        return Err(ConfigError::InvalidProjectWebhooks(
            "cursor token must not be empty".to_string(),
        ));
    }
    Ok(token.to_string())
}

fn parse_project_webhooks(raw: &str) -> Result<HashMap<String, ProjectCursorConfig>, ConfigError> {
    let mut map = HashMap::new();
    for entry in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let (key, value) = entry
            .split_once('=')
            .ok_or_else(|| ConfigError::InvalidProjectWebhooks(entry.to_string()))?;
        let key = key.trim();
        let (url, token) = value
            .split_once('|')
            .ok_or_else(|| ConfigError::InvalidProjectWebhooks(entry.to_string()))?;
        let url = url.trim();
        if key.is_empty() || url.is_empty() {
            return Err(ConfigError::InvalidProjectWebhooks(entry.to_string()));
        }
        map.insert(
            key.to_string(),
            ProjectCursorConfig {
                webhook_url: url.to_string(),
                token: normalize_cursor_token(token)?,
            },
        );
    }
    Ok(map)
}

impl Config {
    /// Resolve the Cursor webhook config for a GitLab project and agent.
    ///
    /// Checks the agent's map by `path_with_namespace`, then numeric `id`.
    pub fn cursor_config_for(
        &self,
        project: &Project,
        agent: WebhookAgent,
    ) -> Option<&ProjectCursorConfig> {
        let map = match agent {
            WebhookAgent::Security => &self.project_webhooks_security,
            WebhookAgent::Verify => &self.project_webhooks_verify,
            WebhookAgent::Implement => &self.project_webhooks_implement,
        };
        if let Some(path) = project.path_with_namespace.as_deref() {
            if let Some(config) = map.get(path) {
                return Some(config);
            }
        }
        map.get(&project.id.to_string())
    }
}

fn parse_optional_project_webhooks(
    var: &'static str,
) -> Result<HashMap<String, ProjectCursorConfig>, ConfigError> {
    match env::var(var) {
        Ok(raw) => parse_project_webhooks(raw.trim()),
        Err(_) => Ok(HashMap::new()),
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let port: u16 = port
            .parse()
            .map_err(|_| ConfigError::InvalidPort(port.clone()))?;
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], port));

        let gitlab_webhook_secret = env::var("GITLAB_WEBHOOK_SECRET")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let allowed_users_raw = env::var("ALLOWED_USERS").unwrap_or_default();
        let allowed_users: HashSet<String> = allowed_users_raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();

        if allowed_users.is_empty() {
            eprintln!(
                "CRITICAL SECURITY: ALLOWED_USERS is not set or empty. \
                 Refusing to start — allowing any GitLab user would forward MR webhooks to Cursor. \
                 Set ALLOWED_USERS to a comma-separated list of GitLab usernames (e.g. plasticdigits,brouie)."
            );
            std::process::exit(1);
        }

        let dedup_ttl_secs = env::var("DEDUP_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(86_400);

        let issue_dedup_ttl_secs = env::var("ISSUE_DEDUP_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(900);

        let project_webhooks_security_raw = env::var("PROJECT_WEBHOOKS_SECURITY")
            .map_err(|_| ConfigError::MissingVar("PROJECT_WEBHOOKS_SECURITY"))?;
        let project_webhooks_security =
            parse_project_webhooks(project_webhooks_security_raw.trim())?;
        if project_webhooks_security.is_empty() {
            return Err(ConfigError::MissingVar("PROJECT_WEBHOOKS_SECURITY"));
        }

        let project_webhooks_verify = parse_optional_project_webhooks("PROJECT_WEBHOOKS_VERIFY")?;
        let project_webhooks_implement =
            parse_optional_project_webhooks("PROJECT_WEBHOOKS_IMPLEMENT")?;

        Ok(Self {
            listen_addr,
            gitlab_webhook_secret,
            allowed_users,
            project_webhooks_security,
            project_webhooks_verify,
            project_webhooks_implement,
            dedup_ttl_secs,
            issue_dedup_ttl_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
        let _guard: MutexGuard<'_, ()> = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let keys: Vec<&str> = vars.iter().map(|(k, _)| *k).collect();
        let saved: Vec<(String, Option<String>)> = keys
            .iter()
            .map(|k| (k.to_string(), env::var(k).ok()))
            .collect();

        for (key, value) in vars {
            match value {
                Some(v) => unsafe { env::set_var(key, v) },
                None => unsafe { env::remove_var(key) },
            }
        }

        f();

        for (key, value) in saved {
            match value {
                Some(v) => unsafe { env::set_var(&key, v) },
                None => unsafe { env::remove_var(&key) },
            }
        }
    }

    #[test]
    fn loads_required_vars_and_parses_allowlist() {
        with_env(
            &[
                ("PORT", Some("9090")),
                (
                    "PROJECT_WEBHOOKS_SECURITY",
                    Some(
                        "plasticdigits/yieldomega=https://cursor.example/yieldomega|crsr_yieldomega",
                    ),
                ),
                ("ALLOWED_USERS", Some("alice, bob ,charlie")),
                ("GITLAB_WEBHOOK_SECRET", Some("secret")),
            ],
            || {
                let cfg = Config::from_env().expect("config should load");
                assert_eq!(cfg.listen_addr.port(), 9090);
                assert_eq!(
                    cfg.project_webhooks_security.get("plasticdigits/yieldomega"),
                    Some(&ProjectCursorConfig {
                        webhook_url: "https://cursor.example/yieldomega".to_string(),
                        token: "crsr_yieldomega".to_string(),
                    })
                );
                assert!(cfg.project_webhooks_verify.is_empty());
                assert!(cfg.project_webhooks_implement.is_empty());
                assert_eq!(cfg.gitlab_webhook_secret.as_deref(), Some("secret"));
                assert!(cfg.allowed_users.contains("alice"));
                assert!(cfg.allowed_users.contains("bob"));
                assert!(cfg.allowed_users.contains("charlie"));
            },
        );
    }

    #[test]
    fn parses_project_webhooks() {
        with_env(
            &[
                ("ALLOWED_USERS", Some("alice")),
                (
                    "PROJECT_WEBHOOKS_SECURITY",
                    Some(
                        "plasticdigits/yieldomega=https://cursor.example/yieldomega|crsr_a,plasticdigits/cl8y-dex-terraclassic=https://cursor.example/cl8y|crsr_b,123=https://cursor.example/by-id|crsr_c",
                    ),
                ),
            ],
            || {
                let cfg = Config::from_env().expect("config should load");
                assert_eq!(
                    cfg.project_webhooks_security.get("plasticdigits/yieldomega"),
                    Some(&ProjectCursorConfig {
                        webhook_url: "https://cursor.example/yieldomega".to_string(),
                        token: "crsr_a".to_string(),
                    })
                );
                assert_eq!(
                    cfg.project_webhooks_security
                        .get("plasticdigits/cl8y-dex-terraclassic"),
                    Some(&ProjectCursorConfig {
                        webhook_url: "https://cursor.example/cl8y".to_string(),
                        token: "crsr_b".to_string(),
                    })
                );
                assert_eq!(
                    cfg.project_webhooks_security.get("123"),
                    Some(&ProjectCursorConfig {
                        webhook_url: "https://cursor.example/by-id".to_string(),
                        token: "crsr_c".to_string(),
                    })
                );
            },
        );
    }

    #[test]
    fn parses_verify_and_implement_webhooks() {
        with_env(
            &[
                ("ALLOWED_USERS", Some("alice")),
                (
                    "PROJECT_WEBHOOKS_SECURITY",
                    Some("plasticdigits/yieldomega=https://cursor.example/security|crsr_sec"),
                ),
                (
                    "PROJECT_WEBHOOKS_VERIFY",
                    Some("plasticdigits/yieldomega=https://cursor.example/verify|crsr_verify"),
                ),
                (
                    "PROJECT_WEBHOOKS_IMPLEMENT",
                    Some("plasticdigits/yieldomega=https://cursor.example/implement|crsr_impl"),
                ),
            ],
            || {
                let cfg = Config::from_env().expect("config should load");
                assert_eq!(
                    cfg.project_webhooks_verify.get("plasticdigits/yieldomega"),
                    Some(&ProjectCursorConfig {
                        webhook_url: "https://cursor.example/verify".to_string(),
                        token: "crsr_verify".to_string(),
                    })
                );
                assert_eq!(
                    cfg.project_webhooks_implement.get("plasticdigits/yieldomega"),
                    Some(&ProjectCursorConfig {
                        webhook_url: "https://cursor.example/implement".to_string(),
                        token: "crsr_impl".to_string(),
                    })
                );
            },
        );
    }

    #[test]
    fn strips_bearer_prefix_from_token() {
        with_env(
            &[
                ("ALLOWED_USERS", Some("alice")),
                (
                    "PROJECT_WEBHOOKS_SECURITY",
                    Some("plasticdigits/yieldomega=https://cursor.example/yieldomega|Bearer crsr_x"),
                ),
            ],
            || {
                let cfg = Config::from_env().expect("config should load");
                assert_eq!(
                    cfg.project_webhooks_security
                        .get("plasticdigits/yieldomega")
                        .unwrap()
                        .token,
                    "crsr_x"
                );
            },
        );
    }

    #[test]
    fn resolves_project_config_by_path_then_id() {
        let cfg = Config {
            listen_addr: "127.0.0.1:8080".parse().unwrap(),
            gitlab_webhook_secret: None,
            allowed_users: HashSet::new(),
            project_webhooks_security: [
                (
                    "plasticdigits/yieldomega".to_string(),
                    ProjectCursorConfig {
                        webhook_url: "https://cursor.example/yieldomega".to_string(),
                        token: "crsr_a".to_string(),
                    },
                ),
                (
                    "999".to_string(),
                    ProjectCursorConfig {
                        webhook_url: "https://cursor.example/by-id".to_string(),
                        token: "crsr_b".to_string(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
            project_webhooks_verify: HashMap::new(),
            project_webhooks_implement: HashMap::new(),
            dedup_ttl_secs: 86_400,
            issue_dedup_ttl_secs: 900,
        };

        let by_path = Project {
            id: 1,
            name: "yieldomega".to_string(),
            path_with_namespace: Some("plasticdigits/yieldomega".to_string()),
        };
        assert_eq!(
            cfg.cursor_config_for(&by_path, WebhookAgent::Security),
            Some(&ProjectCursorConfig {
                webhook_url: "https://cursor.example/yieldomega".to_string(),
                token: "crsr_a".to_string(),
            })
        );

        let by_id = Project {
            id: 999,
            name: "other".to_string(),
            path_with_namespace: Some("plasticdigits/other".to_string()),
        };
        assert_eq!(
            cfg.cursor_config_for(&by_id, WebhookAgent::Security)
                .map(|c| c.token.as_str()),
            Some("crsr_b")
        );

        let unconfigured = Project {
            id: 2,
            name: "unknown".to_string(),
            path_with_namespace: Some("plasticdigits/unknown".to_string()),
        };
        assert_eq!(
            cfg.cursor_config_for(&unconfigured, WebhookAgent::Security),
            None
        );
    }

    #[test]
    fn missing_project_webhooks_security_errors() {
        with_env(
            &[
                ("PROJECT_WEBHOOKS_SECURITY", None),
                ("ALLOWED_USERS", Some("alice")),
            ],
            || {
                let err = Config::from_env().unwrap_err();
                assert!(matches!(
                    err,
                    ConfigError::MissingVar("PROJECT_WEBHOOKS_SECURITY")
                ));
            },
        );
    }
}
