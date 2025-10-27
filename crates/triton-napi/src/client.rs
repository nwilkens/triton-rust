//! Asynchronous NAPI client implementation.

use crate::models::{
    CreateNetworkRequest, Network, NetworkListParams, NetworkPool, Nic, UpdateNetworkRequest,
};
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
    ClientConfig, RetryPolicy, DEFAULT_POOL_IDLE_TIMEOUT, DEFAULT_POOL_MAX_IDLE_PER_HOST,
    NAPI_DEFAULT_TIMEOUT,
};
use triton_core::services::{DiscoveryStatus, ServiceDiscovery};
use triton_core::types::TritonService;
use triton_core::uuid::NetworkUuid;
use triton_core::Error;
use url::Url;

const USER_AGENT: &str = concat!("triton-napi/", env!("CARGO_PKG_VERSION"));

/// Builder for [`NapiClient`].
#[derive(Debug, Clone)]
pub struct NapiClientBuilder {
    base_url: Url,
    http_config: ClientConfig,
    retry_policy: RetryPolicy,
    basic_auth: Option<(String, String)>,
    token: Option<String>,
}

impl NapiClientBuilder {
    /// Create a new builder from the provided base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let url = Url::parse(base_url.as_ref()).map_err(|err| {
            Error::ConfigError(format!(
                "Invalid NAPI base URL `{}`: {err}",
                base_url.as_ref()
            ))
        })?;

        let config = ClientConfig::new()
            .with_timeout(Duration::from_secs(NAPI_DEFAULT_TIMEOUT))
            .with_pool_idle_timeout(Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT))
            .with_pool_max_idle(DEFAULT_POOL_MAX_IDLE_PER_HOST);

        Ok(Self {
            base_url: url,
            retry_policy: config.retry_policy,
            http_config: config,
            basic_auth: None,
            token: None,
        })
    }

    /// Override the retry policy.
    #[must_use]
    pub fn with_retry_policy(mut self, retry: RetryPolicy) -> Self {
        self.retry_policy = retry;
        self
    }

    /// Override the HTTP client configuration.
    #[must_use]
    pub fn with_http_config(mut self, config: ClientConfig) -> Self {
        self.retry_policy = config.retry_policy;
        self.http_config = config;
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

    /// Configure the `X-Auth-Token` header.
    #[must_use]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Build the client instance.
    pub fn build(self) -> Result<NapiClient> {
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
            Error::ConfigError(format!("Failed to build NAPI HTTP client: {err}"))
        })?;

        Ok(NapiClient {
            http,
            base_url: self.base_url,
            retry_policy: self.retry_policy,
            basic_auth: self.basic_auth,
            token: self.token,
        })
    }
}

/// Asynchronous client for NAPI.
#[derive(Clone)]
pub struct NapiClient {
    http: Client,
    base_url: Url,
    retry_policy: RetryPolicy,
    basic_auth: Option<(String, String)>,
    token: Option<String>,
}

