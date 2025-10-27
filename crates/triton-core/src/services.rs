//! Service discovery and integration patterns.
//!
//! This module provides types and traits for discovering and managing Triton DataCenter services,
//! including UFDS credentials and discovery status tracking.

use crate::error::Error;
use crate::types::TritonService;
use crate::uuid::AppUuid;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// UFDS (User, Forensics, and Directory Services) credentials.
///
/// These credentials are used to authenticate with UFDS, Triton's LDAP-based
/// directory service for user and account management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UfdsCredentials {
    /// UFDS admin login (DN - Distinguished Name)
    pub admin_login: String,

    /// UFDS admin password
    #[serde(skip_serializing)]
    pub admin_password: String,

    /// Application UUID that owns these credentials
    pub application_uuid: AppUuid,
}

impl UfdsCredentials {
    /// Create new UFDS credentials.
    ///
    /// # Arguments
    ///
    /// * `admin_login` - The LDAP DN for the admin account
    /// * `admin_password` - The admin password
    /// * `application_uuid` - The UUID of the application these credentials belong to
    #[must_use]
    pub const fn new(
        admin_login: String,
        admin_password: String,
        application_uuid: AppUuid,
    ) -> Self {
        Self {
            admin_login,
            admin_password,
            application_uuid,
        }
    }

    /// Get the LDAP bind DN.
    #[must_use]
    pub fn bind_dn(&self) -> &str {
        &self.admin_login
    }

    /// Get the LDAP bind password.
    #[must_use]
    pub fn bind_password(&self) -> &str {
        &self.admin_password
    }

    /// Get the application UUID.
    #[must_use]
    pub const fn app_uuid(&self) -> &AppUuid {
        &self.application_uuid
    }
}

/// Discovery status for monitoring service discovery operations.
///
/// Tracks the health and performance of service discovery, including cache statistics
/// and error tracking.
#[derive(Debug, Clone)]
pub struct DiscoveryStatus {
    /// When discovery was last attempted
    pub last_discovery_at: Option<Instant>,

    /// When discovery last succeeded
    pub last_success_at: Option<Instant>,

    /// Last error message, if any
    pub last_error: Option<String>,

    /// Number of services successfully discovered
    pub discovered_services: usize,

    /// List of services that failed discovery
    pub failed_services: Vec<String>,

    /// Number of cache hits
    pub cache_hits: u64,

    /// Number of cache misses
    pub cache_misses: u64,
}

impl DiscoveryStatus {
    /// Create a new discovery status with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_discovery_at: None,
            last_success_at: None,
            last_error: None,
            discovered_services: 0,
            failed_services: Vec::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Record a successful discovery.
    #[must_use]
    pub fn with_success(mut self, service_count: usize) -> Self {
        let now = Instant::now();
        self.last_discovery_at = Some(now);
        self.last_success_at = Some(now);
        self.discovered_services = service_count;
        self.last_error = None;
        self
    }

    /// Record a failed discovery.
    #[must_use]
    pub fn with_error(mut self, error: String, failed_service: Option<String>) -> Self {
        self.last_discovery_at = Some(Instant::now());
        self.last_error = Some(error);
        if let Some(service) = failed_service {
            self.failed_services.push(service);
        }
        self
    }

    /// Update cache statistics.
    #[must_use]
    pub const fn with_cache_stats(mut self, hits: u64, misses: u64) -> Self {
        self.cache_hits = hits;
        self.cache_misses = misses;
        self
    }

    /// Calculate cache hit ratio (0.0 to 1.0).
    #[must_use]
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Check if discovery is healthy (no recent errors).
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        self.last_error.is_none() && self.discovered_services > 0
    }

    /// Get time since last successful discovery.
    #[must_use]
    pub fn time_since_last_success(&self) -> Option<std::time::Duration> {
        self.last_success_at.map(|t| t.elapsed())
    }

    /// Get time since last discovery attempt.
    #[must_use]
    pub fn time_since_last_attempt(&self) -> Option<std::time::Duration> {
        self.last_discovery_at.map(|t| t.elapsed())
    }
}

