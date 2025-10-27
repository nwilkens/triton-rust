//! Asynchronous CNAPI client implementation.

use crate::models::{Server, ServerListParams, UpdateServerRequest};
use crate::Result;
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};
use triton_core::client::{
    ClientConfig, RetryPolicy, CNAPI_DEFAULT_TIMEOUT, DEFAULT_POOL_IDLE_TIMEOUT,
    DEFAULT_POOL_MAX_IDLE_PER_HOST,
};
use triton_core::services::{DiscoveryStatus, ServiceDiscovery};
use triton_core::types::TritonService;
use triton_core::uuid::ServerUuid;
use triton_core::Error;
use url::Url;

const USER_AGENT: &str = concat!("triton-cnapi/", env!("CARGO_PKG_VERSION"));

/// Builder for [`CnapiClient`].
#[derive(Debug, Clone)]
pub struct CnapiClientBuilder {
    base_url: Url,
    http_config: ClientConfig,
    retry_policy: RetryPolicy,
    basic_auth: Option<(String, String)>,
    token: Option<String>,
}

impl CnapiClientBuilder {
    /// Create a new builder with the provided CNAPI base URL.
    ///
    /// The URL should include the protocol and hostname (e.g. `https://cnapi.example.com`).
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let url = Url::parse(base_url.as_ref()).map_err(|err| {
            Error::ConfigError(format!(
                "Invalid CNAPI base URL `{}`: {err}",
                base_url.as_ref()
            ))
        })?;

        let client_config = ClientConfig::new()
            .with_timeout(Duration::from_secs(CNAPI_DEFAULT_TIMEOUT))
            .with_pool_idle_timeout(Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT))
            .with_pool_max_idle(DEFAULT_POOL_MAX_IDLE_PER_HOST);

        Ok(Self {
            base_url: url,
            retry_policy: client_config.retry_policy,
            http_config: client_config,
            basic_auth: None,
            token: None,
        })
    }

    /// Override the retry policy.
    #[must_use]
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// Override the HTTP client configuration.
    #[must_use]
    pub fn with_http_config(mut self, config: ClientConfig) -> Self {
        self.http_config = config.clone();
        self.retry_policy = config.retry_policy;
        self
    }

    /// Configure HTTP basic authentication.
    #[must_use]
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.basic_auth = Some((username.into(), password.into()));
        self
    }

    /// Configure token based authentication (sent as `X-Auth-Token`).
    #[must_use]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Build the CNAPI client.
    pub fn build(self) -> Result<CnapiClient> {
        let mut builder = ClientBuilder::new()
            .timeout(self.http_config.timeout)
            .user_agent(USER_AGENT)
            .pool_idle_timeout(self.http_config.pool_idle_timeout)
            .pool_max_idle_per_host(self.http_config.pool_max_idle_per_host)
            .connect_timeout(Duration::from_secs(10));

        if !self.http_config.enable_compression {
            builder = builder.no_gzip();
        }

        let http = builder.build().map_err(|err| {
            Error::ConfigError(format!("Failed to build CNAPI HTTP client: {err}"))
        })?;

        Ok(CnapiClient {
            http,
            base_url: self.base_url,
            retry_policy: self.retry_policy,
            basic_auth: self.basic_auth,
            token: self.token,
        })
    }
}

/// Asynchronous CNAPI client.
#[derive(Clone)]
pub struct CnapiClient {
    http: Client,
    base_url: Url,
    retry_policy: RetryPolicy,
    basic_auth: Option<(String, String)>,
    token: Option<String>,
}

