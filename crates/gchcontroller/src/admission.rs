// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use tokio::sync::Mutex;

/// Serializes provision admission so concurrent webhooks cannot all pass
/// `should_defer_provisioning` before any job is inserted into memory.
#[derive(Debug)]
pub struct ProvisionAdmission {
    gate: Mutex<()>,
}

impl ProvisionAdmission {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            gate: Mutex::new(()),
        })
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.gate.lock().await
    }
}

impl Default for ProvisionAdmission {
    fn default() -> Self {
        Self {
            gate: Mutex::new(()),
        }
    }
}