impl Default for DiscoveryStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Service discovery abstraction trait.
///
/// Defines the interface for discovering Triton service endpoints. This trait can be
/// implemented by different discovery mechanisms (SAPI-based, static configuration, etc.).
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Discover endpoints for a specific service.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service to discover (e.g., "vmapi", "cnapi")
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails or the service is not found.
    async fn discover_service(&self, service_name: &str) -> crate::Result<Vec<String>>;

    /// Discover all available services.
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails.
    async fn discover_all_services(&self) -> crate::Result<Vec<String>>;

    /// Get the current discovery status.
    fn get_status(&self) -> DiscoveryStatus;

    /// Clear any cached discovery data.
    fn clear_cache(&self);

    /// Check if a service is available.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service to check
    async fn is_service_available(&self, service_name: &str) -> bool {
        self.discover_service(service_name).await.is_ok()
    }
}

/// Generic discovery proxy that records discovery status while delegating to another implementation.
#[derive(Clone)]
pub struct ServiceDiscoveryProxy {
    inner: Arc<dyn ServiceDiscovery>,
    status: Arc<RwLock<DiscoveryStatus>>,
    service_name: String,
}

impl ServiceDiscoveryProxy {
    /// Create a proxy for a specific service name.
    #[must_use]
    pub fn new(inner: Arc<dyn ServiceDiscovery>, service_name: impl Into<String>) -> Self {
        Self {
            inner,
            status: Arc::new(RwLock::new(DiscoveryStatus::new())),
            service_name: service_name.into(),
        }
    }

