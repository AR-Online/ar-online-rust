//! One resource per family of routes.

mod allowlist;
mod freshness;
mod tags;
mod templates;
mod version;

pub use allowlist::Allowlist;
pub use freshness::FreshnessResource;
pub use tags::Tags;
pub use templates::Templates;
pub use version::VersionResource;
