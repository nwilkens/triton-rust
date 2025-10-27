//! Asynchronous SAPI client implementation and discovery utilities.

use crate::models::{Application, Instance, InstanceType, Service};
use crate::Result;
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::de::DeserializeOwned;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use triton_core::client::{
    ClientConfig, RetryPolicy, DEFAULT_POOL_IDLE_TIMEOUT, DEFAULT_POOL_MAX_IDLE_PER_HOST,
    SAPI_DEFAULT_TIMEOUT,
};
use triton_core::config::{
    ServiceDiscoveryConfig, ServiceEndpointConfig, ServiceEndpoints, TritonClientConfig,
};
use triton_core::services::{DiscoveryStatus, ServiceDiscovery};
use triton_core::types::TritonService;
use triton_core::uuid::{AppUuid, InstanceUuid, ServiceUuid};
use triton_core::Error;
use url::Url;

const USER_AGENT: &str = concat!("triton-sapi/", env!("CARGO_PKG_VERSION"));
const ACCEPT_VERSION: &str = "2.0.0";

/// Builder for [`SapiClient`].
#[derive(Debug, Clone)]
pub struct SapiClientBuilder {
    config: TritonClientConfig,
    http_config: ClientConfig,
    accept_version: String,
}

impl SapiClientBuilder {
    /// Create a new builder from a [`TritonClientConfig`].
    #[must_use]
    pub fn new(config: TritonClientConfig) -> Self {
        let http_config = ClientConfig::new()
            .with_timeout(Duration::from_secs(SAPI_DEFAULT_TIMEOUT))
            .with_pool_idle_timeout(Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT))
            .with_pool_max_idle(DEFAULT_POOL_MAX_IDLE_PER_HOST);

        Self {
            config,
            http_config,
            accept_version: ACCEPT_VERSION.to_string(),
        }
    }

    /// Override the HTTP client configuration used when building the client.
    #[must_use]
    pub fn with_http_config(mut self, http_config: ClientConfig) -> Self {
        self.http_config = http_config;
        self
    }

    /// Override the `Accept-Version` header sent to SAPI (defaults to `2.0.0`).
    #[must_use]
    pub fn with_accept_version(mut self, version: impl Into<String>) -> Self {
        self.accept_version = version.into();
        self
    }

    /// Finalise the builder and create the [`SapiClient`].
    pub fn build(self) -> Result<SapiClient> {
        let base_url = self.config.parse_sapi_url()?;

        let mut http_config = self.http_config;
        http_config.timeout = self.config.timeout();
        http_config.retry_policy = http_config
            .retry_policy
            .with_max_retries(self.config.max_retries);

        let mut builder = ClientBuilder::new()
            .user_agent(USER_AGENT)
            .timeout(http_config.timeout)
            .pool_idle_timeout(http_config.pool_idle_timeout)
            .pool_max_idle_per_host(http_config.pool_max_idle_per_host)
            .connect_timeout(Duration::from_secs(10));

        if !self.config.tls_verify {
            warn!("TLS verification disabled for SAPI client");
            builder = builder.danger_accept_invalid_certs(true);
        }

        if let Some(ca_cert) = &self.config.tls_ca_cert {
            debug!("loading SAPI CA certificate from {}", ca_cert.display());
            let bytes = std::fs::read(ca_cert).map_err(|err| {
                Error::ConfigError(format!(
                    "Failed to read SAPI CA certificate {}: {err}",
                    ca_cert.display()
                ))
            })?;
            let cert = reqwest::Certificate::from_pem(&bytes)
                .map_err(|err| Error::ConfigError(format!("Invalid SAPI CA certificate: {err}")))?;
            builder = builder.add_root_certificate(cert);
        }

        let http = builder.build().map_err(|err| {
            Error::ConfigError(format!("Failed to build SAPI HTTP client: {err}"))
        })?;

        let api_key = self.config.sapi_key.clone();

        Ok(SapiClient {
            http,
            base_url,
            api_key,
            retry_policy: http_config.retry_policy,
            accept_version: self.accept_version,
            discovery_config: self.config.service_discovery.clone(),
        })
    }
}

