// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug)]
pub struct CloudInitParams<'a> {
    pub job_id: Uuid,
    pub controller_url: &'a str,
    pub runtime_token: &'a str,
    pub cursor_api_key: &'a str,
    pub gitlab_token: &'a str,
    pub cloud_init_script: &'a str,
}

#[derive(Debug, Error)]
pub enum CloudInitError {
    #[error("failed to read template: {0}")]
    Io(#[from] std::io::Error),
    #[error("template not found: {0}")]
    NotFound(String),
}

pub fn render_cloud_init(template_path: &Path, params: &CloudInitParams<'_>) -> Result<String, CloudInitError> {
    let template = fs::read_to_string(template_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CloudInitError::NotFound(template_path.display().to_string())
        } else {
            CloudInitError::Io(e)
        }
    })?;

    let rendered = template
        .replace("${job_id}", &params.job_id.to_string())
        .replace("${controller_url}", params.controller_url)
        .replace("${runtime_token}", params.runtime_token)
        .replace("${cursor_api_key}", params.cursor_api_key)
        .replace("${gitlab_token}", params.gitlab_token)
        .replace("${cloud_init_script}", params.cloud_init_script);

    Ok(rendered)
}
