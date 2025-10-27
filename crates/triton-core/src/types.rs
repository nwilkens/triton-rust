//! Core Triton domain types.
//!
//! This module provides fundamental types for Triton DataCenter operations,
//! including service enumeration, endpoint management, and transport protocols.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

/// Constants for Triton services
pub const DISCOVERY_APP_NAME: &str = "sdc";
/// Default HTTP port
pub const DEFAULT_HTTP_PORT: u16 = 80;
/// Default HTTPS port
pub const DEFAULT_HTTPS_PORT: u16 = 443;
/// Default LDAPS port
pub const DEFAULT_LDAPS_PORT: u16 = 636;

/// Supported Triton services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TritonService {
    /// Virtual Machine API
    Vmapi,
    /// Compute Node API
    Cnapi,
    /// Network API
    Napi,
    /// Image API
    Imgapi,
    /// Package API
    Papi,
    /// Firewall API
    Fwapi,
    /// Service API (for service discovery)
    Sapi,
    /// User/Directory Service (LDAP)
    Ufds,
    /// Monitoring service
    Amon,
    /// Workflow API
    Workflow,
}

impl TritonService {
    /// Returns the service name as a string.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Vmapi => "vmapi",
            Self::Cnapi => "cnapi",
            Self::Napi => "napi",
            Self::Imgapi => "imgapi",
            Self::Papi => "papi",
            Self::Fwapi => "fwapi",
            Self::Sapi => "sapi",
            Self::Ufds => "ufds",
            Self::Amon => "amon",
            Self::Workflow => "workflow",
        }
    }

    /// Returns all available services.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Vmapi,
            Self::Cnapi,
            Self::Napi,
            Self::Imgapi,
            Self::Papi,
            Self::Fwapi,
            Self::Sapi,
            Self::Ufds,
            Self::Amon,
            Self::Workflow,
        ]
    }

    /// Returns the default port for the service.
    #[must_use]
    pub const fn default_port(&self) -> u16 {
        match self {
            Self::Ufds => DEFAULT_LDAPS_PORT,
            _ => DEFAULT_HTTP_PORT,
        }
    }

    /// Returns the default transport type for the service.
    #[must_use]
    pub const fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }
}

impl FromStr for TritonService {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "vmapi" => Ok(Self::Vmapi),
            "cnapi" => Ok(Self::Cnapi),
            "napi" => Ok(Self::Napi),
            "imgapi" => Ok(Self::Imgapi),
            "papi" => Ok(Self::Papi),
            "fwapi" => Ok(Self::Fwapi),
            "sapi" => Ok(Self::Sapi),
            "ufds" => Ok(Self::Ufds),
            "amon" => Ok(Self::Amon),
            "workflow" => Ok(Self::Workflow),
            _ => Err(Error::InvalidRequest(format!("Unknown service: {s}"))),
        }
    }
}

impl std::fmt::Display for TritonService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Type of transport protocol used by an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportType {
    /// TCP transport
    Tcp,
    /// HTTP transport
    Http,
    /// HTTPS transport
    Https,
    /// gRPC transport
    Grpc,
}

/// Health check configuration for an endpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckConfig {
    /// Whether health checks are enabled
    pub enabled: bool,
    /// Interval between health checks
    pub interval: Duration,
    /// Timeout for each health check
    pub timeout: Duration,
    /// Optional path for HTTP/HTTPS health checks
    pub path: Option<String>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            path: None,
        }
    }
}

/// Details about a specific endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointDetails {
    /// Load balancing weight
    pub weight: Option<u32>,
    /// Priority (lower is higher priority)
    pub priority: Option<u32>,
    /// Region identifier
    pub region: Option<String>,
    /// Availability zone
    pub zone: Option<String>,
    /// Tags for filtering/selection
    pub tags: Vec<String>,
}

impl Default for EndpointDetails {
    fn default() -> Self {
        Self {
            weight: None,
            priority: None,
            region: None,
            zone: None,
            tags: Vec::new(),
        }
    }
}

/// Represents a single service endpoint.
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    /// Unique identifier for this endpoint
    pub id: String,
    /// Name of the service
    pub service_name: String,
    /// Network address
    pub address: SocketAddr,
    /// Transport protocol
    pub transport: TransportType,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Optional health check configuration
    pub health_check: Option<HealthCheckConfig>,
    /// Last time this endpoint was seen/updated
    pub last_seen: Instant,
}

