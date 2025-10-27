//! Asynchronous PAPI client implementation.

use crate::models::{CreatePackageRequest, Package, PackageListParams, UpdatePackageRequest};
use crate::Result;
use async_trait::async_trait;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use triton_core::client::{
    ClientConfig, RetryPolicy, ServiceClient, ServiceClientBuilder, PAPI_DEFAULT_TIMEOUT,
};
use triton_core::services::{DiscoveryStatus, ServiceDiscovery, ServiceDiscoveryProxy};
use triton_core::types::TritonService;
use triton_core::uuid::PackageUuid;
use triton_core::Error;
use url::Url;

const USER_AGENT: &str = concat!("triton-papi/", env!("CARGO_PKG_VERSION"));

/// Builder for [`PapiClient`].
#[derive(Debug, Clone)]
pub struct PapiClientBuilder {
    inner: ServiceClientBuilder,
}

impl PapiClientBuilder {
    /// Create a builder for the specified base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let builder = ServiceClientBuilder::new(
            TritonService::Papi,
            base_url,
            Duration::from_secs(PAPI_DEFAULT_TIMEOUT),
        )?
        .with_user_agent(USER_AGENT);

        Ok(Self { inner: builder })
    }

    /// Override the retry policy.
    #[must_use]
    pub fn with_retry_policy(mut self, retry: RetryPolicy) -> Self {
        self.inner = self.inner.with_retry_policy(retry);
        self
    }

    /// Override the HTTP client configuration.
    #[must_use]
    pub fn with_http_config(mut self, config: ClientConfig) -> Self {
        self.inner = self.inner.with_http_config(config);
        self
    }

    /// Configure HTTP basic authentication credentials.
    #[must_use]
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.inner = self.inner.with_basic_auth(username, password);
        self
    }

    /// Configure an X-Auth-Token header.
    #[must_use]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.inner = self.inner.with_token(token);
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<PapiClient> {
        let inner = self.inner.build()?;
        Ok(PapiClient { inner })
    }
}

/// Asynchronous PAPI client.
#[derive(Clone)]
pub struct PapiClient {
    inner: ServiceClient,
}

impl PapiClient {
    /// Construct a client directly from the base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        PapiClientBuilder::new(base_url)?.build()
    }

    /// Return the base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        self.inner.base_url()
    }

    /// List packages with optional filters.
    pub async fn list_packages(&self, params: &PackageListParams) -> Result<Vec<Package>> {
        self.send_json::<(), Vec<Package>>(Method::GET, "packages", None, &params.to_pairs())
            .await
    }

    /// Fetch a single package by UUID.
    pub async fn get_package(&self, uuid: PackageUuid) -> Result<Package> {
        let path = format!("packages/{uuid}");
        self.send_json::<(), Package>(Method::GET, &path, None, &[])
            .await
    }

    /// Create a new package.
    pub async fn create_package(&self, request: &CreatePackageRequest) -> Result<Package> {
        self.send_json(Method::POST, "packages", Some(request), &[])
            .await
    }

    /// Update an existing package.
    pub async fn update_package(
        &self,
        uuid: PackageUuid,
        request: &UpdatePackageRequest,
    ) -> Result<Package> {
        let path = format!("packages/{uuid}");
        self.send_json(Method::PUT, &path, Some(request), &[]).await
    }

    /// Delete a package by UUID.
    pub async fn delete_package(&self, uuid: PackageUuid) -> Result<()> {
        let path = format!("packages/{uuid}");
        self.inner
            .execute_with_retry(
                Method::DELETE,
                &path,
                &[],
                |request| request,
                map_status_to_error,
            )
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
        let response = self
            .inner
            .execute_with_retry(
                method,
                path,
                params,
                |mut request| {
                    request = request.header("Accept", "application/json");
                    if let Some(payload) = body {
                        request = request.json(payload);
                    }
                    request
                },
                map_status_to_error,
            )
            .await?;

        response.json::<R>().await.map_err(Error::from)
    }
}

fn map_status_to_error(status: StatusCode, text: String) -> Error {
    match status {
        StatusCode::NOT_FOUND => Error::NotFound(text),
        StatusCode::BAD_REQUEST => Error::BadRequest(text),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            Error::InvalidRequest(format!("PAPI authentication failed: {text}"))
        }
        StatusCode::CONFLICT => Error::Conflict(text),
        StatusCode::TOO_MANY_REQUESTS
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => {
            Error::ServiceUnavailable(format!("PAPI temporarily unavailable: {text}"))
        }
        status if status.is_server_error() => {
            Error::ServiceUnavailable(format!("PAPI server error {status}: {text}"))
        }
        _ => Error::HttpError(format!("PAPI error {status}: {text}")),
    }
}

