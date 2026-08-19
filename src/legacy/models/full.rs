//! The consolidated answer of `GET /gw/full/{idEmail}`.

use serde::Deserialize;
use serde_json::{Map, Value};

/// One status event: the label and when it happened.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEvent {
    /// The stage's name.
    pub label: String,
    /// `dd/mm/aaaa hh:mm:ss`, Brasília time. Fields suffixed `UTC` elsewhere in
    /// the detail blocks carry the same instant in UTC.
    pub date_time: String,
}

/// A channel's detail block inside the full status.
///
/// It comes through raw. These blocks carry the expert-evidence material --
/// signed timestamps, reading trails, geolocation -- in provider-shaped nests
/// that the public documentation shows by example rather than by schema, and a
/// struct invented on top of an example would promise fields this SDK has never
/// seen. An empty block arrives as `{}`, which is one of the four ways this API
/// says "nothing here", and stays an empty map.
pub type FullChannelDetail = Map<String, Value>;

/// Every channel's status history.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct FullHistory {
    /// The e-mail's stages, in order.
    #[serde(default)]
    pub email: Vec<StatusEvent>,
    /// The SMS's stages, in order.
    #[serde(default)]
    pub sms: Vec<StatusEvent>,
    /// The `WhatsApp` message's stages, in order.
    #[serde(default)]
    pub whatsapp: Vec<StatusEvent>,
    /// The call's stages, in order.
    #[serde(default)]
    pub voz: Vec<StatusEvent>,
    /// The letter's stages, in order.
    #[serde(default)]
    pub carta: Vec<StatusEvent>,
}

/// Each channel's latest status.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct FullLastStatus {
    /// The e-mail's current stage.
    #[serde(default)]
    pub email: Option<StatusEvent>,
    /// The SMS's current stage.
    #[serde(default)]
    pub sms: Option<StatusEvent>,
    /// The `WhatsApp` message's current stage.
    #[serde(default)]
    pub whatsapp: Option<StatusEvent>,
    /// The call's current stage.
    #[serde(default)]
    pub voz: Option<StatusEvent>,
    /// The letter's current stage.
    #[serde(default)]
    pub carta: Option<StatusEvent>,
}

/// Status completo -- the forensic view of a notification.
///
/// The history and the latest status are typed; the per-channel detail arrays
/// pass through as [`FullChannelDetail`]. For following one channel's current
/// stage, that channel's own route is the lighter ask.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusFull {
    /// The e-mail's internal numeric code on the platform.
    pub cod_email: i64,
    /// The full status history of each channel.
    #[serde(default)]
    pub status_full: FullHistory,
    /// Each channel's latest status.
    #[serde(default)]
    pub last_status: FullLastStatus,
    /// The e-mail's forensic blocks.
    #[serde(default)]
    pub email: Vec<FullChannelDetail>,
    /// The SMS's forensic blocks.
    #[serde(default)]
    pub sms: Vec<FullChannelDetail>,
    /// The `WhatsApp` message's forensic blocks.
    #[serde(default)]
    pub whatsapp: Vec<FullChannelDetail>,
    /// The call's forensic blocks.
    #[serde(default)]
    pub voz: Vec<FullChannelDetail>,
    /// The letter's forensic blocks.
    #[serde(default)]
    pub carta: Vec<FullChannelDetail>,
}
