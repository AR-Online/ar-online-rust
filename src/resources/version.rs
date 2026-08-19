//! Which version of the API is running.

use crate::error::Result;
use crate::http::transport::Transport;
use crate::models::VersionInfo;
use std::sync::Arc;

/// The only open route in the SDK.
///
/// It sends no token and works on a client built without one. It is the first
/// thing support asks for.
pub struct VersionResource {
    transport: Arc<Transport>,
}

impl VersionResource {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// The running version, the migration it needs and the environment.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ApiError`] when the API never answers or answers
    /// something that is not the expected JSON.
    pub fn get(&self) -> Result<VersionInfo> {
        // No envelope, and no credential.
        self.transport.bare("/v3/version", false)
    }
}
