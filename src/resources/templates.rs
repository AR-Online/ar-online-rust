//! The message templates your identity reaches.

use crate::error::Result;
use crate::http::transport::{encode_segment, Transport};
use crate::models::{Channel, Template};
use std::sync::Arc;

/// Message templates.
pub struct Templates {
    transport: Arc<Transport>,
}

impl Templates {
    pub(crate) fn new(transport: Arc<Transport>) -> Self {
        Self { transport }
    }

    /// The templates you reach, newest first.
    ///
    /// Pass `None` for every channel. Unlike the `/gw/templates` mirror, it
    /// does not filter by lineage: a template created in /v3 has no legacy row
    /// and still shows up.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ApiError`] when the API refuses the call or never
    /// answers.
    pub fn list(&self, channel: Option<Channel>) -> Result<Vec<Template>> {
        match channel {
            Some(channel) => self
                .transport
                .envelope("/v3/templates", &[("channel", channel.as_str())]),
            None => self.transport.envelope("/v3/templates", &[]),
        }
    }

    /// One template by its public UUID.
    ///
    /// A template that does not exist and one that is not yours fail the same
    /// way -- 404. Answering 403 would confirm the id exists.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ApiError`] when the API refuses the call or never
    /// answers.
    pub fn get(&self, id: &str) -> Result<Template> {
        self.transport
            .envelope(&format!("/v3/templates/{}", encode_segment(id)), &[])
    }
}
