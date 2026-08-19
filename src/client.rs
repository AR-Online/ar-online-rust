//! The client -- the one thing you construct.

use crate::http::transport::{Transport, DEFAULT_BASE_URL, DEFAULT_TIMEOUT};
use crate::legacy::{LegacyArea, LegacyTransport, DEFAULT_LEGACY_BASE_URL};
use crate::resources::{Allowlist, FreshnessResource, Tags, Templates, VersionResource};
use std::sync::Arc;
use std::time::Duration;

/// The AR Online API client.
///
/// ```no_run
/// use aronline::{Channel, Client};
///
/// # fn main() -> Result<(), aronline::ApiError> {
/// let client = Client::builder().token("meu-token").build();
/// let templates = client.templates.list(Some(Channel::WhatsApp))?;
/// # Ok(())
/// # }
/// ```
///
/// It owns the transport and hands it to each resource; the resources are the
/// public surface. Nothing above this line knows that HTTP is involved.
///
/// There are two surfaces, and one client speaks both: the /v3 resources are
/// the fields below, and the old gateway lives in [`legacy()`](Self::legacy).
/// Each has its own address and its own credential, and neither credential ever
/// travels to the other's address.
pub struct Client {
    /// Message templates.
    pub templates: Templates,
    /// Your labels.
    pub tags: Tags,
    /// Your allowed recipients.
    pub allowlist: Allowlist,
    /// How fresh the copy of the data is.
    pub freshness: FreshnessResource,
    /// Which version is running. The one route that needs no token.
    pub version: VersionResource,

    legacy: LegacyArea,
}

impl Client {
    /// Starts building a client.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// The legacy gateway: sending, status, proofs and the gateway's templates.
    ///
    /// A method rather than a field because it is a whole second API, not one
    /// more resource of this one -- see [`legacy`](crate::legacy) for what
    /// differs.
    #[must_use]
    pub fn legacy(&self) -> &LegacyArea {
        &self.legacy
    }
}

/// Builds a [`Client`].
///
/// The default builds a client pointed at production with no credential, which
/// is enough for [`VersionResource::get`].
///
/// Each credential is optional: give the one for the surface you are going to
/// use. A legacy call on a client built without [`legacy_token`] fails before
/// the socket, naming the method that is missing.
///
/// [`legacy_token`]: Self::legacy_token
pub struct ClientBuilder {
    base_url: String,
    token: Option<String>,
    legacy_base_url: String,
    legacy_token: Option<String>,
    timeout: Duration,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_owned(),
            token: None,
            legacy_base_url: DEFAULT_LEGACY_BASE_URL.to_owned(),
            legacy_token: None,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl ClientBuilder {
    /// The RS256 token issued by AR Online.
    ///
    /// Every route except `/v3/version` refuses with 401 without it.
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// The gateway's JWT -- the credential of the legacy area.
    ///
    /// A different credential from [`token`](Self::token), and it travels
    /// differently: raw in `authorization`, with no `Bearer`. Both live in the
    /// same client, and neither leaks into the other's calls.
    #[must_use]
    pub fn legacy_token(mut self, token: impl Into<String>) -> Self {
        self.legacy_token = Some(token.into());
        self
    }

    /// Where to call. Defaults to [`DEFAULT_BASE_URL`].
    ///
    /// [`DEFAULT_BASE_URL`]: crate::DEFAULT_BASE_URL
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Where the legacy gateway is. Defaults to [`DEFAULT_LEGACY_BASE_URL`],
    /// independently of the /v3 address.
    ///
    /// [`DEFAULT_LEGACY_BASE_URL`]: crate::legacy::DEFAULT_LEGACY_BASE_URL
    #[must_use]
    pub fn legacy_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.legacy_base_url = base_url.into();
        self
    }

    /// How long a call waits. Defaults to [`DEFAULT_TIMEOUT`], and applies to
    /// both surfaces.
    ///
    /// [`DEFAULT_TIMEOUT`]: crate::DEFAULT_TIMEOUT
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Builds the client.
    #[must_use]
    pub fn build(self) -> Client {
        let transport = Arc::new(Transport::new(&self.base_url, self.token, self.timeout));
        let legacy = Arc::new(LegacyTransport::new(
            &self.legacy_base_url,
            self.legacy_token,
            self.timeout,
        ));

        Client {
            templates: Templates::new(Arc::clone(&transport)),
            tags: Tags::new(Arc::clone(&transport)),
            allowlist: Allowlist::new(Arc::clone(&transport)),
            freshness: FreshnessResource::new(Arc::clone(&transport)),
            version: VersionResource::new(transport),
            legacy: LegacyArea::new(legacy),
        }
    }
}
