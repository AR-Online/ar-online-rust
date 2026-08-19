//! A template as the legacy gateway shapes it.

use serde::Deserialize;
use serde_json::{Map, Value};

/// The legacy type codes the template list filters by.
///
/// An unknown code answers an **empty list, not an error** -- if you expect
/// results and get nothing, check the code first. An enum moves that mistake to
/// compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GwTemplateType {
    /// `1`
    WhatsApp,
    /// `2`
    Email,
    /// `3`
    Sms,
    /// `4`
    Carta,
}

impl GwTemplateType {
    /// Every code, in the order the gateway numbers them.
    pub const ALL: [Self; 4] = [Self::WhatsApp, Self::Email, Self::Sms, Self::Carta];

    /// The value that travels in the query string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WhatsApp => "1",
            Self::Email => "2",
            Self::Sms => "3",
            Self::Carta => "4",
        }
    }
}

/// A template as the legacy gateway shapes it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GwTemplate {
    /// The public UUID.
    pub id: String,
    /// The provider's identifier, e.g. `hx_boleto_01`.
    pub template_id: Option<String>,
    /// The template's name.
    pub nome: String,
    /// The channel as a word: `whatsapp`, `email`, `sms`, `carta`.
    pub tipo: String,
    /// The message body, with its placeholders.
    pub conteudo: String,
    /// The provider's variable nests, passed through as they come.
    pub variaveis: Option<Vec<Map<String, Value>>>,
    /// Always `None` -- the legacy column was 100% null. Do not build logic on it.
    pub metadata: Option<Value>,
    /// Whether it is in use.
    pub ativo: bool,
    /// Always `1` -- template versioning never shipped. Do not build logic on it.
    pub versao: i64,
    /// Looks ISO with a `Z`, but the `Z` is not really UTC -- see the API docs'
    /// concepts page.
    pub criado_em: String,
    /// `None` while it has never been changed.
    pub atualizado_em: Option<String>,
    /// Always `None` -- the legacy column was 100% null.
    pub criado_por: Option<Value>,
}

/// What the three write routes answer, passed through untyped.
///
/// Production has not been fixtured for the writes yet, so the SDK hands the
/// object over as it came rather than promise fields it has never seen. It
/// tightens into a struct when the mirror proves the shape.
pub type GwTemplateWriteResult = Map<String, Value>;
