//! What the API is running.

use serde::Deserialize;

/// The running version, the migration it needs and the environment.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VersionInfo {
    /// The application's version.
    pub version: String,
    /// The lowest migration this version needs applied.
    pub min_migration: String,
    /// `production`, `staging` or `local`.
    pub environment: String,
}