impl CnapiClient {
    /// Create a new client for the given base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        CnapiClientBuilder::new(base_url)?.build()
    }

    /// Access the underlying base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// List compute nodes using the provided filter parameters.
    pub async fn list_servers(&self, params: &ServerListParams) -> Result<Vec<Server>> {
        let query = params.to_pairs();
        self.get_json("servers", &query).await
    }

    /// Fetch a single server by UUID.
    pub async fn get_server(&self, uuid: ServerUuid) -> Result<Server> {
        let path = format!("servers/{uuid}");
        self.get_json(&path, &[]).await
    }

    /// Update mutable server properties.
    pub async fn update_server(
        &self,
        uuid: ServerUuid,
        request: &UpdateServerRequest,
    ) -> Result<Server> {
        let path = format!("servers/{uuid}");
        self.send_json::<_, Server>(Method::PUT, &path, Some(request), &[])
            .await
    }

    fn build_url(&self, path: &str) -> Result<Url> {
        let normalized = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };

        self.base_url
            .join(normalized)
            .map_err(|err| Error::InvalidEndpoint(format!("Invalid CNAPI path `{path}`: {err}")))
    }

    async fn get_json<T>(&self, path: &str, params: &[(&'static str, String)]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send_json(Method::GET, path, Option::<&()>::None, params)
            .await
    }

    async fn send_json<B, R>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        params: &[(&'static str, String)],
    ) -> Result<R>
    where
        B: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        #[allow(unused_assignments)]
        let mut last_error: Option<Error> = None;
        let mut attempt = 0;

        loop {
            let url = self.build_url(path)?;
            let mut request = self.http.request(method.clone(), url).query(params);

            if let Some((user, pass)) = &self.basic_auth {
                request = request.basic_auth(user, Some(pass));
            }
            if let Some(token) = &self.token {
                request = request.header("X-Auth-Token", token);
            }
            request = request.header("Accept", "application/json");

            if let Some(payload) = body {
                request = request.json(payload);
            }

            info!(path, attempt, "CNAPI request");

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return response.json::<R>().await.map_err(|err| {
                            Error::SapiParseError(format!(
                                "Failed to parse CNAPI response for `{path}`: {err}"
                            ))
                        });
                    }

                    let text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());

                    let error = match status {
                        StatusCode::NOT_FOUND => return Err(Error::NotFound(text)),
                        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                            Error::InvalidRequest(format!("CNAPI authentication failed: {text}"))
                        }
                        StatusCode::TOO_MANY_REQUESTS
                        | StatusCode::BAD_GATEWAY
                        | StatusCode::SERVICE_UNAVAILABLE
                        | StatusCode::GATEWAY_TIMEOUT => Error::ServiceUnavailable(format!(
                            "CNAPI temporarily unavailable: {text}"
                        )),
                        status if status.is_server_error() => Error::ServiceUnavailable(format!(
                            "CNAPI server error {status}: {text}"
                        )),
                        _ => Error::HttpError(format!("CNAPI error {status}: {text}")),
                    };
                    last_error = Some(error);
                }
                Err(err) => {
                    let error = Error::from(err);
                    if matches!(
                        error,
                        Error::Timeout(_) | Error::ServiceUnavailable(_) | Error::HttpError(_)
                    ) {
                        last_error = Some(error);
                    } else {
                        return Err(error);
                    }
                }
            }

            attempt += 1;
            if attempt > self.retry_policy.max_retries {
                break;
            }
            let delay = self.retry_policy.delay_for_attempt(attempt);
            if delay > Duration::from_millis(0) {
                debug!("Retrying CNAPI request after {:?}", delay);
                sleep(delay).await;
            }
        }

        if let Some(error) = last_error {
            Err(error)
        } else {
            Err(Error::ServiceUnavailable(
                "CNAPI request failed after retries".to_string(),
            ))
        }
    }
}

/// SAPI-backed discovery placeholder for CNAPI (leverages existing SAPI client).
///
/// This struct allows consumers to plug CNAPI discovery into the shared trait while we still rely
/// on SAPI for endpoint lookups.
pub struct CnapiDiscovery {
    sapi: Arc<dyn ServiceDiscovery>,
    status: Arc<RwLock<DiscoveryStatus>>,
}

impl CnapiDiscovery {
    /// Create from an existing `ServiceDiscovery` (typically `SapiDiscovery`).
    #[must_use]
    pub fn new(sapi_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            sapi: sapi_discovery,
            status: Arc::new(RwLock::new(DiscoveryStatus::new())),
        }
    }
}