    /// Create a proxy for a [`TritonService`].
    #[must_use]
    pub fn for_service(inner: Arc<dyn ServiceDiscovery>, service: TritonService) -> Self {
        Self::new(inner, service.name())
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

    /// Return the service name associated with this proxy.
    #[must_use]
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

#[async_trait::async_trait]
impl ServiceDiscovery for ServiceDiscoveryProxy {
    async fn discover_service(&self, service_name: &str) -> crate::Result<Vec<String>> {
        match self.inner.discover_service(service_name).await {
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

    async fn discover_all_services(&self) -> crate::Result<Vec<String>> {
        self.inner.discover_service(&self.service_name).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_ufds_credentials_new() {
        let uuid = AppUuid::new_v4();
        let creds = UfdsCredentials::new(
            "cn=admin,dc=example,dc=com".to_string(),
            "secret".to_string(),
            uuid,
        );

        assert_eq!(creds.admin_login, "cn=admin,dc=example,dc=com");
        assert_eq!(creds.admin_password, "secret");
        assert_eq!(creds.application_uuid, uuid);
    }

    #[test]
    fn test_ufds_credentials_accessors() {
        let uuid = AppUuid::new_v4();
        let creds = UfdsCredentials::new(
            "cn=admin,dc=example,dc=com".to_string(),
            "secret".to_string(),
            uuid,
        );

        assert_eq!(creds.bind_dn(), "cn=admin,dc=example,dc=com");
        assert_eq!(creds.bind_password(), "secret");
        assert_eq!(creds.app_uuid(), &uuid);
    }

    #[test]
    fn test_ufds_credentials_serialization() {
        let uuid = AppUuid::new_v4();
        let creds = UfdsCredentials::new(
            "cn=admin,dc=example,dc=com".to_string(),
            "secret".to_string(),
            uuid,
        );

        let json = serde_json::to_string(&creds).unwrap();
        // Password should not be serialized
        assert!(!json.contains("secret"));
        assert!(json.contains("cn=admin,dc=example,dc=com"));
    }

    #[test]
    fn test_discovery_status_new() {
        let status = DiscoveryStatus::new();
        assert!(status.last_discovery_at.is_none());
        assert!(status.last_success_at.is_none());
        assert!(status.last_error.is_none());
        assert_eq!(status.discovered_services, 0);
        assert_eq!(status.failed_services.len(), 0);
        assert_eq!(status.cache_hits, 0);
        assert_eq!(status.cache_misses, 0);
    }

    #[test]
    fn test_discovery_status_default() {
        let status = DiscoveryStatus::default();
        assert!(status.last_discovery_at.is_none());
        assert_eq!(status.discovered_services, 0);
    }

    #[test]
    fn test_discovery_status_with_success() {
        let status = DiscoveryStatus::new().with_success(5);
        assert!(status.last_discovery_at.is_some());
        assert!(status.last_success_at.is_some());
        assert_eq!(status.discovered_services, 5);
        assert!(status.last_error.is_none());
    }

    #[test]
    fn test_discovery_status_with_error() {
        let status =
            DiscoveryStatus::new().with_error("Test error".to_string(), Some("vmapi".to_string()));
        assert!(status.last_discovery_at.is_some());
        assert_eq!(status.last_error, Some("Test error".to_string()));
        assert_eq!(status.failed_services, vec!["vmapi"]);
    }

    #[test]
    fn test_discovery_status_cache_stats() {
        let status = DiscoveryStatus::new().with_cache_stats(10, 5);
        assert_eq!(status.cache_hits, 10);
        assert_eq!(status.cache_misses, 5);
    }

    #[test]
    fn test_discovery_status_cache_hit_ratio() {
        let status1 = DiscoveryStatus::new().with_cache_stats(10, 5);
        assert!((status1.cache_hit_ratio() - 0.666_666).abs() < 0.001);

        let status2 = DiscoveryStatus::new().with_cache_stats(0, 0);
        assert_eq!(status2.cache_hit_ratio(), 0.0);

        let status3 = DiscoveryStatus::new().with_cache_stats(10, 0);
        assert_eq!(status3.cache_hit_ratio(), 1.0);
    }

    #[test]
    fn test_discovery_status_is_healthy() {
        let healthy = DiscoveryStatus::new().with_success(5);
        assert!(healthy.is_healthy());

        let unhealthy = DiscoveryStatus::new().with_error("Error".to_string(), None);
        assert!(!unhealthy.is_healthy());

        let no_services = DiscoveryStatus::new();
        assert!(!no_services.is_healthy());
    }

    #[test]
    fn test_discovery_status_time_tracking() {
        let status = DiscoveryStatus::new().with_success(5);

        // Should have timestamps
        assert!(status.time_since_last_success().is_some());
        assert!(status.time_since_last_attempt().is_some());

        // Empty status should have None
        let empty = DiscoveryStatus::new();
        assert!(empty.time_since_last_success().is_none());
        assert!(empty.time_since_last_attempt().is_none());
    }

    #[test]
    fn test_discovery_status_builder_chain() {
        let status = DiscoveryStatus::new()
            .with_success(10)
            .with_cache_stats(50, 10);

        assert_eq!(status.discovered_services, 10);
        assert_eq!(status.cache_hits, 50);
        assert_eq!(status.cache_misses, 10);
        assert!(status.is_healthy());
    }

    #[tokio::test]
    async fn test_service_discovery_mock() {
        let mut mock = MockServiceDiscovery::new();

        mock.expect_discover_service()
            .with(mockall::predicate::eq("vmapi"))
            .times(1)
            .returning(|_| Ok(vec!["http://vmapi:80".to_string()]));

        mock.expect_get_status()
            .times(1)
            .returning(|| DiscoveryStatus::new().with_success(1));

        let result = mock.discover_service("vmapi").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["http://vmapi:80"]);

        let status = mock.get_status();
        assert_eq!(status.discovered_services, 1);
    }

    #[tokio::test]
    async fn test_service_discovery_proxy_records_success() {
        let mut mock = MockServiceDiscovery::new();

        mock.expect_discover_service()
            .with(mockall::predicate::eq("imgapi"))
            .returning(|_| Ok(vec!["http://imgapi:80".to_string()]));

        let proxy = ServiceDiscoveryProxy::for_service(Arc::new(mock), TritonService::Imgapi);
        let endpoints = proxy.discover_service("imgapi").await.unwrap();
        assert_eq!(endpoints, vec!["http://imgapi:80"]);

        let status = proxy.get_status();
        assert_eq!(status.discovered_services, 1);
        assert!(status.last_error.is_none());
    }
}