/// Asynchronous client for the Triton Services API (SAPI).
#[derive(Clone)]
pub struct SapiClient {
    http: Client,
    base_url: Url,
    api_key: Option<String>,
    retry_policy: RetryPolicy,
    accept_version: String,
    discovery_config: ServiceDiscoveryConfig,
}

impl SapiClient {
    /// Construct a client directly from the configuration.
    pub fn from_config(config: &TritonClientConfig) -> Result<Self> {
        SapiClientBuilder::new(config.clone()).build()
    }

    /// Start a builder pre-populated with the provided configuration.
    #[must_use]
    pub fn builder(config: TritonClientConfig) -> SapiClientBuilder {
        SapiClientBuilder::new(config)
    }

    /// Access the underlying discovery configuration.
    #[must_use]
    pub fn discovery_config(&self) -> &ServiceDiscoveryConfig {
        &self.discovery_config
    }

    /// Create a discovery helper using this client and its discovery settings.
    #[must_use]
    pub fn discovery(&self) -> SapiDiscovery {
        SapiDiscovery::new(Arc::new(self.clone()), self.discovery_config.clone())
    }

    /// List applications registered in SAPI.
    pub async fn list_applications(&self) -> Result<Vec<Application>> {
        self.get_json("applications", &[]).await
    }

    /// Fetch a specific application by UUID.
    pub async fn get_application(&self, uuid: AppUuid) -> Result<Application> {
        let path = format!("applications/{uuid}");
        self.get_json(&path, &[]).await
    }