#[async_trait]
impl ServiceDiscovery for CnapiDiscovery {
    async fn discover_service(&self, service_name: &str) -> Result<Vec<String>> {
        let start = std::time::Instant::now();

        match self.sapi.discover_service(service_name).await {
            Ok(endpoints) => {
                self.record_success(service_name, endpoints.len());
                debug!(
                    "CNAPI discovery fetched {} endpoints for {} in {:?}",
                    endpoints.len(),
                    service_name,
                    start.elapsed()
                );
                Ok(endpoints)
            }
            Err(err) => {
                self.record_error(service_name, Some(&err));
                Err(err)
            }
        }
    }

    async fn discover_all_services(&self) -> Result<Vec<String>> {
        self.sapi
            .discover_service(TritonService::Cnapi.name())
            .await
    }

    fn get_status(&self) -> DiscoveryStatus {
        self.status
            .read()
            .map(|status| status.clone())
            .unwrap_or_else(|_| DiscoveryStatus::new())
    }

    fn clear_cache(&self) {
        if let Ok(mut status) = self.status.write() {
            status.cache_hits = 0;
            status.cache_misses = 0;
        }
    }
}

impl CnapiDiscovery {
    fn record_success(&self, service: &str, count: usize) {
        if let Ok(mut status) = self.status.write() {
            let mut updated = status.clone().with_success(count);
            updated.failed_services.retain(|s| s != service);
            *status = updated;
        }
    }

    fn record_error(&self, service: &str, error: Option<&Error>) {
        if let Ok(mut status) = self.status.write() {
            status.last_discovery_at = Some(std::time::Instant::now());
            if let Some(error) = error {
                status.last_error = Some(error.to_string());
            }
            if !status.failed_services.iter().any(|s| s == service) {
                status.failed_services.push(service.to_string());
            }
        }
    }
}

/// Convenient alias for server list queries.
pub type ServerQuery = ServerListParams;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> CnapiClient {
        CnapiClient::new(server.uri()).unwrap()
    }

    #[tokio::test]
    async fn list_servers_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/servers"))
            .and(query_param("setup", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "uuid": ServerUuid::new_v4(),
                    "hostname": "cn01",
                    "setup": true
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server);
        let params = ServerListParams {
            setup: Some(true),
            ..ServerListParams::default()
        };

        let servers = client.list_servers(&params).await.unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].hostname.as_deref(), Some("cn01"));
    }

    #[tokio::test]
    async fn get_server_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/servers/11111111-1111-1111-1111-111111111111"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server);
        let uuid = ServerUuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let err = client.get_server(uuid).await.unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn update_server_success() {
        let server = MockServer::start().await;
        let uuid = ServerUuid::new_v4();

        Mock::given(method("PUT"))
            .and(path(format!("/servers/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "uuid": uuid,
                "reserved": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server);
        let request = UpdateServerRequest {
            reserved: Some(true),
            reservation_ratio: None,
            overprovision_ratio: None,
            comments: None,
            traits: None,
        };

        let server = client.update_server(uuid, &request).await.unwrap();
        assert_eq!(server.reserved, Some(true));
    }

    #[tokio::test]
    async fn discovery_delegates_to_sapi() {
        struct MockDiscovery;

        #[async_trait]
        impl ServiceDiscovery for MockDiscovery {
            async fn discover_service(&self, service_name: &str) -> Result<Vec<String>> {
                assert_eq!(service_name, "cnapi");
                Ok(vec!["http://cnapi.local:80".into()])
            }

            async fn discover_all_services(&self) -> Result<Vec<String>> {
                Ok(vec![])
            }

            fn get_status(&self) -> DiscoveryStatus {
                DiscoveryStatus::new()
            }

            fn clear_cache(&self) {}
        }

        let discovery = CnapiDiscovery::new(Arc::new(MockDiscovery));
        let endpoints = discovery.discover_service("cnapi").await.unwrap();
        assert_eq!(endpoints, vec!["http://cnapi.local:80"]);
    }
}
