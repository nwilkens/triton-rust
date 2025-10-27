//! Asynchronous FWAPI client implementation.

use crate::models::{
    CreateFirewallRuleRequest, FirewallRule, FirewallRuleListParams, UpdateFirewallRuleRequest,
};
use crate::Result;
use async_trait::async_trait;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use triton_core::client::{
    ClientConfig, RetryPolicy, ServiceClient, ServiceClientBuilder, FWAPI_DEFAULT_TIMEOUT,
};
use triton_core::services::{DiscoveryStatus, ServiceDiscovery, ServiceDiscoveryProxy};
use triton_core::types::TritonService;
use triton_core::uuid::FirewallRuleUuid;
use triton_core::Error;
use url::Url;

const USER_AGENT: &str = concat!("triton-fwapi/", env!("CARGO_PKG_VERSION"));

/// Builder for [`FwapiClient`].
#[derive(Debug, Clone)]
pub struct FwapiClientBuilder {
    inner: ServiceClientBuilder,
}

impl FwapiClientBuilder {
    /// Create a builder for the specified base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let builder = ServiceClientBuilder::new(
            TritonService::Fwapi,
            base_url,
            Duration::from_secs(FWAPI_DEFAULT_TIMEOUT),
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
    pub fn build(self) -> Result<FwapiClient> {
        let inner = self.inner.build()?;
        Ok(FwapiClient { inner })
    }
}

/// Asynchronous FWAPI client.
#[derive(Clone)]
pub struct FwapiClient {
    inner: ServiceClient,
}

impl FwapiClient {
    /// Construct a client directly from the base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        FwapiClientBuilder::new(base_url)?.build()
    }

    /// Return the base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        self.inner.base_url()
    }

    /// List firewall rules with optional filters.
    pub async fn list_rules(&self, params: &FirewallRuleListParams) -> Result<Vec<FirewallRule>> {
        self.send_json::<(), Vec<FirewallRule>>(Method::GET, "rules", None, &params.to_pairs())
            .await
    }

    /// Fetch a single firewall rule by UUID.
    pub async fn get_rule(&self, uuid: FirewallRuleUuid) -> Result<FirewallRule> {
        let path = format!("rules/{uuid}");
        self.send_json::<(), FirewallRule>(Method::GET, &path, None, &[])
            .await
    }

    /// Create a new firewall rule.
    pub async fn create_rule(&self, request: &CreateFirewallRuleRequest) -> Result<FirewallRule> {
        self.send_json(Method::POST, "rules", Some(request), &[])
            .await
    }

    /// Update an existing firewall rule.
    pub async fn update_rule(
        &self,
        uuid: FirewallRuleUuid,
        request: &UpdateFirewallRuleRequest,
    ) -> Result<FirewallRule> {
        let path = format!("rules/{uuid}");
        self.send_json(Method::PUT, &path, Some(request), &[]).await
    }

    /// Delete a firewall rule.
    pub async fn delete_rule(&self, uuid: FirewallRuleUuid) -> Result<()> {
        let path = format!("rules/{uuid}");
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
            Error::InvalidRequest(format!("FWAPI authentication failed: {text}"))
        }
        StatusCode::CONFLICT => Error::Conflict(text),
        StatusCode::TOO_MANY_REQUESTS
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => {
            Error::ServiceUnavailable(format!("FWAPI temporarily unavailable: {text}"))
        }
        status if status.is_server_error() => {
            Error::ServiceUnavailable(format!("FWAPI server error {status}: {text}"))
        }
        _ => Error::HttpError(format!("FWAPI error {status}: {text}")),
    }
}

/// Discovery adapter that relies on SAPI for FWAPI endpoints.
pub struct FwapiDiscovery {
    proxy: ServiceDiscoveryProxy,
}

impl FwapiDiscovery {
    /// Create a discovery wrapper from an existing SAPI discovery instance.
    #[must_use]
    pub fn new(sapi_discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            proxy: ServiceDiscoveryProxy::for_service(sapi_discovery, TritonService::Fwapi),
        }
    }
}

#[async_trait]
impl ServiceDiscovery for FwapiDiscovery {
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

    fn test_client(server: &MockServer) -> FwapiClient {
        FwapiClient::new(server.uri()).unwrap()
    }

    #[tokio::test]
    async fn list_firewall_rules_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rules"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "uuid": FirewallRuleUuid::new_v4(),
                    "rule": "ALLOW everything",
                    "enabled": true,
                    "version": "1"
                }
            ])))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let rules = client
            .list_rules(&FirewallRuleListParams::default())
            .await
            .unwrap();
        assert_eq!(rules.len(), 1);
        assert!(rules[0].enabled);
    }

    #[tokio::test]
    async fn get_firewall_rule_not_found() {
        let server = MockServer::start().await;
        let uuid = FirewallRuleUuid::new_v4();

        Mock::given(method("GET"))
            .and(path(format!("/rules/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(404).set_body_string("missing"))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let err = client.get_rule(uuid).await.unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn create_firewall_rule_returns_rule() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rules"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "uuid": FirewallRuleUuid::new_v4(),
                "rule": "ALLOW everything",
                "enabled": true,
                "version": "1"
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let request = CreateFirewallRuleRequest {
            rule: "ALLOW everything".into(),
            enabled: Some(true),
            description: None,
            owner_uuid: None,
            global: None,
            vms: None,
        };

        let rule = client.create_rule(&request).await.unwrap();
        assert_eq!(rule.rule, "ALLOW everything");
        assert!(rule.enabled);
    }

    #[tokio::test]
    async fn update_firewall_rule_returns_rule() {
        let server = MockServer::start().await;
        let uuid = FirewallRuleUuid::new_v4();

        Mock::given(method("PUT"))
            .and(path(format!("/rules/{uuid}").as_str()))
            .and(body_json(json!({
                "description": "Updated rule"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "uuid": uuid,
                "rule": "ALLOW everything",
                "enabled": true,
                "version": "2",
                "description": "Updated rule"
            })))
            .mount(&server)
            .await;

        let client = test_client(&server);
        let request = UpdateFirewallRuleRequest {
            description: Some("Updated rule".into()),
            ..UpdateFirewallRuleRequest::default()
        };

        let rule = client.update_rule(uuid, &request).await.unwrap();
        assert_eq!(rule.description.as_deref(), Some("Updated rule"));
        assert_eq!(rule.version, "2");
    }

    #[tokio::test]
    async fn delete_firewall_rule_handles_no_content() {
        let server = MockServer::start().await;
        let uuid = FirewallRuleUuid::new_v4();

        Mock::given(method("DELETE"))
            .and(path(format!("/rules/{uuid}").as_str()))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = test_client(&server);
        client.delete_rule(uuid).await.unwrap();
    }

    struct MockDiscovery;

    #[async_trait]
    impl ServiceDiscovery for MockDiscovery {
        async fn discover_service(&self, _: &str) -> Result<Vec<String>> {
            Ok(vec!["http://fwapi.example.com".into()])
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
    async fn fwapi_discovery_delegates_to_proxy() {
        let discovery = Arc::new(MockDiscovery);
        let fwapi = FwapiDiscovery::new(discovery);
        let endpoints = fwapi
            .discover_service(TritonService::Fwapi.name())
            .await
            .unwrap();
        assert_eq!(endpoints.len(), 1);
    }
}
