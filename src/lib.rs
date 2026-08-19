//! The official SDK for the AR Online API.
//!
//! You do not build URLs, set headers, unwrap envelopes or read status codes.
//! You call a method, get a typed struct, and a refusal comes back as an
//! [`ApiError`].
//!
//! ```no_run
//! use aronline::{Channel, Client};
//!
//! # fn main() -> Result<(), aronline::ApiError> {
//! let client = Client::builder().token(std::env::var("AR_TOKEN").unwrap_or_default()).build();
//!
//! for template in client.templates.list(Some(Channel::WhatsApp))? {
//!     println!("{} {}", template.name, template.variables.len());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Sending, status and proofs live on the old gateway, and the SDK speaks that
//! too -- see [`legacy`], reached through [`Client::legacy`]. As /v3 grows an
//! equivalent for a route, the legacy function swaps its transport without
//! changing signature, so migrating is an update, not a rewrite.
//!
//! The /v1 and /v2 mirrors are a different matter: they answer the old
//! contracts byte for byte for callers who cannot move yet, and no SDK covers
//! them.

// `ApiError` is past clippy's 128-byte threshold for an Err variant, and it
// stays that way. What makes it big is what makes it useful: the catalog code,
// the pt-BR message, the request_id support asks for, and one entry per
// rejected field. Boxing it would shave a pointer off a path that only runs
// when a call already failed, in exchange for `Box<ApiError>` in the signature
// of every method a partner calls.
#![allow(clippy::result_large_err)]

mod client;
mod error;
mod http;
pub mod legacy;
mod models;
mod resources;

pub use client::{Client, ClientBuilder};
pub use error::{ApiError, ErrorDetail, Result};
pub use http::{DEFAULT_BASE_URL, DEFAULT_TIMEOUT};
pub use models::{
    AllowlistEntry, Channel, Freshness, Tag, Template, TemplateVariable, VersionInfo,
};
pub use resources::{Allowlist, FreshnessResource, Tags, Templates, VersionResource};

/// This crate's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
