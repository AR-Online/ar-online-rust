//! How fresh the copy of the data is.

use serde::Deserialize;

/// How far behind the copy is, measured by the database clock.
///
/// It answers in COUNTS, not in a list of tables: "46 tracked, 3 behind" is an
/// answer to "is it fresh"; forty-six table names is a report nobody reads at
/// the moment the question is asked.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Freshness {
    /// When the measurement was taken.
    pub refreshed_at: Option<String>,
    /// The newest read mark across every tracked source.
    pub last_load_at: Option<String>,
    /// `None` when no source carries a read mark yet -- which is not zero lag.
    pub worst_lag_seconds: Option<i64>,
    /// How many sources the load watches.
    pub sources_tracked: i64,
    /// How many are past the threshold.
    pub sources_behind: i64,
    /// Never loaded is its own count: it is not lag, and the fix is another.
    pub sources_not_loaded: i64,
}
