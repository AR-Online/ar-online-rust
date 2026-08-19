//! What a refusal from the legacy gateway looks like on this side.

use std::fmt;

/// A refusal from the legacy gateway.
///
/// The gateway has two ways of saying no: an HTTP status carrying a
/// `{ statusCode, message }` body, and -- in the templates family -- an HTTP
/// 200 whose real code hides inside the `{ data, statusCode }` envelope. Both
/// arrive here, so a caller has one type to match on.
///
/// It is a separate type from [`ApiError`] because the two surfaces refuse
/// differently: /v3 answers a catalog code and a `request_id`, the gateway
/// answers a sentence and, sometimes, a status that contradicts the wire.
/// Squeezing both into one struct would leave half the fields empty on every
/// error, and no way to tell which half.
///
/// [`ApiError`]: crate::ApiError
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyApiError {
    /// The code that matters: the envelope's inner `statusCode` when the
    /// refusal came wrapped, the HTTP status otherwise.
    pub status: u16,
    /// What the wire said. `200` when the envelope hid a 404, `0` when the
    /// call never reached the gateway.
    pub http_status: u16,
    /// What went wrong, in the gateway's own words when it gave any.
    pub message: String,
    // Raw text, not a parsed `serde_json::Value`: a proxy answering HTML is
    // one of the cases that lands here, and a Value could not hold it. Whoever
    // wants the fields calls `body_json()` and gets the parse back.
    /// The body exactly as it came. `None` when there was no answer to read.
    pub body: Option<String>,
}

impl LegacyApiError {
    /// The error the SDK raises on its own, without an answer to quote.
    pub(crate) fn local(status: u16, http_status: u16, message: String) -> Self {
        Self {
            status,
            http_status,
            message,
            body: None,
        }
    }

    /// The body parsed as JSON, or `None` when it is not JSON at all.
    #[must_use]
    pub fn body_json(&self) -> Option<serde_json::Value> {
        serde_json::from_str(self.body.as_deref()?).ok()
    }
}

impl fmt::Display for LegacyApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "aronline legado: {} (http {}): {}",
            self.status, self.http_status, self.message
        )
    }
}

impl std::error::Error for LegacyApiError {}

/// What every call in the legacy area returns.
pub type LegacyResult<T> = std::result::Result<T, LegacyApiError>;