    /// List services, optionally filtered using a query.
    pub async fn list_services<'a>(&self, query: &ServiceQuery<'a>) -> Result<Vec<Service>> {
        let params = query.to_params();
        self.get_json("services", &params).await
    }

    /// Fetch a specific service by UUID.
    pub async fn get_service(&self, uuid: ServiceUuid) -> Result<Service> {
        let path = format!("services/{uuid}");
        self.get_json(&path, &[]).await
    }

    /// List instances, optionally filtered using a query.
    pub async fn list_instances(&self, query: &InstanceQuery) -> Result<Vec<Instance>> {
        let params = query.to_params();
        self.get_json("instances", &params).await
    }

    /// Fetch a specific instance by UUID.
    pub async fn get_instance(&self, uuid: InstanceUuid) -> Result<Instance> {
        let path = format!("instances/{uuid}");
        self.get_json(&path, &[]).await
    }

    /// Discover endpoints for a specific Triton service via SAPI.
    pub async fn discover_service_endpoints(&self, service: TritonService) -> Result<Vec<String>> {
        let filters = ServiceQuery::new().with_name(service.name());
        let services = self.list_services(&filters).await?;

        if services.is_empty() {
            return Err(Error::NotFound(format!(
                "SAPI service `{}` not found",
                service.name()
            )));
        }

        let mut endpoints = BTreeSet::new();

        for svc in services {
            let instance_filters = InstanceQuery::new()
                .with_service_uuid(svc.uuid)
                .include_master(true);

            let instances = self.list_instances(&instance_filters).await?;

            for instance in instances {
                for endpoint in self.extract_instance_endpoints(service, &instance) {
                    endpoints.insert(endpoint);
                }
            }
        }

        if endpoints.is_empty() {
            return Err(Error::DiscoveryFailed(format!(
                "No endpoints discovered for service `{}`",
                service.name()
            )));
        }

        Ok(endpoints.into_iter().collect())
    }

    fn extract_instance_endpoints(
        &self,
        service: TritonService,
        instance: &Instance,
    ) -> Vec<String> {
        let mut endpoints = BTreeSet::new();
        let service_name = service.name();

        let key_variants = [
            format!("{service_name}_url"),
            format!("{service_name}_endpoint"),
            "url".to_string(),
        ];

        for key in &key_variants {
            if key == "url" {
                if let Some(value) = instance.metadata.get(key).and_then(|v| v.as_str()) {
                    endpoints.insert(value.to_string());
                }
                if let Some(value) = instance.params.get(key).and_then(|v| v.as_str()) {
                    endpoints.insert(value.to_string());
                }
            } else {
                if let Some(value) = instance.metadata.get(key).and_then(|v| v.as_str()) {
                    endpoints.insert(value.to_string());
                }
                if let Some(value) = instance.params.get(key).and_then(|v| v.as_str()) {
                    endpoints.insert(value.to_string());
                }
            }
        }

        if endpoints.is_empty() {
            if let Some(hostname) = instance.hostname.as_deref() {
                let protocol = match service {
                    TritonService::Ufds => "ldaps",
                    _ => "http",
                };
                let port = service.default_port();
                endpoints.insert(format!("{protocol}://{hostname}:{port}"));
            }
        }

        endpoints.into_iter().collect()
    }

    fn build_url(&self, path: &str) -> Result<Url> {
        self.base_url
            .join(path)
            .map_err(|err| Error::InvalidEndpoint(format!("Invalid SAPI path `{path}`: {err}")))
    }

    async fn get_json<T>(&self, path: &str, params: &[(&'static str, String)]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut attempt = 0;
        #[allow(unused_assignments)]
        let mut last_error: Option<Error> = None;

        loop {
            let url = self.build_url(path)?;
            let mut request = self.http.get(url).query(&params);
            request = request.header("Accept-Version", &self.accept_version);
            request = request.header("Accept", "application/json");

            if let Some(api_key) = &self.api_key {
                request = request.header("X-Api-Key", api_key);
            }

            info!(path = %path, ?params, attempt, "Sending SAPI request");

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        return response.json::<T>().await.map_err(|err| {
                            Error::SapiParseError(format!(
                                "Failed to parse SAPI response for `{path}`: {err}"
                            ))
                        });
                    }

                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());

                    let error = match status {
                        StatusCode::NOT_FOUND => {
                            return Err(Error::NotFound(message));
                        }
                        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                            return Err(Error::InvalidRequest(format!(
                                "SAPI authentication failed: {message}"
                            )));
                        }
                        StatusCode::TOO_MANY_REQUESTS
                        | StatusCode::BAD_GATEWAY
                        | StatusCode::SERVICE_UNAVAILABLE
                        | StatusCode::GATEWAY_TIMEOUT => Error::ServiceUnavailable(format!(
                            "SAPI temporarily unavailable: {message}"
                        )),
                        status if status.is_server_error() => Error::ServiceUnavailable(format!(
                            "SAPI server error {status}: {message}"
                        )),
                        _ => {
                            return Err(Error::HttpError(format!(
                                "SAPI error {status}: {message}"
                            )));
                        }
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
                debug!("Retrying SAPI request after {:?}", delay);
                sleep(delay).await;
            }
        }

        if let Some(error) = last_error {
            Err(error)
        } else {
            Err(Error::ServiceUnavailable(
                "SAPI request failed after retries".to_string(),
            ))
        }
    }
}

/// Query parameters for listing services.
#[derive(Debug, Default)]
pub struct ServiceQuery<'a> {
    name: Option<&'a str>,
    application_uuid: Option<AppUuid>,
    service_type: Option<InstanceType>,
    include_master: bool,
}

impl<'a> ServiceQuery<'a> {
    /// Create an empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            application_uuid: None,
            service_type: None,
            include_master: false,
        }
    }

    /// Filter by service name.
    #[must_use]
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Filter by owning application UUID.
    #[must_use]
    pub fn with_application_uuid(mut self, uuid: AppUuid) -> Self {
        self.application_uuid = Some(uuid);
        self
    }

    /// Filter by service type.
    #[must_use]
    pub fn with_type(mut self, service_type: InstanceType) -> Self {
        self.service_type = Some(service_type);
        self
    }

    /// Include master configuration details in responses.
    #[must_use]
    pub fn include_master(mut self, include: bool) -> Self {
        self.include_master = include;
        self
    }

    fn to_params(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();

        if let Some(name) = self.name {
            params.push(("name", name.to_string()))
        }

        if let Some(uuid) = self.application_uuid {
            params.push(("application_uuid", uuid.to_string()));
        }

        if let Some(service_type) = self.service_type {
            params.push(("type", service_type.as_str().to_string()));
        }

        if self.include_master {
            params.push(("include_master", "true".to_string()));
        }

        params
    }
}

