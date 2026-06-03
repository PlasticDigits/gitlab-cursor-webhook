// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;
use std::env;
use std::net::SocketAddr;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub cursor_webhook_url: String,
    pub cursor_token: String,
    pub gitlab_webhook_secret: Option<String>,
    pub allowed_users: HashSet<String>,
    /// How long to remember forwarded commit keys (bounds in-memory dedup cache).
    pub dedup_ttl_secs: u64,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVar(&'static str),
    #[error("invalid PORT value: {0}")]
    InvalidPort(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let port: u16 = port
            .parse()
            .map_err(|_| ConfigError::InvalidPort(port.clone()))?;
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], port));

        let cursor_webhook_url = env::var("CURSOR_WEBHOOK_URL")
            .map_err(|_| ConfigError::MissingVar("CURSOR_WEBHOOK_URL"))?;
        let cursor_token =
            env::var("CURSOR_TOKEN").map_err(|_| ConfigError::MissingVar("CURSOR_TOKEN"))?;

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

        Ok(Self {
            listen_addr,
            cursor_webhook_url,
            cursor_token,
            gitlab_webhook_secret,
            allowed_users,
            dedup_ttl_secs,
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
                ("CURSOR_WEBHOOK_URL", Some("https://cursor.example/hook")),
                ("CURSOR_TOKEN", Some("crsr_test")),
                ("ALLOWED_USERS", Some("alice, bob ,charlie")),
                ("GITLAB_WEBHOOK_SECRET", Some("secret")),
            ],
            || {
                let cfg = Config::from_env().expect("config should load");
                assert_eq!(cfg.listen_addr.port(), 9090);
                assert_eq!(cfg.cursor_webhook_url, "https://cursor.example/hook");
                assert_eq!(cfg.cursor_token, "crsr_test");
                assert_eq!(cfg.gitlab_webhook_secret.as_deref(), Some("secret"));
                assert!(cfg.allowed_users.contains("alice"));
                assert!(cfg.allowed_users.contains("bob"));
                assert!(cfg.allowed_users.contains("charlie"));
            },
        );
    }

    #[test]
    fn missing_cursor_url_errors() {
        with_env(
            &[
                ("CURSOR_WEBHOOK_URL", None),
                ("CURSOR_TOKEN", Some("crsr_test")),
                ("ALLOWED_USERS", Some("alice")),
            ],
            || {
                let err = Config::from_env().unwrap_err();
                assert!(matches!(err, ConfigError::MissingVar("CURSOR_WEBHOOK_URL")));
            },
        );
    }
}
