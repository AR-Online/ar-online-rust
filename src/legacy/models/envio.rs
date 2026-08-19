//! The send contract of `POST /gw/email`.
//!
//! The field names are the gateway's, Portuguese and all. The legacy area keeps
//! the old vocabulary on purpose: an English rendition invented here would
//! create names that exist in no documentation anywhere.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An attachment, carried inline as base64.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Anexo {
    /// The file name the recipient sees.
    pub name: String,
    /// The file's bytes, base64-encoded.
    pub base64: String,
}

/// Question and answer gate on the notification's page. Needs prior enablement.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct EnvioValidation {
    /// What the recipient is asked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// The answer that opens the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<String>,
}

/// When the SMS goes out.
///
/// One of the two things the SDK checks locally, because it is enumerable: a
/// value outside the pair could only ever be a typo. Business rules -- `to`
/// required on an e-mail-only send, number formats, whether a template exists
/// -- stay on the server, where they already live. A duplicated rule drifts,
/// and the client's copy is the one that lies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SmsTypeSend {
    /// `"1"`, the gateway's default: only if the e-mail is not sent/delivered.
    #[serde(rename = "1")]
    SomenteSeFalhar,
    /// `"2"`: always, regardless of the e-mail.
    #[serde(rename = "2")]
    Sempre,
}

impl SmsTypeSend {
    /// The value that travels on the wire.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SomenteSeFalhar => "1",
            Self::Sempre => "2",
        }
    }
}

/// The SMS leg of a send.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanalSms {
    /// The mobile number, digits only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// When it goes out. The gateway defaults to [`SmsTypeSend::SomenteSeFalhar`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_send: Option<SmsTypeSend>,
    /// Up to 140 characters; `{SHORT_LINK}` is expanded by the gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_message: Option<String>,
}

/// The `WhatsApp` leg of a send.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CanalWhatsapp {
    /// The mobile number, digits only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// The custom template's variables, including the `template` identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Map<String, Value>>,
}

/// The voice-call leg of a send.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CanalVoz {
    /// The number to call, digits only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// The voice template's identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// What fills the template in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Map<String, Value>>,
}

/// The physical-letter leg of a send.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CanalCarta {
    /// The addressee's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The letter's layout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modelo: Option<String>,
    /// The letter template's identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// What fills the template in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Map<String, Value>>,
}

/// What a send takes.
///
/// Despite the path saying e-mail, this is the multichannel request: each
/// optional block adds a channel to the same notification.
///
/// ```
/// use aronline::legacy::{CanalSms, EnvioRequest};
///
/// let envio = EnvioRequest {
///     to: Some("joao@exemplo.com".to_owned()),
///     sms: Some(CanalSms {
///         number: Some("11999998888".to_owned()),
///         ..CanalSms::default()
///     }),
///     ..EnvioRequest::new("João da Silva", "Documento importante", "<p>Olá.</p>")
/// };
///
/// assert_eq!(envio.name_to, "João da Silva");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvioRequest {
    /// The recipient's name.
    pub name_to: String,
    /// The recipient's e-mail. The server requires it on an e-mail-only send.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// The subject line.
    pub subject: String,
    /// The message body, in HTML.
    pub content: String,
    /// Your own reference, echoed back by the status routes.
    #[serde(rename = "customID", skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    /// Files to carry along.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Anexo>,
    /// The question gate on the notification's page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<EnvioValidation>,
    /// Adds the SMS channel to this send.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms: Option<CanalSms>,
    /// Adds the `WhatsApp` channel to this send.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whatsapp: Option<CanalWhatsapp>,
    /// Adds the voice call to this send.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voz: Option<CanalVoz>,
    /// Adds the physical letter to this send.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carta: Option<CanalCarta>,
}

impl EnvioRequest {
    /// A send with the three fields the gateway always wants, and nothing else.
    ///
    /// Fill the rest in with struct update syntax -- every other field is
    /// public and defaults to absent.
    #[must_use]
    pub fn new(
        name_to: impl Into<String>,
        subject: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            name_to: name_to.into(),
            subject: subject.into(),
            content: content.into(),
            ..Self::default()
        }
    }
}

/// What a send answers.
///
/// Processing is asynchronous: `id_email` is the one handle for every later
/// question -- status of any channel, proofs, the works.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvioResponse {
    /// The notification's uuid.
    pub id_email: String,
}
