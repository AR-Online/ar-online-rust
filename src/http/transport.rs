//! The only part of the SDK that knows HTTP exists.

use crate::error::{ApiError, ErrorDetail, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt::Write as _;
use std::time::Duration;

/// Where /v3 lives. Override it for staging or for a local process.
pub const DEFAULT_BASE_URL: &str = "https://v3.ar-online.com.br";

/// How long a call waits before giving up.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Builds the request, reads the answer, and returns what went wrong.
///
/// Everything above it -- the resources, the client -- deals in methods and
/// structs. That is the whole point of the package: whoever installs it should
/// never build a URL, set a header, read a status or unwrap an envelope.
pub(crate) struct Transport {
    base_url: String,
    token: Option<String>,
    agent: ureq::Agent,
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

#[derive(Deserialize, Default)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Deserialize, Default)]
struct ErrorBody {
    code: Option<String>,
    message: Option<String>,
    request_id: Option<String>,
    field: Option<String>,
    #[serde(default)]
    details: Vec<ErrorDetail>,
}

impl Transport {
    pub(crate) fn new(base_url: &str, token: Option<String>, timeout: Duration) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            // 4xx e 5xx são RESPOSTAS, não falhas de transporte. O padrão do
            // ureq é transformá-las em Err antes de entregar o corpo — e é no
            // corpo que vem o catálogo (`code`, `message`, `request_id`), que é
            // exatamente o que quem chamou precisa.
            .http_status_as_error(false)
            .build();

        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            token,
            agent: config.into(),
        }
    }

    /// For the routes that answer `{"data": …}` -- templates, tags, allowlist.
    ///
    /// Not every route wraps its answer, which is the one trap of this API:
    /// freshness and version answer the object itself. Unwrapping all of them,
    /// or none, breaks half the calls, so the choice is made per route by
    /// whichever method the resource calls.
    pub(crate) fn envelope<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let body: Envelope<T> = self.request(path, query, true).map_err(|failure| {
            if failure.code == "invalid_response" && failure.status == 200 {
                ApiError::local(
                    200,
                    "invalid_response",
                    format!("{path} respondeu sem o envelope 'data' que a rota promete"),
                )
            } else {
                failure
            }
        })?;

        Ok(body.data)
    }

    /// For the routes that answer the object directly -- freshness, version.
    pub(crate) fn bare<T: DeserializeOwned>(&self, path: &str, authenticated: bool) -> Result<T> {
        self.request(path, &[], authenticated)
    }

    /// The absolute URL of a path. Crate-visible because the tests assert on it.
    pub(crate) fn url(&self, path: &str, query: &[(&str, &str)]) -> String {
        let mut url = format!("{}{path}", self.base_url);

        for (index, (key, value)) in query.iter().enumerate() {
            url.push(if index == 0 { '?' } else { '&' });
            url.push_str(key);
            url.push('=');
            url.push_str(&encode(value));
        }

        url
    }

    fn request<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<T> {
        // Refused here, before the socket: a 401 round trip teaches nothing
        // that the missing token does not already say.
        if authenticated && self.token.is_none() {
            let hint = "construa o cliente com Client::builder().token(…)";

            return Err(ApiError::local(
                401,
                "unauthenticated",
                format!("{path} exige um token; {hint}"),
            ));
        }

        let token = if authenticated {
            self.token.as_deref()
        } else {
            None
        };

        let mut request = self
            .agent
            .get(self.url(path, query))
            .header("Accept", "application/json");

        if let Some(token) = token {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }

        let mut response = match request.call() {
            Ok(response) => response,
            // Timeout and connection refused arrive as opaque transport
            // errors. They become the SDK's own error so a caller has one type
            // to match on instead of several. A refused STATUS never lands
            // here — the agent is configured to deliver it as a response.
            Err(failure) => {
                return Err(ApiError::local(
                    0,
                    "unreachable",
                    format!("não foi possível falar com {}: {failure}", self.base_url),
                ))
            }
        };

        let status = response.status().as_u16();
        let request_id = header(&response, "x-request-id");
        let retry_after = header(&response, "retry-after");

        let text = response.body_mut().read_to_string().map_err(|failure| {
            ApiError::local(
                status,
                "unreachable",
                format!("a resposta de {path} foi interrompida no meio: {failure}"),
            )
        })?;

        if status >= 400 {
            return Err(to_api_error(
                status,
                &text,
                request_id,
                retry_after.as_deref(),
            ));
        }

        // A 2xx that is not the JSON the route promises is something other
        // than the API answering. It fails like any other refusal -- a raw
        // serde error leaking out here would send whoever hit it looking for a
        // bug in their own code.
        serde_json::from_str(&text).map_err(|_| ApiError {
            status,
            code: "invalid_response".to_owned(),
            message: "a resposta não é o JSON que a rota promete — algo respondeu no lugar da API"
                .to_owned(),
            request_id,
            field: None,
            details: Vec::new(),
            retry_after_seconds: None,
        })
    }
}

/// Turns a refused response into an [`ApiError`].
///
/// A refusal that does not carry the envelope is still a refusal: a proxy
/// answering 502 in HTML has to fail the same way, or whoever hit it goes
/// looking for a bug in their own parsing code.
fn to_api_error(
    status: u16,
    body: &str,
    request_id: Option<String>,
    retry_after: Option<&str>,
) -> ApiError {
    let envelope: ErrorEnvelope = serde_json::from_str(body).unwrap_or_default();

    ApiError {
        status,
        code: envelope
            .error
            .code
            .unwrap_or_else(|| "invalid_response".to_owned()),
        message: envelope
            .error
            .message
            .unwrap_or_else(|| format!("a API respondeu {status} sem o corpo de erro esperado")),
        request_id: envelope.error.request_id.or(request_id),
        field: envelope.error.field,
        details: envelope.error.details,
        retry_after_seconds: parse_retry_after(retry_after),
    }
}

/// Seconds, or nothing.
///
/// `Retry-After` also has an HTTP-date form. The API only ever sends seconds,
/// so anything else is dropped rather than guessed at: a wrong delay is worse
/// than no delay.
fn parse_retry_after(header: Option<&str>) -> Option<f64> {
    header?
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|seconds| *seconds >= 0.0)
}

fn header<T>(response: &ureq::http::Response<T>, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

/// Percent-encodes a query value.
///
/// Small on purpose: the SDK only ever sends the closed channel list, and a
/// whole encoding crate for five known words is a dependency nobody asked for.
fn encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());

    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            // write! em vez de push_str(&format!(…)): o format! alocaria uma
            // String por byte escapado, só para copiá-la e jogá-la fora.
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }

    encoded
}

/// Percent-encodes a path segment. Same rules, and the same reason.
pub(crate) fn encode_segment(value: &str) -> String {
    encode(value)
}
