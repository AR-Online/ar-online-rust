//! The only part of the legacy area that knows HTTP exists.

use super::error::{LegacyApiError, LegacyResult};
use crate::http::transport::encode_segment;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use ureq::{AsSendBody, Body};

/// Where the legacy gateway lives. Override it for staging or for a local
/// process; it is independent of the /v3 address.
pub const DEFAULT_LEGACY_BASE_URL: &str = "https://api.ar-online.com.br";

/// Builds the request, reads the answer, and decides whether it was a refusal.
///
/// Two things make this a different transport from the /v3 one rather than a
/// flag on it: the credential goes **raw** in `authorization` -- no `Bearer`
/// -- and success is not the HTTP status. The templates family answers 200
/// with the real code inside the body, and the voice status answers 200 with a
/// sentence where the other channels answer 404. Each method here is picked by
/// the contract of the route that calls it.
pub(crate) struct LegacyTransport {
    base_url: String,
    token: Option<String>,
    agent: ureq::Agent,
}

impl LegacyTransport {
    pub(crate) fn new(base_url: &str, token: Option<String>, timeout: Duration) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            // 4xx e 5xx são RESPOSTAS, não falhas de transporte: é no corpo
            // que vem a frase do gateway, que é o que quem chamou precisa ler.
            .http_status_as_error(false)
            .build();

        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            token,
            agent: config.into(),
        }
    }

    /// For the routes that answer JSON and refuse with an HTTP status.
    pub(crate) fn json<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
    ) -> LegacyResult<T> {
        let (status, text) = self.exchange(method, path, query, None)?;

        decode(path, status, &text)
    }

    /// The same, for the one route that sends a JSON body.
    pub(crate) fn json_body<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &str,
        path: &str,
        body: &B,
    ) -> LegacyResult<T> {
        let (status, text) = self.exchange(method, path, &[], Some(encode_body(path, body)?))?;

        decode(path, status, &text)
    }

    /// For the `/gw/templates` family, which wraps everything in
    /// `{ data, statusCode }` and answers **HTTP 200 even on error**.
    ///
    /// The code that matters is the inner one. Reading only the HTTP status is
    /// the single most common bug in integrations against this family, and
    /// unwrapping it here is exactly what the legacy area is for.
    pub(crate) fn envelope<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
    ) -> LegacyResult<T> {
        let (status, text) = self.exchange(method, path, query, None)?;

        unwrap_envelope(path, status, &text)
    }

    /// The same, for the two write routes that send a JSON body.
    pub(crate) fn envelope_body<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &str,
        path: &str,
        body: &B,
    ) -> LegacyResult<T> {
        let (status, text) = self.exchange(method, path, &[], Some(encode_body(path, body)?))?;

        unwrap_envelope(path, status, &text)
    }

    /// For the one route that answers a file instead of JSON -- the laudo.
    pub(crate) fn binary(&self, path: &str) -> LegacyResult<Vec<u8>> {
        let mut response = self.send("GET", path, &[], None)?;
        let status = response.status().as_u16();

        if status >= 400 {
            // A 404 on this route answers JSON, so the text is worth reading.
            let text = response.body_mut().read_to_string().unwrap_or_default();

            return Err(refusal(status, text));
        }

        response.body_mut().read_to_vec().map_err(|failure| {
            LegacyApiError::local(
                status,
                status,
                format!("a resposta de {path} foi interrompida no meio: {failure}"),
            )
        })
    }

    /// The absolute URL of a path. Crate-visible because the tests assert on it.
    pub(crate) fn url(&self, path: &str, query: &[(&str, &str)]) -> String {
        let mut url = format!("{}{path}", self.base_url);

        for (index, (key, value)) in query.iter().enumerate() {
            url.push(if index == 0 { '?' } else { '&' });
            url.push_str(key);
            url.push('=');
            url.push_str(&encode_segment(value));
        }

        url
    }

    fn exchange(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: Option<String>,
    ) -> LegacyResult<(u16, String)> {
        let mut response = self.send(method, path, query, body)?;
        let status = response.status().as_u16();

        let text = response.body_mut().read_to_string().map_err(|failure| {
            LegacyApiError::local(
                status,
                status,
                format!("a resposta de {path} foi interrompida no meio: {failure}"),
            )
        })?;

        if status >= 400 {
            return Err(refusal(status, text));
        }

        Ok((status, text))
    }

    fn send(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: Option<String>,
    ) -> LegacyResult<ureq::http::Response<Body>> {
        // Refused here, before the socket: a 401 round trip teaches nothing
        // that the missing token does not already say. Every legacy route
        // needs the credential -- the gateway has no open route.
        let Some(token) = self.token.as_deref() else {
            let hint = "construa o cliente com Client::builder().legacy_token(…)";

            return Err(LegacyApiError::local(
                401,
                0,
                format!("{path} exige o token do gateway; {hint}"),
            ));
        };

        let builder = ureq::http::Request::builder()
            .method(method)
            .uri(self.url(path, query))
            .header("Accept", "application/json")
            // Cru de propósito: o gateway quer o JWT sem `Bearer`, o oposto da
            // /v3. Prefixar aqui transformaria toda chamada no 401 dele.
            .header("authorization", token);

        match body {
            Some(json) => self.dispatch(builder.header("Content-Type", "application/json"), json),
            // `()` e não uma String vazia: a String faria o ureq mandar
            // `Content-Length: 0` num GET, que é corpo onde não há corpo.
            None => self.dispatch(builder, ()),
        }
    }

    fn dispatch<B: AsSendBody>(
        &self,
        builder: ureq::http::request::Builder,
        body: B,
    ) -> LegacyResult<ureq::http::Response<Body>> {
        let request = builder.body(body).map_err(|failure| {
            LegacyApiError::local(0, 0, format!("não consegui montar a chamada: {failure}"))
        })?;

        // Timeout e conexão recusada chegam como falha opaca de transporte, e
        // viram o erro do SDK para quem chamou ter um tipo só. Status recusado
        // nunca cai aqui — o agente entrega status como resposta.
        self.agent.run(request).map_err(|failure| {
            LegacyApiError::local(
                0,
                0,
                format!("não foi possível falar com {}: {failure}", self.base_url),
            )
        })
    }
}