impl ServiceEndpoint {
    /// Creates a new service endpoint.
    #[must_use]
    pub fn new(id: String, service_name: String, address: SocketAddr) -> Self {
        Self {
            id,
            service_name,
            address,
            transport: TransportType::Tcp,
            metadata: HashMap::new(),
            health_check: None,
            last_seen: Instant::now(),
        }
    }

    /// Checks if the endpoint is healthy based on a staleness threshold.
    #[must_use]
    pub fn is_healthy(&self, stale_threshold: Duration) -> bool {
        Instant::now().duration_since(self.last_seen) < stale_threshold
    }

    /// Updates the last seen time to now.
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }
}

/// Collection of endpoints for a service.
#[derive(Debug, Clone, Default)]
pub struct EndpointList {
    /// List of endpoints
    pub endpoints: Vec<ServiceEndpoint>,
}

impl EndpointList {
    /// Creates a new empty endpoint list.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            endpoints: Vec::new(),
        }
    }

    /// Creates an endpoint list from a vector of endpoints.
    #[must_use]
    pub const fn from_endpoints(endpoints: Vec<ServiceEndpoint>) -> Self {
        Self { endpoints }
    }

    /// Returns the number of endpoints.
    #[must_use]
    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    /// Checks if the list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    /// Gets healthy endpoints (those seen recently).
    #[must_use]
    pub fn get_healthy(&self, stale_threshold: Duration) -> Vec<&ServiceEndpoint> {
        self.endpoints
            .iter()
            .filter(|ep| ep.is_healthy(stale_threshold))
            .collect()
    }

    /// Finds endpoints by transport type.
    #[must_use]
    pub fn by_transport(&self, transport: TransportType) -> Vec<&ServiceEndpoint> {
        self.endpoints
            .iter()
            .filter(|ep| ep.transport == transport)
            .collect()
    }

    /// Adds an endpoint to the list.
    pub fn push(&mut self, endpoint: ServiceEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Removes all endpoints from the list.
    pub fn clear(&mut self) {
        self.endpoints.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triton_service_name() {
        assert_eq!(TritonService::Vmapi.name(), "vmapi");
        assert_eq!(TritonService::Cnapi.name(), "cnapi");
        assert_eq!(TritonService::Napi.name(), "napi");
        assert_eq!(TritonService::Imgapi.name(), "imgapi");
        assert_eq!(TritonService::Papi.name(), "papi");
        assert_eq!(TritonService::Fwapi.name(), "fwapi");
        assert_eq!(TritonService::Sapi.name(), "sapi");
        assert_eq!(TritonService::Ufds.name(), "ufds");
        assert_eq!(TritonService::Amon.name(), "amon");
        assert_eq!(TritonService::Workflow.name(), "workflow");
    }

    #[test]
    fn test_triton_service_all() {
        let all = TritonService::all();
        assert_eq!(all.len(), 10);
        assert!(all.contains(&TritonService::Vmapi));
        assert!(all.contains(&TritonService::Workflow));
    }

    #[test]
    fn test_triton_service_default_port() {
        assert_eq!(TritonService::Vmapi.default_port(), DEFAULT_HTTP_PORT);
        assert_eq!(TritonService::Ufds.default_port(), DEFAULT_LDAPS_PORT);
    }

    #[test]
    fn test_triton_service_transport_type() {
        assert_eq!(TritonService::Vmapi.transport_type(), TransportType::Tcp);
    }

    #[test]
    fn test_triton_service_from_str() {
        assert_eq!(
            "vmapi".parse::<TritonService>().unwrap(),
            TritonService::Vmapi
        );
        assert_eq!(
            "VMAPI".parse::<TritonService>().unwrap(),
            TritonService::Vmapi
        );
        assert_eq!(
            "cnapi".parse::<TritonService>().unwrap(),
            TritonService::Cnapi
        );
        assert_eq!(
            "workflow".parse::<TritonService>().unwrap(),
            TritonService::Workflow
        );

        assert!("invalid".parse::<TritonService>().is_err());
    }

    #[test]
    fn test_triton_service_display() {
        assert_eq!(TritonService::Vmapi.to_string(), "vmapi");
        assert_eq!(TritonService::Workflow.to_string(), "workflow");
    }

    #[test]
    fn test_triton_service_serialize() {
        let service = TritonService::Vmapi;
        let json = serde_json::to_string(&service).unwrap();
        assert_eq!(json, "\"Vmapi\"");
    }

    #[test]
    fn test_triton_service_deserialize() {
        let json = "\"Vmapi\"";
        let service: TritonService = serde_json::from_str(json).unwrap();
        assert_eq!(service, TritonService::Vmapi);
    }

    #[test]
    fn test_transport_type_eq() {
        assert_eq!(TransportType::Http, TransportType::Http);
        assert_ne!(TransportType::Http, TransportType::Https);
    }

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert!(config.path.is_none());
    }

    #[test]
    fn test_endpoint_details_default() {
        let details = EndpointDetails::default();
        assert!(details.weight.is_none());
        assert!(details.priority.is_none());
        assert!(details.region.is_none());
        assert!(details.zone.is_none());
        assert!(details.tags.is_empty());
    }

    #[test]
    fn test_service_endpoint_new() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let endpoint = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr);

        assert_eq!(endpoint.id, "ep-1");
        assert_eq!(endpoint.service_name, "vmapi");
        assert_eq!(endpoint.address, addr);
        assert_eq!(endpoint.transport, TransportType::Tcp);
        assert!(endpoint.metadata.is_empty());
        assert!(endpoint.health_check.is_none());
    }

    #[test]
    fn test_service_endpoint_is_healthy() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let endpoint = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr);

        assert!(endpoint.is_healthy(Duration::from_secs(60)));

        // Sleep to ensure endpoint becomes stale
        std::thread::sleep(Duration::from_millis(5));
        assert!(!endpoint.is_healthy(Duration::from_millis(1)));
    }

    #[test]
    fn test_service_endpoint_touch() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut endpoint = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr);

        let original = endpoint.last_seen;
        std::thread::sleep(Duration::from_millis(10));
        endpoint.touch();

        assert!(endpoint.last_seen > original);
    }

    #[test]
    fn test_endpoint_list_new() {
        let list = EndpointList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_endpoint_list_from_endpoints() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let endpoint = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr);

        let list = EndpointList::from_endpoints(vec![endpoint]);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_endpoint_list_push() {
        let mut list = EndpointList::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let endpoint = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr);

        list.push(endpoint);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_endpoint_list_clear() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let endpoint = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr);
        let mut list = EndpointList::from_endpoints(vec![endpoint]);

        assert_eq!(list.len(), 1);
        list.clear();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_endpoint_list_get_healthy() {
        let addr1: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8081".parse().unwrap();

        let ep1 = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr1);
        let ep2 = ServiceEndpoint::new("ep-2".to_string(), "vmapi".to_string(), addr2);

        let list = EndpointList::from_endpoints(vec![ep1, ep2]);

        let healthy = list.get_healthy(Duration::from_secs(60));
        assert_eq!(healthy.len(), 2);

        // Sleep to ensure endpoints become stale
        std::thread::sleep(Duration::from_millis(5));
        let healthy = list.get_healthy(Duration::from_millis(1));
        assert_eq!(healthy.len(), 0);
    }

    #[test]
    fn test_endpoint_list_by_transport() {
        let addr1: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8081".parse().unwrap();

        let mut ep1 = ServiceEndpoint::new("ep-1".to_string(), "vmapi".to_string(), addr1);
        ep1.transport = TransportType::Http;

        let mut ep2 = ServiceEndpoint::new("ep-2".to_string(), "vmapi".to_string(), addr2);
        ep2.transport = TransportType::Https;

        let list = EndpointList::from_endpoints(vec![ep1, ep2]);

        let http_endpoints = list.by_transport(TransportType::Http);
        assert_eq!(http_endpoints.len(), 1);

        let https_endpoints = list.by_transport(TransportType::Https);
        assert_eq!(https_endpoints.len(), 1);

        let grpc_endpoints = list.by_transport(TransportType::Grpc);
        assert_eq!(grpc_endpoints.len(), 0);
    }

    #[test]
    fn test_endpoint_list_default() {
        let list = EndpointList::default();
        assert!(list.is_empty());
    }
}
