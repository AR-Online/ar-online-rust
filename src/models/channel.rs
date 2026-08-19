//! The channels a template can belong to.

use std::fmt;

/// A channel accepted by the template filter.
///
/// The list is closed on the server (decision A11): a value outside it answers
/// 400 `invalid_value` with the accepted ones in the message, never a silent
/// empty list. An enum moves that refusal to compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    /// `email`
    Email,
    /// `sms`
    Sms,
    /// `whatsapp`
    WhatsApp,
    /// `voice`
    Voice,
    /// `letter`
    Letter,
}

impl Channel {
    /// Every channel, in the order the API documents them.
    pub const ALL: [Self; 5] = [
        Self::Email,
        Self::Sms,
        Self::WhatsApp,
        Self::Voice,
        Self::Letter,
    ];

    /// The value that travels on the wire.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Sms => "sms",
            Self::WhatsApp => "whatsapp",
            Self::Voice => "voice",
            Self::Letter => "letter",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
