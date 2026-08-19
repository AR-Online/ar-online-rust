//! Your labels.

use crate::error::Result;
use crate::http::transport::{encode_segment, Transport};
use crate::models::Tag;
use std::sync::Arc;

/// Labels are personal: these routes answer a PERSON's token.
///
/// An integration token gets 403 saying so, rather than an empty list -- which
/// would read as "you have none".
pub struct Tags {
    transport: Arc<Transport>,
}

impl Tags {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// Your labels, ordered by name.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ApiError`] when the API refuses the call or never
    /// answers.
    pub fn list(&self) -> Result<Vec<Tag>> {
        self.transport.envelope("/v3/tags", &[])
    }

    /// One of your labels.
    ///
    /// A label that does not exist and one that is not yours both answer 404.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ApiError`] when the API refuses the call or never
    /// answers.
    pub fn get(&self, id: &str) -> Result<Tag> {
        self.transport
            .envelope(&format!("/v3/tags/{}", encode_segment(id)), &[])
    }
}
