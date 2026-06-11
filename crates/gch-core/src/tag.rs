// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::filter::IssueAgent;

/// Agent tag resolved from a GitLab webhook event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookTag {
    Security,
    Verify,
    Implement,
}

impl WebhookTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            WebhookTag::Security => "security",
            WebhookTag::Verify => "verify",
            WebhookTag::Implement => "implement",
        }
    }

    pub fn from_issue_agent(agent: IssueAgent) -> Self {
        match agent {
            IssueAgent::Verify => WebhookTag::Verify,
            IssueAgent::Implement => WebhookTag::Implement,
        }
    }
}

impl fmt::Display for WebhookTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WebhookTag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "security" => Ok(WebhookTag::Security),
            "verify" => Ok(WebhookTag::Verify),
            "implement" => Ok(WebhookTag::Implement),
            other => Err(format!("unknown tag: {other}")),
        }
    }
}
