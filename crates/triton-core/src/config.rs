//! Configuration structures for Triton clients.
//!
//! This module provides configuration types for connecting to Triton DataCenter services,
//! including service discovery, endpoint configuration, and validation.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;
use validator::Validate;

/// Configuration for a Triton client instance.
///
/// This is the main configuration structure that controls how a Triton client
/// connects to and interacts with Triton DataCenter services.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TritonClientConfig {
    /// SAPI (Services API) base URL
    #[validate(url)]
    pub sapi_url: String,

    /// Optional API key for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sapi_key: Option<String>,

    /// Whether to verify TLS certificates
    #[serde(default = "default_tls_verify")]
    pub tls_verify: bool,

    /// Optional path to custom CA certificate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_ca_cert: Option<std::path::PathBuf>,

    /// Request timeout in seconds
    #[validate(range(min = 1, max = 300))]
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,

    /// Maximum number of retry attempts
    #[validate(range(min = 0, max = 10))]
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Service discovery configuration
    #[validate(nested)]
    #[serde(default)]
    pub service_discovery: ServiceDiscoveryConfig,
}

const fn default_tls_verify() -> bool {
    true
}

const fn default_request_timeout_secs() -> u64 {
    30
}

const fn default_max_retries() -> u32 {
    3
}

impl TritonClientConfig {
    /// Create a new client configuration with required parameters.
    ///
    /// # Arguments
    ///
    /// * `sapi_url` - The base URL for SAPI (e.g., "https://sapi.example.com")
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or validation fails.
    pub fn new(sapi_url: impl Into<String>) -> Result<Self, Error> {
        let config = Self {
            sapi_url: sapi_url.into(),
            sapi_key: None,
            tls_verify: default_tls_verify(),
            tls_ca_cert: None,
            request_timeout_secs: default_request_timeout_secs(),
            max_retries: default_max_retries(),
            service_discovery: ServiceDiscoveryConfig::default(),
        };

        config.validate().map_err(|e| {
            Error::ConfigError(format!("Invalid configuration: {}", e))
        })?;

        Ok(config)
    }

    /// Set the API key for authentication.
    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.sapi_key = Some(api_key.into());
        self
    }

    /// Set whether to verify TLS certificates.
    #[must_use]
    pub const fn with_tls_verify(mut self, verify: bool) -> Self {
        self.tls_verify = verify;
        self
    }

    /// Set custom CA certificate path.
    #[must_use]
    pub fn with_ca_cert(mut self, path: std::path::PathBuf) -> Self {
        self.tls_ca_cert = Some(path);
        self
    }

    /// Set request timeout in seconds.
    #[must_use]
    pub const fn with_timeout(mut self, seconds: u64) -> Self {
        self.request_timeout_secs = seconds;
        self
    }

    /// Set maximum retry attempts.
    #[must_use]
    pub const fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set service discovery configuration.
    #[must_use]
    pub fn with_service_discovery(mut self, config: ServiceDiscoveryConfig) -> Self {
        self.service_discovery = config;
        self
    }

    /// Get the request timeout as a Duration.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    /// Parse and validate the SAPI URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed.
    pub fn parse_sapi_url(&self) -> Result<Url, Error> {
        Url::parse(&self.sapi_url)
            .map_err(|e| Error::ConfigError(format!("Invalid SAPI URL: {}", e)))
    }
}

impl Default for TritonClientConfig {
    fn default() -> Self {
        Self {
            sapi_url: "http://localhost:8080".to_string(),
            sapi_key: None,
            tls_verify: default_tls_verify(),
            tls_ca_cert: None,
            request_timeout_secs: default_request_timeout_secs(),
            max_retries: default_max_retries(),
            service_discovery: ServiceDiscoveryConfig::default(),
        }
    }
}

/// Configuration for service discovery.
///
/// Controls how Triton services are discovered and cached.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServiceDiscoveryConfig {
    /// Whether service discovery is enabled
    #[serde(default = "default_discovery_enabled")]
    pub enabled: bool,

    /// Cache TTL in seconds
    #[validate(range(min = 1, max = 3600))]
    #[serde(default = "default_cache_ttl_secs")]
    pub cache_ttl_secs: u64,

    /// Discovery timeout in seconds
    #[validate(range(min = 1, max = 60))]
    #[serde(default = "default_discovery_timeout_secs")]
    pub timeout_secs: u64,

    /// Number of retry attempts for discovery
    #[validate(range(min = 0, max = 10))]
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,

    /// Static service endpoints (fallback)
    #[serde(default)]
    pub services: ServiceEndpoints,
}

const fn default_discovery_enabled() -> bool {
    true
}

const fn default_cache_ttl_secs() -> u64 {
    300
}

const fn default_discovery_timeout_secs() -> u64 {
    5
}

const fn default_retry_attempts() -> u32 {
    3
}