/// Discovery adapter that relies on SAPI for PAPI endpoints.
pub struct PapiDiscovery {
    proxy: ServiceDiscoveryProxy,
}

impl PapiDiscovery {
    /// Create a discovery wrapper from an existing SAPI discovery instance.
    #[must_use]
    pub fn new(sapi_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            proxy: ServiceDiscoveryProxy::for_service(sapi_discovery, TritonService::Papi),
        }
    }
}

#[async_trait]
impl ServiceDiscovery for PapiDiscovery {
    async fn discover_service(&self, service_name: &str) -> Result<Vec<String>> {
        self.proxy.discover_service(service_name).await
    }

    async fn discover_all_services(&self) -> Result<Vec<String>> {
        self.proxy.discover_all_services().await
    }

    fn get_status(&self) -> DiscoveryStatus {
        self.proxy.get_status()
    }

    fn clear_cache(&self) {
        self.proxy.clear_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> PapiClient {
        PapiClient::new(server.uri()).unwrap()
    }

    #[tokio::test]
    async fn list_packages_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/packages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "uuid": PackageUuid::new_v4(),
                    "name": "standard-2cpu-4gb",
                    "max_physical_memory": 4096
                }
            ])))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let packages = client
            .list_packages(&PackageListParams::default())
            .await
            .unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "standard-2cpu-4gb");
    }

    #[tokio::test]
    async fn get_package_not_found() {
        let server = MockServer::start().await;
        let uuid = PackageUuid::new_v4();

        Mock::given(method("GET"))
            .and(path(format!("/packages/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(404).set_body_string("missing"))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let err = client.get_package(uuid).await.unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn create_package_returns_package() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/packages"))
            .and(body_json(json!({
                "name": "standard",
                "max_physical_memory": 4096
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "uuid": PackageUuid::new_v4(),
                "name": "standard",
                "max_physical_memory": 4096
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let request = CreatePackageRequest {
            name: "standard".into(),
            version: None,
            max_physical_memory: 4096,
            quota: None,
            cpu_cap: None,
            cpu_shares: None,
            max_swap: None,
            max_lwps: None,
            zfs_io_priority: None,
            vcpus: None,
            memory: None,
            os: None,
            description: None,
            brand: None,
            disk: None,
            networks: None,
            group: None,
            active: None,
            default: None,
            ram_ratio: None,
            cpu_burst_ratio: None,
            cpu_burst_duty_cycle: None,
            io_priority: None,
            io_throttle: None,
            billing_tags: None,
            tags: None,
            common_name: None,
            owner_uuids: None,
            traits: None,
        };

        let package = client.create_package(&request).await.unwrap();
        assert_eq!(package.max_physical_memory, 4096);
    }

    #[tokio::test]
    async fn update_package_returns_package() {
        let server = MockServer::start().await;
        let uuid = PackageUuid::new_v4();

        Mock::given(method("PUT"))
            .and(path(format!("/packages/{uuid}").as_str()))
            .and(body_json(json!({
                "description": "Updated"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "uuid": uuid,
                "name": "standard",
                "max_physical_memory": 4096,
                "description": "Updated"
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let request = UpdatePackageRequest {
            description: Some("Updated".into()),
            ..UpdatePackageRequest::default()
        };

        let package = client.update_package(uuid, &request).await.unwrap();
        assert_eq!(package.description.as_deref(), Some("Updated"));
    }

    #[tokio::test]
    async fn delete_package_handles_no_content() {
        let server = MockServer::start().await;
        let uuid = PackageUuid::new_v4();

        Mock::given(method("DELETE"))
            .and(path(format!("/packages/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = test_client(&server);
        client.delete_package(uuid).await.unwrap();
    }

    struct MockDiscovery;

    #[async_trait]
    impl ServiceDiscovery for MockDiscovery {
        async fn discover_service(&self, _: &str) -> Result<Vec<String>> {
            Ok(vec!["http://papi.example.com".into()])
        }

        async fn discover_all_services(&self) -> Result<Vec<String>> {
            Ok(vec![])
        }

        fn get_status(&self) -> DiscoveryStatus {
            DiscoveryStatus::new()
        }

        fn clear_cache(&self) {}
    }

    #[tokio::test]
    async fn papi_discovery_delegates_to_proxy() {
        let discovery = Arc::new(MockDiscovery);
        let papi = PapiDiscovery::new(discovery);
        let endpoints = papi
            .discover_service(TritonService::Papi.name())
            .await
            .unwrap();
        assert_eq!(endpoints.len(), 1);
    }
}
