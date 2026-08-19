//! One resource per family of legacy routes.

mod status;
mod templates;

pub use status::LegacyStatus;
pub use templates::{LegacyTemplates, UpdateGwTemplate};
