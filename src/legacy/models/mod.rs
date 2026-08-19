//! The shapes the legacy gateway answers, and the one it takes.

mod envio;
mod field;
mod full;
mod gw_template;
mod sending_proof;
mod status;
mod webhook;

pub use envio::{
    Anexo, CanalCarta, CanalSms, CanalVoz, CanalWhatsapp, EnvioRequest, EnvioResponse,
    EnvioValidation, SmsTypeSend,
};
pub use field::LegacyField;
pub use full::{FullChannelDetail, FullHistory, FullLastStatus, StatusEvent, StatusFull};
pub use gw_template::{GwTemplate, GwTemplateType, GwTemplateWriteResult};
pub use sending_proof::SendingProof;
pub use status::{SmsAnswer, StatusCarta, StatusEmail, StatusSms, StatusVoz, StatusWhatsapp};
pub use webhook::{WebhookChannel, WebhookMetadata, WebhookPayloadV1, WebhookPayloadV2};
