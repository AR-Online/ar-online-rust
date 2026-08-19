//! The legacy gateway, as functions.

use super::base64;
use super::error::{LegacyApiError, LegacyResult};
use super::models::{EnvioRequest, EnvioResponse, SendingProof};
use super::resources::{LegacyStatus, LegacyTemplates};
use super::transport::LegacyTransport;
use crate::http::transport::encode_segment;
use serde::Deserialize;
use std::sync::Arc;

/// What finalizing the ladder answers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FinalizarReguaResult {
    /// The gateway's sentence, when it sends one.
    #[serde(default)]
    pub message: Option<String>,
}

/// Everything docs.ar-online.com.br documents of the gateway, spoken exactly as
/// the old API speaks it.
///
/// This area exists so an integration written against the old contract gets
/// typed calls today. As /v3 grows an equivalent for a route, the function here
/// swaps its transport without changing shape -- the migration happens under
/// your feet, not in your code. Each function's documentation names its /v3
/// equivalent when one exists.
///
/// ```no_run
/// use aronline::legacy::EnvioRequest;
/// use aronline::Client;
///
/// # fn main() -> Result<(), aronline::legacy::LegacyApiError> {
/// let client = Client::builder().legacy_token("jwt-do-gateway").build();
///
/// let sent = client.legacy().send(&EnvioRequest::new(
///     "João da Silva",
///     "Documento importante",
///     "<p>Você recebeu um documento.</p>",
/// ))?;
///
/// let status = client.legacy().status().email(&sent.id_email)?;
///
/// println!("{}", status.description);
/// # Ok(())
/// # }
/// ```
pub struct LegacyArea {
    status: LegacyStatus,
    templates: LegacyTemplates,
    transport: Arc<LegacyTransport>,
}

impl LegacyArea {
    pub(crate) fn new(transport: Arc<LegacyTransport>) -> Self {
        Self {
            status: LegacyStatus::new(Arc::clone(&transport)),
            templates: LegacyTemplates::new(Arc::clone(&transport)),
            transport,
        }
    }

    /// Per-channel status and the consolidated view.
    #[must_use]
    pub fn status(&self) -> &LegacyStatus {
        &self.status
    }

    /// The gateway's template routes.
    #[must_use]
    pub fn templates(&self) -> &LegacyTemplates {
        &self.templates
    }

    /// Sends a notification -- `POST /gw/email`, the multichannel route despite
    /// the name.
    ///
    /// Processing is asynchronous: keep the `id_email` you get back, it is the
    /// handle for every status and proof question later.
    ///
    /// No /v3 equivalent yet.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the send or never
    /// answers.
    pub fn send(&self, request: &EnvioRequest) -> LegacyResult<EnvioResponse> {
        self.transport.json_body("POST", "/gw/email", request)
    }

    /// The sending proof as a PDF.
    ///
    /// The wire carries it in base64 inside JSON; this decodes it for you and
    /// keeps the raw string reachable. While the e-mail has no delivery status
    /// the gateway answers a message instead and `pdf` comes back `None` --
    /// that is a wait, not a failure.
    ///
    /// No /v3 equivalent yet.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call, never
    /// answers, or sends base64 that does not decode.
    pub fn sending_proof(&self, id: &str) -> LegacyResult<SendingProof> {
        let path = format!("/gw/sending-proof/{}", encode_segment(id));
        let wire: SendingProofWire = self.transport.json("GET", &path, &[])?;

        let Some(content) = wire.content else {
            return Ok(SendingProof {
                pdf: None,
                content_base64: None,
                message: wire.message,
            });
        };

        let Some(pdf) = base64::decode(&content) else {
            return Err(LegacyApiError {
                status: 200,
                http_status: 200,
                message: "o comprovante veio com base64 ilegível".to_owned(),
                body: Some(content),
            });
        };

        Ok(SendingProof {
            pdf: Some(pdf),
            content_base64: Some(content),
            message: None,
        })
    }

    /// The expert-evidence report -- the one route that answers the PDF binary
    /// directly, no base64, no JSON.
    ///
    /// No /v3 equivalent yet.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers. A missing record refuses with JSON even on this route.
    pub fn laudo(&self, id: &str) -> LegacyResult<Vec<u8>> {
        self.transport
            .binary(&format!("/gw/email/laudo/{}", encode_segment(id)))
    }

    /// Stops the notification ladder for this send.
    ///
    /// A GET with a side effect -- that is the old contract, and the SDK does
    /// not "fix" it to POST. A caller who saw a POST here would be integrating
    /// against a route that does not exist.
    ///
    /// No /v3 equivalent yet.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call or never
    /// answers.
    pub fn finalizar_regua(&self, id: &str) -> LegacyResult<FinalizarReguaResult> {
        self.transport.json(
            "GET",
            &format!("/regua-notificacao/finalizar/{}", encode_segment(id)),
            &[],
        )
    }
}

/// What the sending-proof route answers on the wire -- one of the two fields.
#[derive(Deserialize)]
struct SendingProofWire {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    message: Option<String>,
}