impl ServiceDiscoveryConfig {
    /// Create a new service discovery configuration with defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enabled: default_discovery_enabled(),
            cache_ttl_secs: default_cache_ttl_secs(),
            timeout_secs: default_discovery_timeout_secs(),
            retry_attempts: default_retry_attempts(),
            services: ServiceEndpoints::new(),
        }
    }

    /// Disable service discovery (use static endpoints only).
    #[must_use]
    pub const fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Set cache TTL in seconds.
    #[must_use]
    pub const fn with_cache_ttl(mut self, seconds: u64) -> Self {
        self.cache_ttl_secs = seconds;
        self
    }

    /// Set discovery timeout in seconds.
    #[must_use]
    pub const fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_secs = seconds;
        self
    }

    /// Set retry attempts.
    #[must_use]
    pub const fn with_retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Get cache TTL as a Duration.
    #[must_use]
    pub const fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache_ttl_secs)
    }

    /// Get timeout as a Duration.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Static service endpoint configurations.
///
/// Provides fallback endpoints when service discovery is unavailable or disabled.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// VMAPI endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vmapi: Option<ServiceEndpointConfig>,

    /// CNAPI endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cnapi: Option<ServiceEndpointConfig>,

    /// NAPI endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub napi: Option<ServiceEndpointConfig>,

    /// IMGAPI endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imgapi: Option<ServiceEndpointConfig>,

    /// PAPI endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub papi: Option<ServiceEndpointConfig>,

    /// FWAPI endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fwapi: Option<ServiceEndpointConfig>,
}

impl ServiceEndpoints {
    /// Create a new empty service endpoints collection.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            vmapi: None,
            cnapi: None,
            napi: None,
            imgapi: None,
            papi: None,
            fwapi: None,
        }
    }

    /// Set VMAPI endpoint.
    #[must_use]
    pub fn with_vmapi(mut self, endpoint: ServiceEndpointConfig) -> Self {
        self.vmapi = Some(endpoint);
        self
    }

    /// Set CNAPI endpoint.
    #[must_use]
    pub fn with_cnapi(mut self, endpoint: ServiceEndpointConfig) -> Self {
        self.cnapi = Some(endpoint);
        self
    }

    /// Set NAPI endpoint.
    #[must_use]
    pub fn with_napi(mut self, endpoint: ServiceEndpointConfig) -> Self {
        self.napi = Some(endpoint);
        self
    }

    /// Set IMGAPI endpoint.
    #[must_use]
    pub fn with_imgapi(mut self, endpoint: ServiceEndpointConfig) -> Self {
        self.imgapi = Some(endpoint);
        self
    }

    /// Set PAPI endpoint.
    #[must_use]
    pub fn with_papi(mut self, endpoint: ServiceEndpointConfig) -> Self {
        self.papi = Some(endpoint);
        self
    }

    /// Set FWAPI endpoint.
    #[must_use]
    pub fn with_fwapi(mut self, endpoint: ServiceEndpointConfig) -> Self {
        self.fwapi = Some(endpoint);
        self
    }
}

/// Configuration for a single service endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServiceEndpointConfig {
    /// Service base URL
    #[validate(url)]
    pub url: String,

    /// Optional timeout override for this service (in seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 300))]
    pub timeout_override_secs: Option<u64>,
}

impl ServiceEndpointConfig {
    /// Create a new service endpoint configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL for the service
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or validation fails.
    pub fn new(url: impl Into<String>) -> Result<Self, Error> {
        let config = Self {
            url: url.into(),
            timeout_override_secs: None,
        };

        config.validate().map_err(|e| {
            Error::ConfigError(format!("Invalid endpoint configuration: {}", e))
        })?;

        Ok(config)
    }

