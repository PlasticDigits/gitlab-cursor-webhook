// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod config;
pub mod cursor;
pub mod dedup;
pub mod filter;
pub mod routes;

pub use config::Config;
pub use filter::{should_forward, GitLabMrWebhook, SkipReason};
