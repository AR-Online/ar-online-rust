//! The per-channel answers of `GET /gw/<canal>/{idEmail}`.
//!
//! In every one of the five routes the id is the notification's uuid -- the
//! e-mail's -- there is no per-channel id to keep.
//!
//! Each field keeps the absence convention the wire uses for it, by the rule in
//! [`LegacyField`]. Dates stay `String`: `"18/07/2026 01:01:32"` carries no
//! offset, so it does not name an unambiguous instant, and parsing it into a
//! date type would mean guessing a timezone and handing back a wrong one.

use super::field::LegacyField;
use serde::Deserialize;
use serde_json::{Map, Value};

/// Status do AR-Email.
///
/// `date_send` and `date_delivery` come as `""` until they happen, while
/// `date_reading` and `date_acceptance` come as `null`. Both mean the same
/// thing -- not yet -- and testing only for one misses half of them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEmail {
    /// `""` until it goes out.
    pub date_send: String,
    /// `""` until the recipient's server accepts it.
    pub date_delivery: String,
    /// `null` until it is read.
    pub date_reading: Option<String>,
    /// `null` until the recipient's server accepts it.
    pub date_acceptance: Option<String>,
    /// Whether the send failed.
    pub error: bool,
    /// Climbs with the stage reached: `Processado`, `Enviado`, `Entregue`,
    /// `Lido`. A failure beats all of them.
    pub description: String,
    /// Filled in when `error` is on.
    pub failure_reason: Option<String>,
    /// The long form of the failure, when the gateway sends one.
    #[serde(default)]
    pub failure_reason_description: LegacyField<String>,
    /// Your own reference, echoed back from the send.
    #[serde(rename = "customID")]
    pub custom_id: Option<String>,
    /// The notification's uuid.
    pub id_email: String,
}

/// One entry of [`StatusSms::answered`].
///
/// The old documentation calls it a list of strings and is wrong: the wire
/// carries objects, whoever integrated read the objects, so the object stays.
pub type SmsAnswer = Map<String, Value>;

/// Status do AR-SMS.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSms {
    /// `Lido (acessou o link)` beats every other label when the link was opened.
    pub description: String,
    /// `""` until it goes out.
    pub date_send: String,
    /// `null` until it is read.
    pub date_reading: Option<String>,
    /// `null` until the recipient answers.
    pub date_answered: Option<String>,
    /// The recipient's answers, as objects.
    #[serde(default)]
    pub answered: Vec<SmsAnswer>,
}

/// Status do AR-WhatsApp.
///
/// The dates that have not happened **vanish** from the response instead of
/// coming null -- hence [`LegacyField`] on every one of them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusWhatsapp {
    /// The stage reached.
    pub description: String,
    /// When it went out.
    #[serde(default)]
    pub date_sent: LegacyField<String>,
    /// When the provider delivered it.
    #[serde(default)]
    pub date_delivery: LegacyField<String>,
    /// When the recipient answered.
    #[serde(default)]
    pub date_response: LegacyField<String>,
    /// When the recipient opened the link.
    #[serde(default)]
    pub date_access_link: LegacyField<String>,
    /// Whether the send failed.
    pub error: bool,
    /// Filled in when `error` is on.
    pub failure_reason: Option<String>,
    /// Always `None` on this route, even when the message has one -- read it on
    /// the e-mail route instead.
    #[serde(rename = "customID")]
    pub custom_id: Option<String>,
    /// The notification's uuid.
    pub id_email: String,
}

/// Status do AR-Voz.
///
/// The one route that never answers 404: an unknown uuid gets a 200 carrying
/// only `description` -- `Não há registro de voz para este envio`. That is not
/// an error, and the SDK does not turn it into one.
///
/// When a call failed before succeeding, the answer tells only the failure:
/// `date_success_call` never travels together with `date_failure_call`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusVoz {
    /// The stage reached, or the sentence for a uuid with no voice record.
    pub description: String,
    /// When the call went out.
    #[serde(default)]
    pub date_sent: LegacyField<String>,
    /// When the call was answered.
    #[serde(default)]
    pub date_success_call: LegacyField<String>,
    /// When the call failed.
    #[serde(default)]
    pub date_failure_call: LegacyField<String>,
    /// The recording's link -- depends on a data load that may lag behind.
    #[serde(default)]
    pub link_call: LegacyField<String>,
}

/// Status do AR-Cartas.
///
/// Two stages change name on the way out: the provider produces `datePrepared`
/// and `dateDelivered`, the response carries `datePreparation` and
/// `dateDelivery`. The provider's names never reach the client.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCarta {
    /// The stage reached.
    pub description: String,
    /// Whether the send failed.
    pub error: bool,
    /// When the letter entered processing.
    #[serde(default)]
    pub date_processing: LegacyField<String>,
    /// When it was prepared -- the provider calls this `datePrepared`.
    #[serde(default)]
    pub date_preparation: LegacyField<String>,
    /// When the Correios took it.
    #[serde(default)]
    pub date_sent: LegacyField<String>,
    /// When it arrived -- the provider calls this `dateDelivered`.
    #[serde(default)]
    pub date_delivery: LegacyField<String>,
    /// The Correios tracking code.
    #[serde(default)]
    pub sro: LegacyField<String>,
    /// The signed link to the delivery receipt.
    #[serde(default)]
    pub link_ar_carta_comprovante: LegacyField<String>,
    /// The public Correios tracking page.
    #[serde(default)]
    pub link_rastreio: LegacyField<String>,
}