    /// Set timeout override in seconds.
    #[must_use]
    pub const fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_override_secs = Some(seconds);
        self
    }

    /// Get the timeout as a Duration, if set.
    #[must_use]
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout_override_secs.map(Duration::from_secs)
    }

    /// Parse and validate the URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed.
    pub fn parse_url(&self) -> Result<Url, Error> {
        Url::parse(&self.url)
            .map_err(|e| Error::ConfigError(format!("Invalid service URL: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triton_client_config_new() {
        let config = TritonClientConfig::new("https://sapi.example.com").unwrap();
        assert_eq!(config.sapi_url, "https://sapi.example.com");
        assert!(config.tls_verify);
        assert_eq!(config.request_timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_triton_client_config_invalid_url() {
        let result = TritonClientConfig::new("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_triton_client_config_builder() {
        let config = TritonClientConfig::new("https://sapi.example.com")
            .unwrap()
            .with_api_key("test-key")
            .with_tls_verify(false)
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.sapi_key, Some("test-key".to_string()));
        assert!(!config.tls_verify);
        assert_eq!(config.request_timeout_secs, 60);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_triton_client_config_default() {
        let config = TritonClientConfig::default();
        assert_eq!(config.sapi_url, "http://localhost:8080");
        assert!(config.sapi_key.is_none());
        assert!(config.tls_verify);
    }

    #[test]
    fn test_triton_client_config_timeout() {
        let config = TritonClientConfig::new("https://sapi.example.com")
            .unwrap()
            .with_timeout(45);
        assert_eq!(config.timeout(), Duration::from_secs(45));
    }

    #[test]
    fn test_triton_client_config_parse_sapi_url() {
        let config = TritonClientConfig::new("https://sapi.example.com:8080").unwrap();
        let url = config.parse_sapi_url().unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("sapi.example.com"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_service_discovery_config_new() {
        let config = ServiceDiscoveryConfig::new();
        assert!(config.enabled);
        assert_eq!(config.cache_ttl_secs, 300);
        assert_eq!(config.timeout_secs, 5);
        assert_eq!(config.retry_attempts, 3);
    }

    #[test]
    fn test_service_discovery_config_disabled() {
        let config = ServiceDiscoveryConfig::new().disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_service_discovery_config_builder() {
        let config = ServiceDiscoveryConfig::new()
            .with_cache_ttl(600)
            .with_timeout(10)
            .with_retry_attempts(5);

        assert_eq!(config.cache_ttl_secs, 600);
        assert_eq!(config.timeout_secs, 10);
        assert_eq!(config.retry_attempts, 5);
    }

    #[test]
    fn test_service_discovery_config_durations() {
        let config = ServiceDiscoveryConfig::new()
            .with_cache_ttl(600)
            .with_timeout(10);

        assert_eq!(config.cache_ttl(), Duration::from_secs(600));
        assert_eq!(config.timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_service_endpoints_new() {
        let endpoints = ServiceEndpoints::new();
        assert!(endpoints.vmapi.is_none());
        assert!(endpoints.cnapi.is_none());
        assert!(endpoints.napi.is_none());
    }

    #[test]
    fn test_service_endpoints_builder() {
        let vmapi = ServiceEndpointConfig::new("http://vmapi:80").unwrap();
        let cnapi = ServiceEndpointConfig::new("http://cnapi:80").unwrap();

        let endpoints = ServiceEndpoints::new()
            .with_vmapi(vmapi)
            .with_cnapi(cnapi);

        assert!(endpoints.vmapi.is_some());
        assert!(endpoints.cnapi.is_some());
        assert!(endpoints.napi.is_none());
    }

    #[test]
    fn test_service_endpoint_config_new() {
        let config = ServiceEndpointConfig::new("http://service:8080").unwrap();
        assert_eq!(config.url, "http://service:8080");
        assert!(config.timeout_override_secs.is_none());
    }

    #[test]
    fn test_service_endpoint_config_invalid_url() {
        let result = ServiceEndpointConfig::new("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_service_endpoint_config_with_timeout() {
        let config = ServiceEndpointConfig::new("http://service:8080")
            .unwrap()
            .with_timeout(60);

        assert_eq!(config.timeout_override_secs, Some(60));
        assert_eq!(config.timeout(), Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_service_endpoint_config_parse_url() {
        let config = ServiceEndpointConfig::new("http://service:8080").unwrap();
        let url = config.parse_url().unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("service"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_config_serialization() {
        let config = TritonClientConfig::new("https://sapi.example.com")
            .unwrap()
            .with_api_key("test-key");

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TritonClientConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.sapi_url, deserialized.sapi_url);
        assert_eq!(config.sapi_key, deserialized.sapi_key);
    }

    #[test]
    fn test_service_discovery_serialization() {
        let config = ServiceDiscoveryConfig::new()
            .with_cache_ttl(600)
            .with_timeout(10);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ServiceDiscoveryConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.cache_ttl_secs, deserialized.cache_ttl_secs);
        assert_eq!(config.timeout_secs, deserialized.timeout_secs);
    }

    #[test]
    fn test_service_endpoint_serialization() {
        let config = ServiceEndpointConfig::new("http://service:8080")
            .unwrap()
            .with_timeout(60);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ServiceEndpointConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.url, deserialized.url);
        assert_eq!(config.timeout_override_secs, deserialized.timeout_override_secs);
    }

    #[test]
    fn test_config_validation_timeout_range() {
        let mut config = TritonClientConfig::default();
        config.request_timeout_secs = 0;
        assert!(config.validate().is_err());

        config.request_timeout_secs = 301;
        assert!(config.validate().is_err());

        config.request_timeout_secs = 30;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_retries_range() {
        let mut config = TritonClientConfig::default();
        config.max_retries = 11;
        assert!(config.validate().is_err());

        config.max_retries = 3;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_discovery_config_validation_cache_ttl_range() {
        let mut config = ServiceDiscoveryConfig::default();
        config.cache_ttl_secs = 0;
        assert!(config.validate().is_err());

        config.cache_ttl_secs = 3601;
        assert!(config.validate().is_err());

        config.cache_ttl_secs = 300;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_discovery_config_validation_timeout_range() {
        let mut config = ServiceDiscoveryConfig::default();
        config.timeout_secs = 0;
        assert!(config.validate().is_err());

        config.timeout_secs = 61;
        assert!(config.validate().is_err());

        config.timeout_secs = 5;
        assert!(config.validate().is_ok());
    }
}
