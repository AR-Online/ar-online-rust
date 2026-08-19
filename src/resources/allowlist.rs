//! The recipients allowed to receive messages.

use crate::error::Result;
use crate::http::transport::Transport;
use crate::models::AllowlistEntry;
use std::sync::Arc;

/// The allowed recipients.
///
/// The legacy called this a whitelist and answered it under the key `leads`, a
/// copy-paste that became contract. Here it is an allowlist, and the name says
/// what the list holds. Like labels, it is personal -- an integration token
/// gets 403.
pub struct Allowlist {
    transport: Arc<Transport>,
}

impl Allowlist {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// Your allowed recipients, ordered by recipient.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ApiError`] when the API refuses the call or never
    /// answers.
    pub fn list(&self) -> Result<Vec<AllowlistEntry>> {
        self.transport.envelope("/v3/allowlist", &[])
    }
}