/// Query parameters for listing instances.
#[derive(Debug, Default)]
pub struct InstanceQuery {
    service_uuid: Option<ServiceUuid>,
    service_type: Option<InstanceType>,
    include_master: bool,
}

impl InstanceQuery {
    /// Create an empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            service_uuid: None,
            service_type: None,
            include_master: false,
        }
    }

    /// Filter by service UUID.
    #[must_use]
    pub fn with_service_uuid(mut self, uuid: ServiceUuid) -> Self {
        self.service_uuid = Some(uuid);
        self
    }

    /// Filter by instance type.
    #[must_use]
    pub fn with_type(mut self, instance_type: InstanceType) -> Self {
        self.service_type = Some(instance_type);
        self
    }

    /// Include master configuration details.
    #[must_use]
    pub fn include_master(mut self, include: bool) -> Self {
        self.include_master = include;
        self
    }

    fn to_params(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();

        if let Some(uuid) = self.service_uuid {
            params.push(("service_uuid", uuid.to_string()));
        }

        if let Some(instance_type) = self.service_type {
            params.push(("type", instance_type.as_str().to_string()));
        }

        if self.include_master {
            params.push(("include_master", "true".to_string()));
        }

        params
    }
}

#[derive(Clone)]
struct CachedEntry {
    endpoints: Vec<String>,
    fetched_at: Instant,
}

/// SAPI-backed implementation of [`ServiceDiscovery`].
pub struct SapiDiscovery {
    client: Arc<SapiClient>,
    cache: Arc<RwLock<HashMap<String, CachedEntry>>>,
    status: Arc<RwLock<DiscoveryStatus>>,
    ttl: Duration,
    retry_attempts: u32,
    fallback: HashMap<String, Vec<String>>,
    enabled: bool,
}

impl SapiDiscovery {
    /// Create a new discovery helper from an existing [`SapiClient`].
    #[must_use]
    pub fn new(client: Arc<SapiClient>, config: ServiceDiscoveryConfig) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(DiscoveryStatus::new())),
            ttl: Duration::from_secs(config.cache_ttl_secs),
            retry_attempts: config.retry_attempts,
            fallback: build_fallback_map(&config.services),
            enabled: config.enabled,
        }
    }

    async fn refresh_service(&self, service: &str) -> Result<Vec<String>> {
        let triton_service = service.parse::<TritonService>()?;
        let mut attempt = 0;
        let mut last_error: Option<Error> = None;

        while attempt <= self.retry_attempts {
            match self.client.discover_service_endpoints(triton_service).await {
                Ok(endpoints) => {
                    self.record_success(service, endpoints.len()).await;
                    let mut cache = self.cache.write().unwrap();
                    cache.insert(
                        service.to_string(),
                        CachedEntry {
                            endpoints: endpoints.clone(),
                            fetched_at: Instant::now(),
                        },
                    );
                    return Ok(endpoints);
                }
                Err(err) => {
                    last_error = Some(err);
                    attempt += 1;
                    if attempt > self.retry_attempts {
                        break;
                    }
                    let delay = self.client.retry_policy.delay_for_attempt(attempt);
                    if delay > Duration::from_millis(0) {
                        sleep(delay).await;
                    }
                }
            }
        }

        self.record_error(service, last_error.as_ref()).await;

        if let Some(fallback) = self.fallback.get(service) {
            if !fallback.is_empty() {
                info!("Using fallback endpoints for service {}", service);
                return Ok(fallback.clone());
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::DiscoveryFailed(format!("Failed to discover endpoints for `{service}`"))
        }))
    }

    async fn record_success(&self, service: &str, count: usize) {
        let mut status = self.status.write().unwrap();
        let now = Instant::now();
        status.last_discovery_at = Some(now);
        status.last_success_at = Some(now);
        status.last_error = None;
        status.discovered_services = count;
        status.failed_services.retain(|failed| failed != service);
        status.cache_misses += 1;
    }

    async fn record_error(&self, service: &str, error: Option<&Error>) {
        let mut status = self.status.write().unwrap();
        status.last_discovery_at = Some(Instant::now());
        if let Some(error) = error {
            status.last_error = Some(error.to_string());
        }
        if !status.failed_services.iter().any(|s| s == service) {
            status.failed_services.push(service.to_string());
        }
    }

    fn cache_entry_is_fresh(&self, entry: &CachedEntry) -> bool {
        entry.fetched_at.elapsed() <= self.ttl
    }
}

