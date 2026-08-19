//! The webhook payloads.
//!
//! The SDK does not receive HTTP for you -- webhooks arrive at *your* endpoint.
//! These are the payload structs, exported so whoever receives them does not
//! have to type the contract by hand.

use super::field::LegacyField;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

/// The channels a webhook event names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookChannel {
    /// AR-Email.
    Email,
    /// AR-SMS.
    Sms,
    /// AR-WhatsApp.
    Whatsapp,
    /// AR-Voz.
    Voz,
    /// AR-Cartas.
    Carta,
}

/// The default payload, delivered unless v2 was enabled with support.
///
/// On failure events the three dates come `null` together.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayloadV1 {
    /// The notification's uuid -- the same `idEmail` a send answers.
    #[serde(rename = "notificationID")]
    pub notification_id: String,
    /// Which channel the event is about.
    pub channel: String,
    /// The stage reached.
    pub description: String,
    /// When it went out.
    pub date_sent: Option<String>,
    /// When it was delivered.
    pub date_delivery: Option<String>,
    /// When it was read.
    pub date_read: Option<String>,
    /// When the event itself was recorded.
    pub log_date: String,
}

/// How a v2 delivery describes itself.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookMetadata {
    /// Always `v2` on this payload.
    pub webhook_version: String,
    /// Delivery attempt -- up to 4 with the retry schedule.
    pub attempt: i64,
}

/// The enriched payload, enabled by asking support.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayloadV2 {
    /// The payload's own version.
    pub event_version: String,
    /// ISO 8601 timestamp of the event itself.
    pub occurred_at: String,
    /// The notification's uuid -- the same `idEmail` a send answers.
    #[serde(rename = "notificationID")]
    pub notification_id: String,
    /// Which channel the event is about.
    pub channel: WebhookChannel,
    /// The stage reached.
    pub status: String,
    /// When the stage happened, when the channel knows.
    #[serde(default)]
    pub status_timestamp: LegacyField<String>,
    /// Mirrors the answer of the channel's own status route. Read it with
    /// [`payload_as`](Self::payload_as).
    pub payload: Value,
    /// How this delivery describes itself.
    pub metadata: WebhookMetadata,
}

impl WebhookPayloadV2 {
    /// Reads `payload` as the status struct of the channel in `channel`.
    ///
    /// It stays a raw [`Value`] on the struct because which struct it is
    /// depends on a sibling field: an untagged enum would happily decode a
    /// voice payload as a letter one, since both are almost entirely optional.
    /// Narrowing by `channel` first is the only reading that cannot be wrong.
    ///
    /// ```
    /// use aronline::legacy::{StatusEmail, WebhookChannel, WebhookPayloadV2};
    ///
    /// # fn read(event: &WebhookPayloadV2) -> Option<StatusEmail> {
    /// if event.channel == WebhookChannel::Email {
    ///     return event.payload_as::<StatusEmail>().ok();
    /// }
    /// # None
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the serde error when the payload is not that channel's shape.
    pub fn payload_as<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        T::deserialize(&self.payload)
    }
}
