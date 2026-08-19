//! How far behind the copy of the data is.

use crate::error::Result;
use crate::http::transport::Transport;
use crate::models::Freshness;
use std::sync::Arc;

/// The freshness of the copy.
///
/// It answers the practical question behind a query that returned less than
/// expected: is the API wrong, or is the load late? Without this number the two
/// look the same.
pub struct FreshnessResource {
    transport: Arc<Transport>,
}

impl FreshnessResource {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// Measured by the database clock, not by the caller's.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ApiError`] when the API refuses the call or never
    /// answers.
    pub fn get(&self) -> Result<Freshness> {
        // No envelope on this one: the route answers the object itself.
        self.transport.bare("/v3/freshness", true)
    }
}
