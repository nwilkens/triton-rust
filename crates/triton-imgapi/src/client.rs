//! Asynchronous IMGAPI client implementation.

use crate::models::{
    CreateImageRequest, ExportImageRequest, Image, ImageAction, ImageImportRequest,
    ImageListParams, UpdateImageRequest,
};
use crate::Result;
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use triton_core::client::{
    ClientConfig, RetryPolicy, ServiceClient, ServiceClientBuilder, IMGAPI_DEFAULT_TIMEOUT,
};
use triton_core::services::{DiscoveryStatus, ServiceDiscovery, ServiceDiscoveryProxy};
use triton_core::types::TritonService;
use triton_core::uuid::ImageUuid;
use triton_core::Error;
use url::Url;

const USER_AGENT: &str = concat!("triton-imgapi/", env!("CARGO_PKG_VERSION"));

/// Builder for [`ImgapiClient`].
#[derive(Debug, Clone)]
pub struct ImgapiClientBuilder {
    inner: ServiceClientBuilder,
}

impl ImgapiClientBuilder {
    /// Create a builder for the specified base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let builder = ServiceClientBuilder::new(
            TritonService::Imgapi,
            base_url,
            Duration::from_secs(IMGAPI_DEFAULT_TIMEOUT),
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
    pub fn build(self) -> Result<ImgapiClient> {
        let inner = self.inner.build()?;
        Ok(ImgapiClient { inner })
    }
}

/// Asynchronous IMGAPI client.
#[derive(Clone)]
pub struct ImgapiClient {
    inner: ServiceClient,
}

impl ImgapiClient {
    /// Construct a client directly from the base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        ImgapiClientBuilder::new(base_url)?.build()
    }

    /// Return the base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        self.inner.base_url()
    }

    /// List images.
    pub async fn list_images(&self, params: &ImageListParams) -> Result<Vec<Image>> {
        self.send_json::<(), Vec<Image>>(Method::GET, "images", None, &params.to_pairs())
            .await
    }

    /// Fetch a single image by UUID.
    pub async fn get_image(&self, uuid: ImageUuid) -> Result<Image> {
        let path = format!("images/{uuid}");
        self.send_json::<(), Image>(Method::GET, &path, None, &[])
            .await
    }

    /// Create a new image record.
    pub async fn create_image(&self, request: &CreateImageRequest) -> Result<Image> {
        self.send_json(Method::POST, "images", Some(request), &[])
            .await
    }

    /// Update an existing image.
    pub async fn update_image(
        &self,
        uuid: ImageUuid,
        request: &UpdateImageRequest,
    ) -> Result<Image> {
        let path = format!("images/{uuid}");
        self.send_json(Method::PUT, &path, Some(request), &[]).await
    }

    /// Delete an image.
    pub async fn delete_image(&self, uuid: ImageUuid) -> Result<()> {
        let path = format!("images/{uuid}");
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

    /// Perform an action (activate/disable/enable) on an image.
    pub async fn perform_action(&self, uuid: ImageUuid, action: ImageAction) -> Result<Image> {
        let path = format!("images/{uuid}/{}", action.as_str());
        let empty = Value::Object(Default::default());
        self.send_json(Method::POST, &path, Some(&empty), &[]).await
    }

    /// Import an image from an external source.
    pub async fn import_image(&self, request: &ImageImportRequest) -> Result<Image> {
        self.send_json(Method::POST, "images/import", Some(request), &[])
            .await
    }

    /// Export an image to Manta or other storage.
    pub async fn export_image(
        &self,
        uuid: ImageUuid,
        request: &ExportImageRequest,
    ) -> Result<Image> {
        let path = format!("images/{uuid}/export");
        self.send_json(Method::POST, &path, Some(request), &[])
            .await
    }

    /// Download the binary contents of an image file.
    pub async fn download_image_file(&self, uuid: ImageUuid) -> Result<Bytes> {
        let path = format!("images/{uuid}/file");
        let response = self
            .inner
            .execute_with_retry(
                Method::GET,
                &path,
                &[],
                |request| request.header("Accept", "application/octet-stream"),
                map_status_to_error,
            )
            .await?;

        response.bytes().await.map_err(Error::from)
    }

    /// Upload an image file.
    pub async fn upload_image_file(
        &self,
        uuid: ImageUuid,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<()> {
        let path = format!("images/{uuid}/file");
        self.inner
            .execute_with_retry(
                Method::PUT,
                &path,
                &[],
                move |request| {
                    let mut request = request.body(data.clone());
                    if let Some(ct) = content_type {
                        request = request.header("Content-Type", ct);
                    }
                    request
                },
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
            Error::InvalidRequest(format!("IMGAPI authentication failed: {text}"))
        }
        StatusCode::CONFLICT => Error::Conflict(text),
        StatusCode::TOO_MANY_REQUESTS
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => {
            Error::ServiceUnavailable(format!("IMGAPI temporarily unavailable: {text}"))
        }
        status if status.is_server_error() => {
            Error::ServiceUnavailable(format!("IMGAPI server error {status}: {text}"))
        }
        _ => Error::HttpError(format!("IMGAPI error {status}: {text}")),
    }
}

/// Discovery adapter that relies on SAPI for IMGAPI endpoints.
pub struct ImgapiDiscovery {
    proxy: ServiceDiscoveryProxy,
}

impl ImgapiDiscovery {
    /// Create a discovery wrapper from an existing SAPI discovery instance.
    #[must_use]
    pub fn new(sapi_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            proxy: ServiceDiscoveryProxy::for_service(sapi_discovery, TritonService::Imgapi),
        }
    }
}

