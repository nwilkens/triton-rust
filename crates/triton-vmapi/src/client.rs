//! Asynchronous VMAPI client implementation.

use crate::models::{
    BatchVMRequest, BatchVMResponse, CreateSnapshotRequest, CreateVMRequest, JobListParams,
    SnapshotActionResponse, UpdateVMRequest, VMListParams, Vm, VmSnapshot, VmapiJob,
};
use crate::Result;
use async_trait::async_trait;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use triton_core::client::{
    ClientConfig, RetryPolicy, ServiceClient, ServiceClientBuilder, VMAPI_DEFAULT_TIMEOUT,
};
use triton_core::services::{DiscoveryStatus, ServiceDiscovery, ServiceDiscoveryProxy};
use triton_core::types::TritonService;
use triton_core::uuid::InstanceUuid;
use triton_core::Error;
use url::Url;

const USER_AGENT: &str = concat!("triton-vmapi/", env!("CARGO_PKG_VERSION"));

/// Builder for [`VmapiClient`].
#[derive(Debug, Clone)]
pub struct VmapiClientBuilder {
    inner: ServiceClientBuilder,
}

impl VmapiClientBuilder {
    /// Create a builder for the specified base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let builder = ServiceClientBuilder::new(
            TritonService::Vmapi,
            base_url,
            Duration::from_secs(VMAPI_DEFAULT_TIMEOUT),
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
    pub fn build(self) -> Result<VmapiClient> {
        let inner = self.inner.build()?;
        Ok(VmapiClient { inner })
    }
}

/// Asynchronous VMAPI client.
#[derive(Clone)]
pub struct VmapiClient {
    inner: ServiceClient,
}

impl VmapiClient {
    /// Construct a client directly from the base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        VmapiClientBuilder::new(base_url)?.build()
    }

    /// Return the base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        self.inner.base_url()
    }

    /// List virtual machines.
    pub async fn list_vms(&self, params: &VMListParams) -> Result<Vec<Vm>> {
        self.get_json("vms", &params.to_pairs()).await
    }

    /// Fetch a single VM by UUID.
    pub async fn get_vm(&self, uuid: InstanceUuid) -> Result<Vm> {
        let path = format!("vms/{uuid}");
        self.get_json(&path, &[]).await
    }

    /// Create a VM (returns the provisioning job).
    pub async fn create_vm(&self, request: &CreateVMRequest) -> Result<VmapiJob> {
        self.send_json(Method::POST, "vms", Some(request), &[])
            .await
    }

    /// Update VM properties (returns the job).
    pub async fn update_vm(
        &self,
        uuid: InstanceUuid,
        request: &UpdateVMRequest,
    ) -> Result<VmapiJob> {
        let path = format!("vms/{uuid}");
        self.send_json(Method::PUT, &path, Some(request), &[]).await
    }

    /// Delete a VM.
    pub async fn delete_vm(&self, uuid: InstanceUuid) -> Result<VmapiJob> {
        let path = format!("vms/{uuid}");
        self.send_json::<(), VmapiJob>(Method::DELETE, &path, None, &[])
            .await
    }

    /// List VM snapshots.
    pub async fn list_snapshots(&self, uuid: InstanceUuid) -> Result<Vec<VmSnapshot>> {
        let path = format!("vms/{uuid}/snapshots");
        self.get_json(&path, &[]).await
    }

    /// Create a snapshot.
    pub async fn create_snapshot(
        &self,
        uuid: InstanceUuid,
        request: &CreateSnapshotRequest,
    ) -> Result<SnapshotActionResponse> {
        let path = format!("vms/{uuid}/snapshots");
        self.send_json(Method::POST, &path, Some(request), &[])
            .await
    }

    /// Delete a snapshot.
    pub async fn delete_snapshot(
        &self,
        uuid: InstanceUuid,
        snapshot: &str,
    ) -> Result<SnapshotActionResponse> {
        let path = format!("vms/{uuid}/snapshots/{snapshot}");
        self.send_json::<(), SnapshotActionResponse>(Method::DELETE, &path, None, &[])
            .await
    }

    /// Execute a batch action on multiple VMs.
    pub async fn batch_action(&self, request: &BatchVMRequest) -> Result<BatchVMResponse> {
        self.send_json(Method::POST, "vms/actions", Some(request), &[])
            .await
    }

    /// List jobs.
    pub async fn list_jobs(&self, params: &JobListParams) -> Result<Vec<VmapiJob>> {
        self.get_json("jobs", &params.to_pairs()).await
    }

    async fn get_json<T>(&self, path: &str, params: &[(&'static str, String)]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send_json::<(), T>(Method::GET, path, None, params)
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

        response.json::<R>().await.map_err(|err| {
            Error::SapiParseError(format!(
                "Failed to parse VMAPI response for `{path}`: {err}"
            ))
        })
    }
}

fn map_status_to_error(status: StatusCode, text: String) -> Error {
    match status {
        StatusCode::NOT_FOUND => Error::NotFound(text),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            Error::InvalidRequest(format!("VMAPI authentication failed: {text}"))
        }
        StatusCode::TOO_MANY_REQUESTS
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => {
            Error::ServiceUnavailable(format!("VMAPI temporarily unavailable: {text}"))
        }
        status if status.is_server_error() => {
            Error::ServiceUnavailable(format!("VMAPI server error {status}: {text}"))
        }
        _ => Error::HttpError(format!("VMAPI error {status}: {text}")),
    }
}

