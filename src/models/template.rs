//! A message template.

use serde::Deserialize;

/// A placeholder a template expects to be filled in.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TemplateVariable {
    /// The placeholder's name.
    pub name: String,
    /// Where the provider puts it. `None` when the channel has no components.
    pub component: Option<String>,
    /// The provider's type for it.
    ///
    /// The wire calls this `type`, which is a keyword here — so the field is
    /// `kind` and serde maps it. The alternative, `r#type`, would put a raw
    /// identifier in every caller's code to save one line in ours.
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

/// A message template.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Template {
    /// The public UUID. The internal id never leaves the API.
    pub id: String,
    /// The template's name.
    pub name: String,
    /// The channel it belongs to.
    pub channel: String,
    /// Only channels that carry a subject fill this in.
    pub subject: Option<String>,
    /// The message body, with its placeholders.
    pub body: String,
    /// Whether it is in use.
    pub active: bool,
    /// How the provider knows it.
    pub provider_identifier: Option<String>,
    /// A flat list -- the provider's component nesting stays in the mirror.
    pub variables: Vec<TemplateVariable>,
    /// ISO 8601 with the real offset, e.g. `2024-10-12T14:11:13-03:00`.
    pub created_at: String,
    /// `None` while it has never been changed.
    pub updated_at: Option<String>,
}
