//! The official SDK for the AR Online API.
//!
//! The SDK speaks the /v3 surface only. The /v1 and /v2 mirrors answer the old
//! contracts byte for byte -- idiosyncrasies included -- and a typed client
//! that "improved" them would break the callers they exist to keep working.

/// Where /v3 lives. Override it to point at staging or at a local process.
pub const DEFAULT_BASE_URL: &str = "https://v3.ar-online.com.br";

/// This crate's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
