//! What a refusal from the API looks like on this side.

use serde::Deserialize;
use std::fmt;

/// One field-level complaint inside a validation error.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ErrorDetail {
    /// The field the complaint is about.
    pub field: String,
    /// The catalog code for this field.
    pub code: String,
    /// What is wrong with it, in pt-BR.
    pub message: String,
}

/// A refusal from the API.
///
/// Every call returns it in the `Err` arm, so a failed call cannot be mistaken
/// for a good one. `request_id` is a first-class field on purpose: it is the
/// first thing support asks for, and an SDK that swallowed it would force
/// whoever hit the failure to reproduce the call with curl just to find the
/// number.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    /// The HTTP status. Zero when the API was never reached.
    pub status: u16,
    /// The catalog code -- `not_found`, `forbidden`, `rate_limited`, ...
    pub code: String,
    /// What the API said, in pt-BR.
    pub message: String,
    /// From the error body, or from the `X-Request-Id` header when absent.
    pub request_id: Option<String>,
    /// The field at fault, when the refusal is about one.
    pub field: Option<String>,
    /// One entry per rejected field, on validation errors.
    pub details: Vec<ErrorDetail>,
    /// From the `Retry-After` header, on 429 and 503.
    pub retry_after_seconds: Option<f64>,
}

impl ApiError {
    /// Builds the error the SDK raises when it never reached the API.
    #[must_use]
    pub fn local(status: u16, code: &str, message: String) -> Self {
        Self {
            status,
            code: code.to_owned(),
            message,
            request_id: None,
            field: None,
            details: Vec::new(),
            retry_after_seconds: None,
        }
    }

    /// Whether the API said this is worth trying again.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        self.status == 429 || self.status == 503
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.request_id {
            Some(id) => write!(
                formatter,
                "aronline: {} ({}): {} [request_id={id}]",
                self.code, self.status, self.message
            ),
            None => write!(
                formatter,
                "aronline: {} ({}): {}",
                self.code, self.status, self.message
            ),
        }
    }
}

impl std::error::Error for ApiError {}

/// What every call in this SDK returns.
pub type Result<T> = std::result::Result<T, ApiError>;
