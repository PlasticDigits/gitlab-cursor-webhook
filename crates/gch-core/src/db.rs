// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;

use crate::filter::Project;
use crate::tag::{order_issue_tags, validate_tag_name};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gitlab_path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    workspace_path TEXT NOT NULL DEFAULT '/home/agent/workspace',
    enabled INTEGER NOT NULL DEFAULT 1,
    signing_token TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    hetzner_snapshot_id TEXT NOT NULL,
    server_type TEXT NOT NULL DEFAULT 'cpx32',
    hetzner_location TEXT NOT NULL DEFAULT 'fsn1',
    model TEXT NOT NULL DEFAULT 'composer-2.5',
    enabled INTEGER NOT NULL DEFAULT 1,
    UNIQUE(project_id, name)
);

CREATE TABLE IF NOT EXISTS prompts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_id INTEGER NOT NULL UNIQUE REFERENCES tags(id) ON DELETE CASCADE,
    template TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('job_timeout_secs', '10800');
INSERT OR IGNORE INTO settings (key, value) VALUES ('heartbeat_stale_secs', '300');
INSERT OR IGNORE INTO settings (key, value) VALUES ('provisioning_timeout_secs', '900');
INSERT OR IGNORE INTO settings (key, value) VALUES ('controller_url', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('firewall_id', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('max_concurrent_jobs', '10');
"#;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("tag not found: project={project} tag={tag}")]
    TagNotFound { project: String, tag: String },
    #[error("invalid tag name: {0}")]
    InvalidTag(String),
    #[error("invalid signing token: expected whsec_... base64 key")]
    InvalidSigningToken,
}

const PROJECT_SELECT: &str =
    "SELECT id, gitlab_path, name, workspace_path, enabled, signing_token FROM projects";

fn map_project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRecord> {
    Ok(ProjectRecord {
        id: row.get(0)?,
        gitlab_path: row.get(1)?,
        name: row.get(2)?,
        workspace_path: row.get(3)?,
        enabled: row.get::<_, i64>(4)? != 0,
        signing_token: row.get(5)?,
    })
}

fn migrate(conn: &Connection) -> Result<(), DbError> {
    let has_column = conn
        .prepare("PRAGMA table_info(projects)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(Result::ok)
        .any(|name| name == "signing_token");
    if !has_column {
        conn.execute("ALTER TABLE projects ADD COLUMN signing_token TEXT", [])?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: i64,
    pub gitlab_path: String,
    pub name: String,
    pub workspace_path: String,
    pub enabled: bool,
    pub signing_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRecord {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub hetzner_snapshot_id: String,
    pub server_type: String,
    pub hetzner_location: String,
    pub model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptRecord {
    pub tag_id: i64,
    pub template: String,
}

#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub controller_url: String,
    pub firewall_id: String,
    pub job_timeout_secs: u64,
    pub heartbeat_stale_secs: u64,
    pub provisioning_timeout_secs: u64,
    pub max_concurrent_jobs: u64,
}

#[derive(Debug, Clone)]
pub struct ResolvedTag {
    pub project: ProjectRecord,
    pub tag: TagRecord,
    pub prompt_template: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    fn with_conn<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Connection) -> Result<T, DbError>,
    {
        let guard = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        f(&guard)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn add_project(
        &self,
        gitlab_path: &str,
        name: &str,
        workspace_path: &str,
    ) -> Result<i64, DbError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO projects (gitlab_path, name, workspace_path) VALUES (?1, ?2, ?3)",
                params![gitlab_path, name, workspace_path],
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    pub fn remove_project(&self, gitlab_path: &str) -> Result<bool, DbError> {
        self.with_conn(|conn| {
            let n = conn.execute(
                "DELETE FROM projects WHERE gitlab_path = ?1",
                params![gitlab_path],
            )?;
            Ok(n > 0)
        })
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>, DbError> {
        self.with_conn(|conn| {
            let mut stmt =
                conn.prepare(&format!("{PROJECT_SELECT} ORDER BY gitlab_path"))?;
            let rows = stmt.query_map([], map_project_row)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
        })
    }

    pub fn set_project_signing_token(
        &self,
        gitlab_path: &str,
        signing_token: &str,
    ) -> Result<(), DbError> {
        let signing_token = signing_token.trim();
        if signing_token.is_empty() {
            return Err(DbError::InvalidSigningToken);
        }
        standardwebhooks::Webhook::new(signing_token).map_err(|_| DbError::InvalidSigningToken)?;
        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE projects SET signing_token = ?2 WHERE gitlab_path = ?1",
                params![gitlab_path, signing_token],
            )?;
            if n == 0 {
                return Err(DbError::ProjectNotFound(gitlab_path.to_string()));
            }
            Ok(())
        })
    }

    pub fn list_project_signing_tokens(&self) -> Result<Vec<String>, DbError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT signing_token FROM projects
                 WHERE enabled = 1 AND signing_token IS NOT NULL AND signing_token != ''",
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
        })
    }

    pub fn find_project_by_gitlab_path(
        &self,
        gitlab_path: &str,
    ) -> Result<Option<ProjectRecord>, DbError> {
        self.with_conn(|conn| {
            conn.query_row(
                &format!("{PROJECT_SELECT} WHERE gitlab_path = ?1"),
                params![gitlab_path],
                map_project_row,
            )
            .optional()
            .map_err(DbError::from)
        })
    }

    pub fn resolve_project(&self, project: &Project) -> Result<Option<ProjectRecord>, DbError> {
        if let Some(path) = project.path_with_namespace.as_deref() {
            if let Some(rec) = self.find_project_by_gitlab_path(path)? {
                return Ok(Some(rec));
            }
        }
        self.find_project_by_gitlab_path(&project.id.to_string())
    }

    pub fn find_project_by_name(&self, name: &str) -> Result<Option<ProjectRecord>, DbError> {
        self.with_conn(|conn| {
            conn.query_row(
                &format!("{PROJECT_SELECT} WHERE name = ?1 OR gitlab_path = ?1"),
                params![name],
                map_project_row,
            )
            .optional()
            .map_err(DbError::from)
        })
    }

    pub fn add_tag(
        &self,
        project_ref: &str,
        tag_name: &str,
        snapshot_id: &str,
        server_type: Option<&str>,
        location: Option<&str>,
        model: Option<&str>,
    ) -> Result<i64, DbError> {
        validate_tag_name(tag_name).map_err(DbError::InvalidTag)?;
        let project = self
            .find_project_by_name(project_ref)?
            .ok_or_else(|| DbError::ProjectNotFound(project_ref.to_string()))?;
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO tags (project_id, name, hetzner_snapshot_id, server_type, hetzner_location, model)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(project_id, name) DO UPDATE SET
                   hetzner_snapshot_id = excluded.hetzner_snapshot_id,
                   server_type = excluded.server_type,
                   hetzner_location = excluded.hetzner_location,
                   model = excluded.model",
                params![
                    project.id,
                    tag_name,
                    snapshot_id,
                    server_type.unwrap_or("cpx32"),
                    location.unwrap_or("fsn1"),
                    model.unwrap_or("composer-2.5"),
                ],
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    pub fn set_tag_snapshot(
        &self,
        project_ref: &str,
        tag_name: &str,
        snapshot_id: &str,
    ) -> Result<(), DbError> {
        let project = self
            .find_project_by_name(project_ref)?
            .ok_or_else(|| DbError::ProjectNotFound(project_ref.to_string()))?;
        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE tags SET hetzner_snapshot_id = ?3 WHERE project_id = ?1 AND name = ?2",
                params![project.id, tag_name, snapshot_id],
            )?;
            if n == 0 {
                return Err(DbError::TagNotFound {
                    project: project_ref.to_string(),
                    tag: tag_name.to_string(),
                });
            }
            Ok(())
        })
    }

    pub fn list_tags(
        &self,
        project_ref: Option<&str>,
    ) -> Result<Vec<(ProjectRecord, TagRecord)>, DbError> {
        let project_id = if let Some(r) = project_ref {
            Some(
                self.find_project_by_name(r)?
                    .ok_or_else(|| DbError::ProjectNotFound(r.to_string()))?
                    .id,
            )
        } else {
            None
        };

        self.with_conn(|conn| {
            let map_row = |row: &rusqlite::Row<'_>| {
                Ok((
                    ProjectRecord {
                        id: row.get(0)?,
                        gitlab_path: row.get(1)?,
                        name: row.get(2)?,
                        workspace_path: row.get(3)?,
                        enabled: row.get::<_, i64>(4)? != 0,
                        signing_token: row.get(5)?,
                    },
                    TagRecord {
                        id: row.get(6)?,
                        project_id: row.get(7)?,
                        name: row.get(8)?,
                        hetzner_snapshot_id: row.get(9)?,
                        server_type: row.get(10)?,
                        hetzner_location: row.get(11)?,
                        model: row.get(12)?,
                        enabled: row.get::<_, i64>(13)? != 0,
                    },
                ))
            };

            if let Some(pid) = project_id {
                let mut stmt = conn.prepare(
                    "SELECT p.id, p.gitlab_path, p.name, p.workspace_path, p.enabled, p.signing_token,
                            t.id, t.project_id, t.name, t.hetzner_snapshot_id, t.server_type,
                            t.hetzner_location, t.model, t.enabled
                     FROM tags t JOIN projects p ON p.id = t.project_id
                     WHERE p.id = ?1 ORDER BY p.gitlab_path, t.name",
                )?;
                let rows = stmt.query_map(params![pid], map_row)?;
                rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
            } else {
                let mut stmt = conn.prepare(
                    "SELECT p.id, p.gitlab_path, p.name, p.workspace_path, p.enabled, p.signing_token,
                            t.id, t.project_id, t.name, t.hetzner_snapshot_id, t.server_type,
                            t.hetzner_location, t.model, t.enabled
                     FROM tags t JOIN projects p ON p.id = t.project_id
                     ORDER BY p.gitlab_path, t.name",
                )?;
                let rows = stmt.query_map([], map_row)?;
                rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
            }
        })
    }

    pub fn set_prompt(
        &self,
        project_ref: &str,
        tag_name: &str,
        template: &str,
    ) -> Result<(), DbError> {
        let project = self
            .find_project_by_name(project_ref)?
            .ok_or_else(|| DbError::ProjectNotFound(project_ref.to_string()))?;
        self.with_conn(|conn| {
            let tag_id: i64 = conn
                .query_row(
                    "SELECT id FROM tags WHERE project_id = ?1 AND name = ?2",
                    params![project.id, tag_name],
                    |row| row.get(0),
                )
                .map_err(|_| DbError::TagNotFound {
                    project: project_ref.to_string(),
                    tag: tag_name.to_string(),
                })?;
            conn.execute(
                "INSERT INTO prompts (tag_id, template) VALUES (?1, ?2)
                 ON CONFLICT(tag_id) DO UPDATE SET template = excluded.template",
                params![tag_id, template],
            )?;
            Ok(())
        })
    }

    pub fn get_prompt(&self, project_ref: &str, tag_name: &str) -> Result<String, DbError> {
        let project = self
            .find_project_by_name(project_ref)?
            .ok_or_else(|| DbError::ProjectNotFound(project_ref.to_string()))?;
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT pr.template FROM prompts pr
                 JOIN tags t ON t.id = pr.tag_id
                 WHERE t.project_id = ?1 AND t.name = ?2",
                params![project.id, tag_name],
                |row| row.get(0),
            )
            .map_err(|_| DbError::TagNotFound {
                project: project_ref.to_string(),
                tag: tag_name.to_string(),
            })
        })
    }

    /// Pick the highest-priority triggered issue tag that is configured for this project.
    pub fn resolve_issue_tag(
        &self,
        project: &Project,
        tags: &[String],
    ) -> Result<Option<String>, DbError> {
        for tag in order_issue_tags(tags) {
            if self.resolve_tag(project, &tag)?.is_some() {
                return Ok(Some(tag));
            }
        }
        Ok(None)
    }

    pub fn resolve_tag(
        &self,
        project: &Project,
        tag_name: &str,
    ) -> Result<Option<ResolvedTag>, DbError> {
        let Some(project_rec) = self.resolve_project(project)? else {
            return Ok(None);
        };
        if !project_rec.enabled {
            return Ok(None);
        }
        let row: Option<(TagRecord, String)> = self.with_conn(|conn| {
            conn.query_row(
                "SELECT t.id, t.project_id, t.name, t.hetzner_snapshot_id, t.server_type,
                        t.hetzner_location, t.model, t.enabled, COALESCE(pr.template, '')
                 FROM tags t
                 LEFT JOIN prompts pr ON pr.tag_id = t.id
                 WHERE t.project_id = ?1 AND t.name = ?2 AND t.enabled = 1",
                params![project_rec.id, tag_name],
                |row| {
                    Ok((
                        TagRecord {
                            id: row.get(0)?,
                            project_id: row.get(1)?,
                            name: row.get(2)?,
                            hetzner_snapshot_id: row.get(3)?,
                            server_type: row.get(4)?,
                            hetzner_location: row.get(5)?,
                            model: row.get(6)?,
                            enabled: row.get::<_, i64>(7)? != 0,
                        },
                        row.get(8)?,
                    ))
                },
            )
            .optional()
            .map_err(DbError::from)
        })?;

        Ok(row.map(|(tag_rec, template)| ResolvedTag {
            project: project_rec,
            tag: tag_rec,
            prompt_template: template,
        }))
    }

    pub fn get_settings(&self) -> Result<Settings, DbError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut map = std::collections::HashMap::new();
            for row in rows {
                let (k, v) = row?;
                map.insert(k, v);
            }
            Ok(Settings {
                controller_url: map.get("controller_url").cloned().unwrap_or_default(),
                firewall_id: map.get("firewall_id").cloned().unwrap_or_default(),
                job_timeout_secs: map
                    .get("job_timeout_secs")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10_800),
                heartbeat_stale_secs: map
                    .get("heartbeat_stale_secs")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
                provisioning_timeout_secs: map
                    .get("provisioning_timeout_secs")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(900),
                max_concurrent_jobs: map
                    .get("max_concurrent_jobs")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            })
        })
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )?;
            Ok(())
        })
    }
}