impl NapiClient {
    /// Construct directly from a base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        NapiClientBuilder::new(base_url)?.build()
    }

    /// Access the base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// List networks.
    pub async fn list_networks(&self, params: &NetworkListParams) -> Result<Vec<Network>> {
        self.get_json("networks", &params.to_pairs()).await
    }

    /// Fetch a network by UUID.
    pub async fn get_network(&self, uuid: NetworkUuid) -> Result<Network> {
        let path = format!("networks/{uuid}");
        self.get_json(&path, &[]).await
    }

    /// Create a new network.
    pub async fn create_network(&self, request: &CreateNetworkRequest) -> Result<Network> {
        self.send_json(Method::POST, "networks", Some(request), &[])
            .await
    }

    /// Update an existing network.
    pub async fn update_network(
        &self,
        uuid: NetworkUuid,
        request: &UpdateNetworkRequest,
    ) -> Result<Network> {
        let path = format!("networks/{uuid}");
        self.send_json(Method::PUT, &path, Some(request), &[]).await
    }

    /// Delete a network.
    pub async fn delete_network(&self, uuid: NetworkUuid) -> Result<()> {
        let path = format!("networks/{uuid}");
        self.send_empty(Method::DELETE, &path, &[]).await
    }

    /// List network pools.
    pub async fn list_network_pools(&self) -> Result<Vec<NetworkPool>> {
        self.get_json("network_pools", &[]).await
    }

    /// Fetch a specific network pool by UUID.
    pub async fn get_network_pool(&self, uuid: &str) -> Result<NetworkPool> {
        let path = format!("network_pools/{uuid}");
        self.get_json(&path, &[]).await
    }

    /// List NICs (optionally filtered by query parameters).
    pub async fn list_nics(&self, params: &[(&'static str, String)]) -> Result<Vec<Nic>> {
        self.get_json("nics", params).await
    }

    /// Fetch a NIC by MAC address.
    pub async fn get_nic(&self, mac: &str) -> Result<Nic> {
        let path = format!("nics/{mac}");
        self.get_json(&path, &[]).await
    }

    /// Create a NIC.
    pub async fn create_nic(&self, nic: &Nic) -> Result<Nic> {
        self.send_json(Method::POST, "nics", Some(nic), &[]).await
    }

    /// Update a NIC by MAC address.
    pub async fn update_nic(&self, mac: &str, nic: &Nic) -> Result<Nic> {
        let path = format!("nics/{mac}");
        self.send_json(Method::PUT, &path, Some(nic), &[]).await
    }

    /// Delete a NIC by MAC address.
    pub async fn delete_nic(&self, mac: &str) -> Result<()> {
        let path = format!("nics/{mac}");
        self.send_empty(Method::DELETE, &path, &[]).await
    }

    fn build_url(&self, path: &str) -> Result<Url> {
        let normalized = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        self.base_url
            .join(normalized)
            .map_err(|err| Error::InvalidEndpoint(format!("Invalid NAPI path `{path}`: {err}")))
    }

    async fn get_json<T>(&self, path: &str, params: &[(&'static str, String)]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send_json::<(), T>(Method::GET, path, None, params)
            .await
    }

    async fn send_empty(
        &self,
        method: Method,
        path: &str,
        params: &[(&'static str, String)],
    ) -> Result<()> {
        self.send_json::<(), serde_json::Value>(method, path, None, params)
            .await
            .map(|_| ())
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

            info!(path, attempt, "NAPI request");

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    let bytes = response.bytes().await.map_err(|err| {
                        Error::HttpError(format!("Failed to read NAPI response body: {err}"))
                    })?;

                    if status.is_success() {
                        return deserialize_body(path, status, &bytes);
                    }

                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    let error = match status {
                        StatusCode::NOT_FOUND => return Err(Error::NotFound(text)),
                        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                            Error::InvalidRequest(format!("NAPI authentication failed: {text}"))
                        }
                        StatusCode::TOO_MANY_REQUESTS
                        | StatusCode::BAD_GATEWAY
                        | StatusCode::SERVICE_UNAVAILABLE
                        | StatusCode::GATEWAY_TIMEOUT => Error::ServiceUnavailable(format!(
                            "NAPI temporarily unavailable: {text}"
                        )),
                        status if status.is_server_error() => {
                            Error::ServiceUnavailable(format!("NAPI server error {status}: {text}"))
                        }
                        _ => Error::HttpError(format!("NAPI error {status}: {text}")),
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
                debug!("Retrying NAPI request after {:?}", delay);
                sleep(delay).await;
            }
        }

        if let Some(error) = last_error {
            Err(error)
        } else {
            Err(Error::ServiceUnavailable(
                "NAPI request failed after retries".to_string(),
            ))
        }
    }
}

fn deserialize_body<R>(path: &str, status: StatusCode, bytes: &[u8]) -> Result<R>
where
    R: DeserializeOwned,
{
    if status == StatusCode::NO_CONTENT || bytes.is_empty() {
        serde_json::from_value(serde_json::Value::Null).map_err(|err| {
            Error::SapiParseError(format!(
                "Failed to parse empty NAPI response for `{path}`: {err}"
            ))
        })
    } else {
        serde_json::from_slice(bytes).map_err(|err| {
            Error::SapiParseError(format!("Failed to parse NAPI response for `{path}`: {err}"))
        })
    }
}

/// Discovery adapter that reuses SAPI-based discovery for NAPI endpoints.
pub struct NapiDiscovery {
    sapi: Arc<dyn ServiceDiscovery>,
    status: Arc<RwLock<DiscoveryStatus>>,
}

impl NapiDiscovery {
    /// Create a new discovery helper.
    #[must_use]
    pub fn new(sapi_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            sapi: sapi_discovery,
            status: Arc::new(RwLock::new(DiscoveryStatus::new())),
        }
    }

    fn record_success(&self, count: usize) {
        if let Ok(mut status) = self.status.write() {
            *status = status.clone().with_success(count);
        }
    }

    fn record_error(&self, error: &Error) {
        if let Ok(mut status) = self.status.write() {
            status.last_error = Some(error.to_string());
        }
    }
}

#[async_trait]
impl ServiceDiscovery for NapiDiscovery {
    async fn discover_service(&self, service_name: &str) -> Result<Vec<String>> {
        match self.sapi.discover_service(service_name).await {
            Ok(endpoints) => {
                self.record_success(endpoints.len());
                Ok(endpoints)
            }
            Err(err) => {
                self.record_error(&err);
                Err(err)
            }
        }
    }

    async fn discover_all_services(&self) -> Result<Vec<String>> {
        self.sapi.discover_service(TritonService::Napi.name()).await
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

/// Convenient alias for network queries.
pub type NetworkQuery = NetworkListParams;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use triton_core::uuid::OwnerUuid;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(server: &MockServer) -> NapiClient {
        NapiClient::new(server.uri()).unwrap()
    }

    #[tokio::test]
    async fn list_networks_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/networks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "uuid": NetworkUuid::new_v4(),
                    "name": "admin",
                    "vlan_id": 42,
                    "subnet": "10.0.0.0/24",
                    "netmask": "255.255.255.0",
                    "nic_tag": "admin"
                }
            ])))
            .mount(&server)
            .await;

        let client = client(&server);
        let networks = client
            .list_networks(&NetworkListParams::default())
            .await
            .unwrap();
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].name, "admin");
    }

    #[tokio::test]
    async fn get_network_not_found() {
        let server = MockServer::start().await;
        let uuid = NetworkUuid::new_v4();
        Mock::given(method("GET"))
            .and(path(format!("/networks/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(404).set_body_string("missing"))
            .mount(&server)
            .await;

        let client = client(&server);
        let err = client.get_network(uuid).await.unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn create_network_success() {
        let server = MockServer::start().await;
        let owner = OwnerUuid::new_v4();
        Mock::given(method("POST"))
            .and(path("/networks"))
            .and(body_json(json!({
                "name": "admin",
                "vlan_id": 42,
                "subnet": "10.0.0.0/24",
                "netmask": "255.255.255.0",
                "nic_tag": "admin",
                "owner_uuids": [owner]
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "uuid": NetworkUuid::new_v4(),
                "name": "admin",
                "vlan_id": 42,
                "subnet": "10.0.0.0/24",
                "netmask": "255.255.255.0",
                "nic_tag": "admin"
            })))
            .mount(&server)
            .await;

        let request = CreateNetworkRequest {
            name: "admin".into(),
            vlan_id: 42,
            subnet: "10.0.0.0/24".into(),
            netmask: "255.255.255.0".into(),
            gateway: None,
            provision_start_ip: None,
            provision_end_ip: None,
            nic_tag: "admin".into(),
            description: None,
            owner_uuids: Some(vec![owner]),
            routes: None,
            resolvers: None,
            fabric: None,
            internet_nat: None,
            mtu: None,
        };

        let client = client(&server);
        let network = client.create_network(&request).await.unwrap();
        assert_eq!(network.name, "admin");
    }

    #[tokio::test]
    async fn discovery_delegates_to_sapi() {
        struct MockDiscovery;

        #[async_trait]
        impl ServiceDiscovery for MockDiscovery {
            async fn discover_service(&self, service_name: &str) -> Result<Vec<String>> {
                assert_eq!(service_name, "napi");
                Ok(vec!["http://napi.local:80".into()])
            }

            async fn discover_all_services(&self) -> Result<Vec<String>> {
                Ok(vec![])
            }

            fn get_status(&self) -> DiscoveryStatus {
                DiscoveryStatus::new()
            }

            fn clear_cache(&self) {}
        }

        let discovery = NapiDiscovery::new(Arc::new(MockDiscovery));
        let endpoints = discovery.discover_service("napi").await.unwrap();
        assert_eq!(endpoints, vec!["http://napi.local:80"]);
    }
}