#[async_trait]
impl ServiceDiscovery for SapiDiscovery {
    async fn discover_service(&self, service_name: &str) -> Result<Vec<String>> {
        let service_key = service_name.to_lowercase();

        if !self.enabled {
            if let Some(fallback) = self.fallback.get(&service_key) {
                if !fallback.is_empty() {
                    return Ok(fallback.clone());
                }
            }
            return Err(Error::DiscoveryFailed(format!(
                "Service discovery disabled and no fallback for `{service_name}`"
            )));
        }

        if let Some(entry) = self.cache.read().unwrap().get(&service_key).cloned() {
            if self.cache_entry_is_fresh(&entry) {
                let mut status = self.status.write().unwrap();
                status.cache_hits += 1;
                return Ok(entry.endpoints);
            }
        }

        self.refresh_service(&service_key).await
    }

    async fn discover_all_services(&self) -> Result<Vec<String>> {
        let mut endpoints = Vec::new();
        for service in TritonService::all() {
            if let Ok(mut discovered) = self.discover_service(service.name()).await {
                endpoints.append(&mut discovered);
            }
        }
        Ok(endpoints)
    }

    fn get_status(&self) -> DiscoveryStatus {
        self.status.read().unwrap().clone()
    }

    fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
        let mut status = self.status.write().unwrap();
        status.cache_hits = 0;
        status.cache_misses = 0;
    }
}