#[async_trait]
impl ServiceDiscovery for ImgapiDiscovery {
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
    use triton_core::uuid::OwnerUuid;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(server: &MockServer) -> ImgapiClient {
        ImgapiClient::new(server.uri()).unwrap()
    }

    #[tokio::test]
    async fn list_images_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/images"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "uuid": ImageUuid::new_v4(),
                    "name": "ubuntu-22.04",
                    "os": "linux",
                    "type": "zone-dataset",
                    "state": "active"
                }
            ])))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let images = client
            .list_images(&ImageListParams::default())
            .await
            .unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].name, "ubuntu-22.04");
    }

    #[tokio::test]
    async fn get_image_not_found() {
        let server = MockServer::start().await;
        let uuid = ImageUuid::new_v4();

        Mock::given(method("GET"))
            .and(path(format!("/images/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(404).set_body_string("missing"))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let err = client.get_image(uuid).await.unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn create_image_returns_image() {
        let server = MockServer::start().await;
        let owner = OwnerUuid::new_v4();

        Mock::given(method("POST"))
            .and(path("/images"))
            .and(header("content-type", "application/json"))
            .and(body_json(json!({
                "name": "ubuntu",
                "version": "22.04",
                "os": "linux",
                "type": "zone-dataset",
                "owner": owner
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "uuid": ImageUuid::new_v4(),
                "name": "ubuntu",
                "version": "22.04",
                "os": "linux",
                "type": "zone-dataset",
                "state": "unactivated",
                "owner": owner
            })))
            .mount(&server)
            .await;

        let request = CreateImageRequest {
            name: "ubuntu".into(),
            version: "22.04".into(),
            os: "linux".into(),
            r#type: "zone-dataset".into(),
            description: None,
            homepage: None,
            public: None,
            owner: Some(owner),
            tags: None,
            origin: None,
            files: None,
            requirements: None,
            users: None,
            billing_tags: None,
            traits: None,
            channels: None,
            nic_driver: None,
            disk_driver: None,
            cpu_type: None,
            image_size: None,
            virtual_size: None,
            min_memory: None,
            min_disk: None,
            generate_passwords: None,
            inherited_directories: None,
        };

        let client = test_client(&server);
        let image = client.create_image(&request).await.unwrap();
        assert_eq!(image.state, "unactivated");
    }

    #[tokio::test]
    async fn perform_action_posts_to_action_endpoint() {
        let server = MockServer::start().await;
        let uuid = ImageUuid::new_v4();

        Mock::given(method("POST"))
            .and(path(format!("/images/{uuid}/activate").as_str()))
            .and(body_json(json!({})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "uuid": uuid,
                "name": "ubuntu",
                "os": "linux",
                "type": "zone-dataset",
                "state": "active"
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let image = client
            .perform_action(uuid, ImageAction::Activate)
            .await
            .unwrap();
        assert_eq!(image.state, "active");
    }

    #[tokio::test]
    async fn delete_image_handles_no_content() {
        let server = MockServer::start().await;
        let uuid = ImageUuid::new_v4();

        Mock::given(method("DELETE"))
            .and(path(format!("/images/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = test_client(&server);
        client.delete_image(uuid).await.unwrap();
    }

    #[tokio::test]
    async fn download_image_file_returns_bytes() {
        let server = MockServer::start().await;
        let uuid = ImageUuid::new_v4();

        Mock::given(method("GET"))
            .and(path(format!("/images/{uuid}/file").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"binary"))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let bytes = client.download_image_file(uuid).await.unwrap();
        assert_eq!(bytes, Bytes::from_static(b"binary"));
    }

    #[tokio::test]
    async fn upload_image_file_sends_bytes() {
        let server = MockServer::start().await;
        let uuid = ImageUuid::new_v4();

        Mock::given(method("PUT"))
            .and(path(format!("/images/{uuid}/file").as_str()))
            .and(header("content-type", "application/octet-stream"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;

        let client = test_client(&server);
        client
            .upload_image_file(
                uuid,
                Bytes::from_static(b"data"),
                Some("application/octet-stream"),
            )
            .await
            .unwrap();
    }

    struct MockDiscovery;

    #[async_trait]
    impl ServiceDiscovery for MockDiscovery {
        async fn discover_service(&self, _: &str) -> Result<Vec<String>> {
            Ok(vec!["http://imgapi.example.com".into()])
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
    async fn imgapi_discovery_delegates_to_proxy() {
        let discovery = Arc::new(MockDiscovery);
        let imgapi = ImgapiDiscovery::new(discovery);
        let endpoints = imgapi
            .discover_service(TritonService::Imgapi.name())
            .await
            .unwrap();
        assert_eq!(endpoints.len(), 1);
    }
}
