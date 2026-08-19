//! The shapes /v3 answers.
//!
//! The field names are the API's, which is also Rust's convention -- the two
//! agree on `snake_case`, so nothing is renamed in the middle and nothing can
//! drift from the server without the compiler noticing.

mod allowlist;
mod channel;
mod freshness;
mod tag;
mod template;
mod version;

pub use allowlist::AllowlistEntry;
pub use channel::Channel;
pub use freshness::Freshness;
pub use tag::Tag;
pub use template::{Template, TemplateVariable};
pub use version::VersionInfo;
