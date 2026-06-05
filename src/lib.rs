// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod config;
pub mod cursor;
pub mod dedup;
pub mod filter;
pub mod routes;

pub use config::{Config, WebhookAgent};
pub use filter::{
    select_issue_agent, should_forward, should_forward_issue, GitLabIssueWebhook, GitLabMrWebhook,
    IssueAgent, SkipReason,
};
