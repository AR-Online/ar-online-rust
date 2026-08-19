//! How fresh the copy of the data is.

use serde::Deserialize;

/// A table whose copy is past the threshold.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BehindTable {
    /// The legacy table, `schema.table`.
    pub legacy: String,
    /// How far behind it is.
    pub lag_seconds: i64,
}

/// How far behind the copy is, measured by the database clock.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Freshness {
    /// When the measurement was taken.
    pub refreshed_at: Option<String>,
    /// The newest read mark across every tracked table.
    pub last_load_at: Option<String>,
    /// `None` when no table carries a read mark yet -- which is not zero lag.
    pub worst_lag_seconds: Option<i64>,
    /// How many tables the loader watches.
    pub tables_tracked: i64,
    /// Never loaded is its own count: it is not lag, and the fix is another.
    pub tables_never_loaded: i64,
    /// Past the threshold, worst first.
    pub behind: Vec<BehindTable>,
}
