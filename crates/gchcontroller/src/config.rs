// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use gch_core::db::{Database, Settings};
use thiserror::Error;

pub struct ControllerConfig {
    pub listen_addr: SocketAddr,
    pub allowed_users: HashSet<String>,
    pub dedup_ttl_secs: u64,
    pub issue_dedup_ttl_secs: u64,
    pub db_path: PathBuf,
    pub hcloud_token: String,
    pub firewall_id: String,
    pub ssh_key_ids: Vec<String>,
    pub controller_url: String,
    pub cursor_api_key: String,
    pub gitlab_token: String,
    pub jobs_dir: PathBuf,
    pub terraform_module_dir: PathBuf,
    pub cloud_init_template: PathBuf,
    pub provision_enabled: bool,
    pub admin_token: Option<String>,
    pub settings: Settings,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVar(&'static str),
    #[error("invalid PORT value: {0}")]
    InvalidPort(String),
    #[error("invalid LISTEN_ADDR value: {0}")]
    InvalidListenAddr(String),
    #[error("database error: {0}")]
    Database(#[from] gch_core::db::DbError),
}

impl ControllerConfig {
    pub fn from_env() -> Result<(Self, Database), ConfigError> {
        let listen_addr = match env::var("LISTEN_ADDR") {
            Ok(addr) if !addr.trim().is_empty() => addr
                .trim()
                .parse()
                .map_err(|_| ConfigError::InvalidListenAddr(addr))?,
            _ => {
                let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
                let port: u16 = port
                    .parse()
                    .map_err(|_| ConfigError::InvalidPort(port.clone()))?;
                SocketAddr::from(([0, 0, 0, 0], port))
            }
        };

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
                 Refusing to start."
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

        let db_path = PathBuf::from(
            env::var("GCH_DB_PATH").unwrap_or_else(|_| "/var/lib/gch/gch.db".to_string()),
        );
        let db = Database::open(&db_path)?;
        let mut settings = db.get_settings()?;

        if let Ok(url) = env::var("GCH_CONTROLLER_URL") {
            if !url.is_empty() {
                settings.controller_url = url;
            }
        }
        if let Ok(fw) = env::var("GCH_FIREWALL_ID") {
            if !fw.is_empty() {
                settings.firewall_id = fw;
            }
        }

        let hcloud_token = env::var("HCLOUD_TOKEN")
            .map_err(|_| ConfigError::MissingVar("HCLOUD_TOKEN"))?;

        let firewall_id = if settings.firewall_id.is_empty() {
            return Err(ConfigError::MissingVar("GCH_FIREWALL_ID"));
        } else {
            settings.firewall_id.clone()
        };

        let controller_url = if settings.controller_url.is_empty() {
            return Err(ConfigError::MissingVar("GCH_CONTROLLER_URL"));
        } else {
            settings.controller_url.clone()
        };

        let cursor_api_key = env::var("CURSOR_API_KEY")
            .map_err(|_| ConfigError::MissingVar("CURSOR_API_KEY"))?;

        let gitlab_token = env::var("GITLAB_TOKEN")
            .map_err(|_| ConfigError::MissingVar("GITLAB_TOKEN"))?;

        let ssh_key_ids: Vec<String> = env::var("GCH_SSH_KEY_IDS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();

        let jobs_dir = PathBuf::from(
            env::var("GCH_JOBS_DIR").unwrap_or_else(|_| "/var/lib/gch/jobs".to_string()),
        );

        let terraform_module_dir = PathBuf::from(
            env::var("GCH_TERRAFORM_MODULE")
                .unwrap_or_else(|_| "terraform/modules/agent-vm".to_string()),
        );

        let cloud_init_template = PathBuf::from(
            env::var("GCH_CLOUD_INIT_TEMPLATE")
                .unwrap_or_else(|_| "templates/cloud_init.yaml.tpl".to_string()),
        );

        let provision_enabled = env::var("GCH_PROVISION_ENABLED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        let admin_token = env::var("GCH_ADMIN_TOKEN")
            .ok()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());

        Ok((
            Self {
                listen_addr,
                allowed_users,
                dedup_ttl_secs,
                issue_dedup_ttl_secs,
                db_path,
                hcloud_token,
                firewall_id,
                ssh_key_ids,
                controller_url,
                cursor_api_key,
                gitlab_token,
                jobs_dir,
                terraform_module_dir,
                cloud_init_template,
                provision_enabled,
                admin_token,
                settings,
            },
            db,
        ))
    }
}
