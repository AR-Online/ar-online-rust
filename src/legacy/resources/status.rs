//! "What happened to this notification", one route per channel.

use crate::http::transport::encode_segment;
use crate::legacy::error::LegacyResult;
use crate::legacy::models::{
    StatusCarta, StatusEmail, StatusFull, StatusSms, StatusVoz, StatusWhatsapp,
};
use crate::legacy::transport::LegacyTransport;
use std::sync::Arc;

/// The status routes of the legacy gateway -- the most used surface of the old
/// API.
///
/// Every method takes the same id: the notification's uuid, the one a send
/// answered. Asking for the SMS is asking for the SMS *of that notification*;
/// there is no per-channel id to keep.
///
/// An unknown id answers 404 and arrives as a [`LegacyApiError`] -- except on
/// [`voz`](Self::voz), where the old API answers 200 with a sentence, and so
/// does this.
///
/// No /v3 equivalent yet, for any of them.
///
/// [`LegacyApiError`]: crate::legacy::LegacyApiError
pub struct LegacyStatus {
    transport: Arc<LegacyTransport>,
}

impl LegacyStatus {
    pub(crate) fn new(transport: Arc<LegacyTransport>) -> Self {
        Self { transport }
    }

    /// When it went out, when the recipient's server accepted it, when it was read.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn email(&self, id: &str) -> LegacyResult<StatusEmail> {
        self.transport
            .json("GET", &format!("/gw/email/{}", encode_segment(id)), &[])
    }

    /// What happened to the SMS, and the recipient's answers when there were any.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn sms(&self, id: &str) -> LegacyResult<StatusSms> {
        self.transport
            .json("GET", &format!("/gw/sms/{}", encode_segment(id)), &[])
    }

    /// The `WhatsApp` message's stages. The dates that have not happened are
    /// [`Missing`], not null.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers.
    ///
    /// [`Missing`]: crate::legacy::LegacyField::Missing
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn whatsapp(&self, id: &str) -> LegacyResult<StatusWhatsapp> {
        self.transport
            .json("GET", &format!("/gw/whatsapp/{}", encode_segment(id)), &[])
    }

    /// The call's outcome. Never a 404: a uuid with no record answers 200
    /// carrying only a `description`, and that is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn voz(&self, id: &str) -> LegacyResult<StatusVoz> {
        self.transport
            .json("GET", &format!("/gw/voz/{}", encode_segment(id)), &[])
    }

    /// The letter's stages, preparation to delivery, with the Correios tracking.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn carta(&self, id: &str) -> LegacyResult<StatusCarta> {
        self.transport
            .json("GET", &format!("/gw/carta/{}", encode_segment(id)), &[])
    }

    /// Every channel's forensic data in one call. For following a single
    /// channel's current stage, that channel's route is the lighter ask.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn full(&self, id: &str) -> LegacyResult<StatusFull> {
        self.transport
            .json("GET", &format!("/gw/full/{}", encode_segment(id)), &[])
    }
}
