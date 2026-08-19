//! A recipient allowed to receive messages.

use serde::Deserialize;

/// An allowed recipient.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AllowlistEntry {
    /// The entry's id.
    pub id: String,
    /// Who is allowed.
    pub recipient: String,
    /// ISO 8601 with the real offset.
    pub created_at: String,
}
