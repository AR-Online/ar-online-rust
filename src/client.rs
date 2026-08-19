//! The client -- the one thing you construct.

use crate::http::transport::{Transport, DEFAULT_BASE_URL, DEFAULT_TIMEOUT};
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
}

impl Client {
    /// Starts building a client.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}

/// Builds a [`Client`].
///
/// The default builds a client pointed at production with no credential, which
/// is enough for [`VersionResource::get`].
pub struct ClientBuilder {
    base_url: String,
    token: Option<String>,
    timeout: Duration,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_owned(),
            token: None,
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

    /// Where to call. Defaults to [`DEFAULT_BASE_URL`].
    ///
    /// [`DEFAULT_BASE_URL`]: crate::DEFAULT_BASE_URL
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// How long a call waits. Defaults to [`DEFAULT_TIMEOUT`].
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

        Client {
            templates: Templates::new(Arc::clone(&transport)),
            tags: Tags::new(Arc::clone(&transport)),
            allowlist: Allowlist::new(Arc::clone(&transport)),
            freshness: FreshnessResource::new(Arc::clone(&transport)),
            version: VersionResource::new(transport),
        }
    }
}
