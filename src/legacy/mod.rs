//! The legacy gateway of AR Online, as typed functions.
//!
//! Everything the public documentation of `api.ar-online.com.br` describes:
//! sending, per-channel status, the proofs and the gateway's templates. It is a
//! second surface inside the same client -- `client.legacy()` -- with its own
//! address, its own credential and its own error type, because it is a
//! different API and pretending otherwise would leak one contract's habits into
//! the other.
//!
//! Three things work differently from /v3, and all three are the old contract
//! rather than a defect:
//!
//! - the credential is the gateway's JWT, sent **raw** in `authorization` with
//!   no `Bearer`;
//! - success is not the HTTP status: the templates family answers 200 carrying
//!   the real code inside the body, and the voice status answers 200 with a
//!   sentence where the other channels answer 404;
//! - a date is a `String` like `"18/07/2026 01:01:32"`, with no offset. It does
//!   not name an unambiguous instant, so the SDK does not pretend it does.
//!
//! The names here are the legacy vocabulary -- `laudo`, `regua`,
//! `sending_proof`, `voz`, `carta`. That is a deliberate exception to the
//! project's English rule: translating them would invent names that appear in
//! no documentation anywhere.

mod area;
mod base64;
mod error;
mod models;
mod resources;
mod transport;

pub use area::{FinalizarReguaResult, LegacyArea};
pub use error::{LegacyApiError, LegacyResult};
pub use models::{
    Anexo, CanalCarta, CanalSms, CanalVoz, CanalWhatsapp, EnvioRequest, EnvioResponse,
    EnvioValidation, FullChannelDetail, FullHistory, FullLastStatus, GwTemplate, GwTemplateType,
    GwTemplateWriteResult, LegacyField, SendingProof, SmsAnswer, SmsTypeSend, StatusCarta,
    StatusEmail, StatusEvent, StatusFull, StatusSms, StatusVoz, StatusWhatsapp, WebhookChannel,
    WebhookMetadata, WebhookPayloadV1, WebhookPayloadV2,
};
pub use resources::{LegacyStatus, LegacyTemplates, UpdateGwTemplate};
pub use transport::DEFAULT_LEGACY_BASE_URL;

pub(crate) use transport::LegacyTransport;
