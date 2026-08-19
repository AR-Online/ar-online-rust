//! A label. Labels belong to a person, not to an entity.

use serde::Deserialize;

/// A label.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tag {
    /// The label's id.
    pub id: String,
    /// What it is called.
    pub name: String,
    /// Its colour, when it has one.
    pub color: Option<String>,
    /// ISO 8601 with the real offset.
    pub created_at: String,
}