/// Discovery adapter that relies on SAPI for VMAPI endpoints.
pub struct VmapiDiscovery {
    proxy: ServiceDiscoveryProxy,
}

impl VmapiDiscovery {
    /// Create a discovery wrapper from an existing SAPI discovery instance.
    #[must_use]
    pub fn new(sapi_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            proxy: ServiceDiscoveryProxy::for_service(sapi_discovery, TritonService::Vmapi),
        }
    }
}

#[async_trait]
impl ServiceDiscovery for VmapiDiscovery {
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

/// Convenient alias for VM list queries.
pub type VmQuery = VMListParams;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use triton_core::uuid::{ImageUuid, OwnerUuid};
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> VmapiClient {
        VmapiClient::new(server.uri()).unwrap()
    }

    #[tokio::test]
    async fn list_vms_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vms"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "uuid": InstanceUuid::new_v4(),
                    "alias": "vm-01",
                    "state": "running"
                }
            ])))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let params = VMListParams {
            owner_uuid: Some(OwnerUuid::new_v4()),
            ..VMListParams::default()
        };
        let vms = client.list_vms(&params).await.unwrap();
        assert_eq!(vms.len(), 1);
        assert_eq!(vms[0].alias.as_deref(), Some("vm-01"));
    }

    #[tokio::test]
    async fn get_vm_not_found() {
        let server = MockServer::start().await;
        let uuid = InstanceUuid::new_v4();

        Mock::given(method("GET"))
            .and(path(format!("/vms/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(404).set_body_string("missing"))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let err = client.get_vm(uuid).await.unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn create_vm_returns_job() {
        let server = MockServer::start().await;
        let owner = OwnerUuid::new_v4();
        let image = ImageUuid::new_v4();

        Mock::given(method("POST"))
            .and(path("/vms"))
            .and(body_json(json!({
                "brand": "joyent",
                "owner_uuid": owner,
                "ram": 1024,
                "image_uuid": image,
                "networks": []
            })))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "uuid": "job-uuid",
                "name": "provision-vm",
                "execution": "running",
                "params": {}
            })))
            .mount(&server)
            .await;

        let request = CreateVMRequest {
            alias: None,
            brand: "joyent".into(),
            owner_uuid: owner,
            ram: 1024,
            cpu_shares: None,
            cpu_cap: None,
            quota: None,
            vcpus: None,
            image_uuid: image,
            server_uuid: None,
            package_uuid: None,
            networks: json!([]),
            tags: None,
            customer_metadata: None,
            internal_metadata: None,
            firewall_enabled: None,
        };

        let client = test_client(&server);
        let job = client.create_vm(&request).await.unwrap();
        assert_eq!(job.execution, "running");
    }

    #[tokio::test]
    async fn batch_action_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/vms/actions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "summary": {"total": 2, "succeeded": 2, "failed": 0},
                "results": []
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let request = BatchVMRequest {
            vm_uuids: vec![InstanceUuid::new_v4()],
            concurrency: 5,
        };
        let response = client.batch_action(&request).await.unwrap();
        assert_eq!(response.summary.succeeded, 2);
    }
}