fn build_fallback_map(endpoints: &ServiceEndpoints) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();

    fn insert_endpoint(
        map: &mut HashMap<String, Vec<String>>,
        service: &'static str,
        endpoint: &Option<ServiceEndpointConfig>,
    ) {
        if let Some(config) = endpoint {
            map.entry(service.to_string())
                .or_default()
                .push(config.url.clone());
        }
    }

    insert_endpoint(&mut map, "vmapi", &endpoints.vmapi);
    insert_endpoint(&mut map, "cnapi", &endpoints.cnapi);
    insert_endpoint(&mut map, "napi", &endpoints.napi);
    insert_endpoint(&mut map, "imgapi", &endpoints.imgapi);
    insert_endpoint(&mut map, "papi", &endpoints.papi);
    insert_endpoint(&mut map, "fwapi", &endpoints.fwapi);

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use triton_core::config::TritonClientConfig;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(base_url: &str) -> TritonClientConfig {
        TritonClientConfig::new(base_url)
            .unwrap()
            .with_tls_verify(true)
    }

    #[tokio::test]
    async fn test_list_services_with_name_filter() {
        let server = MockServer::start().await;
        let service_uuid = ServiceUuid::new_v4();
        let response_body = serde_json::json!([
            {
                "uuid": service_uuid.to_string(),
                "name": "vmapi",
                "application_uuid": AppUuid::new_v4().to_string(),
                "params": {},
                "metadata": {},
                "manifests": null,
                "master": null
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/services"))
            .and(query_param("name", "vmapi"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&server)
            .await;

        let config = test_config(&server.uri());
        let client = SapiClient::from_config(&config).unwrap();
        let services = client
            .list_services(&ServiceQuery::new().with_name("vmapi"))
            .await
            .unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].uuid, service_uuid);
    }

    #[tokio::test]
    async fn test_list_instances_with_service_uuid() {
        let server = MockServer::start().await;
        let service_uuid = ServiceUuid::new_v4();
        let instance_uuid = InstanceUuid::new_v4();

        let response_body = serde_json::json!([
            {
                "uuid": instance_uuid.to_string(),
                "service_uuid": service_uuid.to_string(),
                "hostname": "vmapi.local",
                "params": {},
                "metadata": {},
                "manifests": null,
                "master": false
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/instances"))
            .and(query_param("service_uuid", &service_uuid.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&server)
            .await;

        let config = test_config(&server.uri());
        let client = SapiClient::from_config(&config).unwrap();
        let query = InstanceQuery::new().with_service_uuid(service_uuid);
        let instances = client.list_instances(&query).await.unwrap();

        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].uuid, instance_uuid);
    }

    #[tokio::test]
    async fn test_discover_service_endpoints() {
        let server = MockServer::start().await;
        let service_uuid = ServiceUuid::new_v4();
        let instance_uuid = InstanceUuid::new_v4();
        let service_response = serde_json::json!([
            {
                "uuid": service_uuid.to_string(),
                "name": "vmapi",
                "application_uuid": AppUuid::new_v4().to_string(),
                "params": {},
                "metadata": {},
                "manifests": null,
                "master": null
            }
        ]);
        let instance_response = serde_json::json!([
            {
                "uuid": instance_uuid.to_string(),
                "service_uuid": service_uuid.to_string(),
                "hostname": "vmapi.local",
                "params": {},
                "metadata": {},
                "manifests": null,
                "master": false
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/services"))
            .and(query_param("name", "vmapi"))
            .respond_with(ResponseTemplate::new(200).set_body_json(service_response))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/instances"))
            .and(query_param("service_uuid", &service_uuid.to_string()))
            .and(query_param("include_master", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(instance_response))
            .expect(1)
            .mount(&server)
            .await;

        let config = test_config(&server.uri());
        let client = SapiClient::from_config(&config).unwrap();
        let endpoints = client
            .discover_service_endpoints(TritonService::Vmapi)
            .await
            .unwrap();

        assert_eq!(endpoints, vec!["http://vmapi.local:80".to_string()]);
    }

    #[tokio::test]
    async fn test_sapi_discovery_cache() {
        let server = MockServer::start().await;
        let service_uuid = ServiceUuid::new_v4();
        let service_response = serde_json::json!([
            {
                "uuid": service_uuid.to_string(),
                "name": "vmapi",
                "application_uuid": AppUuid::new_v4().to_string(),
                "params": {},
                "metadata": {},
                "manifests": null,
                "master": null
            }
        ]);
        let instance_response = serde_json::json!([
            {
                "uuid": InstanceUuid::new_v4().to_string(),
                "service_uuid": service_uuid.to_string(),
                "hostname": "vmapi.local",
                "params": {},
                "metadata": {},
                "manifests": null,
                "master": false
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/services"))
            .and(query_param("name", "vmapi"))
            .respond_with(ResponseTemplate::new(200).set_body_json(service_response))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/instances"))
            .and(query_param("service_uuid", &service_uuid.to_string()))
            .and(query_param("include_master", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(instance_response))
            .expect(1)
            .mount(&server)
            .await;

        let config = test_config(&server.uri());
        let client = SapiClient::from_config(&config).unwrap();
        let discovery = client.discovery();

        let first = discovery.discover_service("vmapi").await.unwrap();
        assert_eq!(first.len(), 1);

        let second = discovery.discover_service("vmapi").await.unwrap();
        assert_eq!(second.len(), 1);

        let status = discovery.get_status();
        assert_eq!(status.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_sapi_discovery_fallback() {
        let config = TritonClientConfig::new("http://localhost:1234")
            .unwrap()
            .with_service_discovery(ServiceDiscoveryConfig {
                enabled: false,
                cache_ttl_secs: 1,
                timeout_secs: 1,
                retry_attempts: 0,
                services: ServiceEndpoints::new()
                    .with_vmapi(ServiceEndpointConfig::new("http://fallback:80").unwrap()),
            });

        let client = SapiClient::from_config(&config).unwrap();
        let discovery = client.discovery();

        let endpoints = discovery.discover_service("vmapi").await.unwrap();
        assert_eq!(endpoints, vec!["http://fallback:80".to_string()]);
    }
}