/// Parses a 2xx body into the shape the route promises.
fn decode<T: DeserializeOwned>(path: &str, status: u16, text: &str) -> LegacyResult<T> {
    // A 200 that is not the JSON the route promises is something other than
    // the gateway answering. A raw serde error leaking out here would send
    // whoever hit it looking for a bug in their own code.
    serde_json::from_str(text).map_err(|failure| LegacyApiError {
        status,
        http_status: status,
        message: format!("a resposta de {path} não é o JSON que a rota promete: {failure}"),
        body: Some(text.to_owned()),
    })
}

/// Reads the `{ data, statusCode }` envelope, inner code and all.
fn unwrap_envelope<T: DeserializeOwned>(path: &str, status: u16, text: &str) -> LegacyResult<T> {
    let parsed: Value = decode(path, status, text)?;

    let Value::Object(mut envelope) = parsed else {
        return Err(missing_envelope(path, status, text));
    };

    let Some(data) = envelope.remove("data") else {
        return Err(missing_envelope(path, status, text));
    };

    let inner = envelope
        .get("statusCode")
        .and_then(Value::as_u64)
        .and_then(|code| u16::try_from(code).ok())
        .unwrap_or(status);

    if inner >= 400 {
        return Err(LegacyApiError {
            status: inner,
            http_status: status,
            message: envelope_error(&data)
                .unwrap_or_else(|| format!("o gateway recusou com {inner}")),
            body: Some(text.to_owned()),
        });
    }

    serde_json::from_value(data).map_err(|failure| LegacyApiError {
        status: inner,
        http_status: status,
        message: format!("o `data` de {path} não tem a forma que a rota promete: {failure}"),
        body: Some(text.to_owned()),
    })
}

fn missing_envelope(path: &str, status: u16, text: &str) -> LegacyApiError {
    LegacyApiError {
        status,
        http_status: status,
        message: format!(
            "{path} respondeu sem o envelope {{ data, statusCode }} que a família promete"
        ),
        body: Some(text.to_owned()),
    }
}

// The family's error sentence sits inside `data`: `{ "error": "Template não
// encontrado" }`. Anything else stays raw on the error, for the caller.
fn envelope_error(data: &Value) -> Option<String> {
    sentence(data.get("error")?)
}

/// One sentence out of an error field, whichever of its two shapes came.
///
/// The gateway usually answers `"message": "uma frase"`, but when the
/// validation layer refuses several fields at once it answers a **list** of
/// sentences. Reading only the string shape would drop the most informative
/// refusal the API produces -- and, worse, would drop the `statusCode` sitting
/// next to it.
fn sentence(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => {
            let lines: Vec<&str> = items.iter().filter_map(Value::as_str).collect();

            if lines.is_empty() {
                None
            } else {
                Some(lines.join("; "))
            }
        }
        _ => None,
    }
}

/// An HTTP-level refusal -- `{ statusCode, message }`, or whatever a proxy sent.
fn refusal(http_status: u16, body: String) -> LegacyApiError {
    let parsed: Option<Value> = serde_json::from_str(&body).ok();

    let status = parsed
        .as_ref()
        .and_then(|value| value.get("statusCode"))
        .and_then(Value::as_u64)
        .and_then(|code| u16::try_from(code).ok())
        .unwrap_or(http_status);

    let message = parsed
        .as_ref()
        .and_then(|value| value.get("message"))
        .and_then(sentence)
        .unwrap_or_else(|| {
            format!("o gateway respondeu {http_status} sem o corpo de erro esperado")
        });

    LegacyApiError {
        status,
        http_status,
        message,
        body: Some(body),
    }
}

fn encode_body<B: Serialize>(path: &str, body: &B) -> LegacyResult<String> {
    serde_json::to_string(body).map_err(|failure| {
        LegacyApiError::local(
            0,
            0,
            format!("não consegui montar o corpo de {path}: {failure}"),
        )
    })
}
