//! The gateway's template routes.

use crate::http::transport::encode_segment;
use crate::legacy::error::LegacyResult;
use crate::legacy::models::{GwTemplate, GwTemplateType, GwTemplateWriteResult};
use crate::legacy::transport::LegacyTransport;
use serde::Serialize;
use std::sync::Arc;

/// What [`LegacyTemplates::update`] changes -- the two fields the gateway lets
/// you edit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGwTemplate {
    /// The new name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome: Option<String>,
    /// Whether everyone in the entity may use it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartilhado_com_entidade: Option<bool>,
}

/// The gateway's template routes.
///
/// The whole family answers through the `{ data, statusCode }` envelope with
/// **HTTP 200 even on error**; the transport unwraps it and turns the inner
/// 403/404/500 into a [`LegacyApiError`], so none of that reaches you.
///
/// The /v3 equivalent for the reads is [`Client::templates`] -- same database
/// row, clean contract. The writes have no /v3 equivalent yet.
///
/// The version routes (`/{id}/versions` and `/{id}/versions/{v}`) are
/// deliberately **not** here: production answers empty or 404 for every
/// template, and a function that never finds anything only invites an
/// integration against a dead resource.
///
/// [`LegacyApiError`]: crate::legacy::LegacyApiError
/// [`Client::templates`]: crate::Client::templates
pub struct LegacyTemplates {
    transport: Arc<LegacyTransport>,
}

impl LegacyTemplates {
    pub(crate) fn new(transport: Arc<LegacyTransport>) -> Self {
        Self { transport }
    }

    /// Your entity's templates and the ones shared with it, newest first.
    ///
    /// Pass `None` for every type.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call -- inside
    /// the envelope or outside it -- or never answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn list(&self, kind: Option<GwTemplateType>) -> LegacyResult<Vec<GwTemplate>> {
        match kind {
            Some(kind) => {
                self.transport
                    .envelope("GET", "/gw/templates", &[("type", kind.as_str())])
            }
            None => self.transport.envelope("GET", "/gw/templates", &[]),
        }
    }

    /// One template by its public UUID. Someone else's answers the family's 403.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call -- inside
    /// the envelope or outside it -- or never answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn get(&self, id: &str) -> LegacyResult<GwTemplate> {
        self.transport
            .envelope("GET", &format!("/gw/templates/{}", encode_segment(id)), &[])
    }

    /// Edits name and entity-wide sharing -- the two things the gateway lets
    /// you touch.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call -- inside
    /// the envelope or outside it -- or never answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn update(
        &self,
        id: &str,
        changes: &UpdateGwTemplate,
    ) -> LegacyResult<GwTemplateWriteResult> {
        self.transport.envelope_body(
            "PUT",
            &format!("/gw/templates/{}", encode_segment(id)),
            changes,
        )
    }

    /// Soft delete: the template deactivates, the row stays.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call -- inside
    /// the envelope or outside it -- or never answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn deactivate(&self, id: &str) -> LegacyResult<GwTemplateWriteResult> {
        self.transport.envelope(
            "DELETE",
            &format!("/gw/templates/{}", encode_segment(id)),
            &[],
        )
    }

    /// Turns a template on or off without deleting anything.
    ///
    /// # Errors
    ///
    /// Returns [`LegacyApiError`] when the gateway refuses the call -- inside
    /// the envelope or outside it -- or never answers.
    ///
    /// [`LegacyApiError`]: crate::legacy::LegacyApiError
    pub fn set_status(&self, id: &str, ativo: bool) -> LegacyResult<GwTemplateWriteResult> {
        self.transport.envelope_body(
            "PATCH",
            &format!("/gw/templates/{}/status", encode_segment(id)),
            &SetStatus { ativo },
        )
    }
}

#[derive(Serialize)]
struct SetStatus {
    ativo: bool,
}
