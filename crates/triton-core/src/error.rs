//! Error types for Triton operations.
//!
//! This module provides a comprehensive error type hierarchy for Triton DataCenter operations,
//! including HTTP status code mapping and structured error responses.

use serde::Serialize;
use thiserror::Error;

/// Main error type for Triton operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// Triton service is unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Service discovery failed
    #[error("Service discovery failed: {0}")]
    DiscoveryFailed(String),

    /// Failed to parse SAPI response
    #[error("Failed to parse SAPI response: {0}")]
    SapiParseError(String),

    /// Invalid UUID format
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    /// Invalid network configuration
    #[error("Invalid network configuration: {0}")]
    InvalidNetwork(String),

    /// Invalid VM state
    #[error("Invalid VM state: {0}")]
    InvalidVmState(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    /// Operation timed out
    #[error("Timeout waiting for service: {0}")]
    Timeout(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Bad request with details
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Conflict error
    #[error("Conflict: {0}")]
    Conflict(String),

    /// External service error
    #[error("External service error: {service}: {message}")]
    ExternalServiceError {
        /// Service name that failed
        service: String,
        /// Error message
        message: String,
    },

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Endpoint cache error
    #[error("Endpoint cache error: {0}")]
    CacheError(String),

    /// Invalid endpoint
    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),

    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Specialized result type for Triton operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Structured error response for serialization.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetail,
    /// Optional request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Error detail structure.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ErrorDetail {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Error {
    /// Returns the error code for this error type.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::DiscoveryFailed(_) => "DISCOVERY_FAILED",
            Self::SapiParseError(_) => "SAPI_PARSE_ERROR",
            Self::InvalidUuid(_) => "INVALID_UUID",
            Self::InvalidNetwork(_) => "INVALID_NETWORK",
            Self::InvalidVmState(_) => "INVALID_VM_STATE",
            Self::ConfigError(_) => "CONFIG_ERROR",
            Self::HttpError(_) => "HTTP_ERROR",
            Self::Timeout(_) => "TIMEOUT",
            Self::NotFound(_) => "NOT_FOUND",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::Conflict(_) => "CONFLICT",
            Self::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::CacheError(_) => "CACHE_ERROR",
            Self::InvalidEndpoint(_) => "INVALID_ENDPOINT",
            Self::NotImplemented(_) => "NOT_IMPLEMENTED",
        }
    }

    /// Converts the error into an `ErrorResponse`.
    #[must_use]
    pub fn into_error_response(self) -> ErrorResponse {
        self.into_error_response_with_id(None)
    }

    /// Converts the error into an `ErrorResponse` with a request ID.
    #[must_use]
    pub fn into_error_response_with_id(self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                details: None,
            },
            request_id,
        }
    }

    /// Returns true if this error should be logged as a serious error.
    #[must_use]
    pub const fn should_log(&self) -> bool {
        matches!(
            self,
            Self::InternalError(_) | Self::ConfigError(_) | Self::ExternalServiceError { .. }
        )
    }
}

// Conversions from external error types
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout(err.to_string())
        } else if err.is_connect() {
            Self::ServiceUnavailable(err.to_string())
        } else {
            Self::HttpError(err.to_string())
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidEndpoint(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SapiParseError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError(err.to_string())
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Self::InvalidUuid(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            Error::ServiceUnavailable("test".to_string()).error_code(),
            "SERVICE_UNAVAILABLE"
        );
        assert_eq!(
            Error::DiscoveryFailed("test".to_string()).error_code(),
            "DISCOVERY_FAILED"
        );
        assert_eq!(
            Error::SapiParseError("test".to_string()).error_code(),
            "SAPI_PARSE_ERROR"
        );
        assert_eq!(
            Error::InvalidUuid("test".to_string()).error_code(),
            "INVALID_UUID"
        );
        assert_eq!(
            Error::InvalidNetwork("test".to_string()).error_code(),
            "INVALID_NETWORK"
        );
        assert_eq!(
            Error::InvalidVmState("test".to_string()).error_code(),
            "INVALID_VM_STATE"
        );
        assert_eq!(
            Error::ConfigError("test".to_string()).error_code(),
            "CONFIG_ERROR"
        );
        assert_eq!(
            Error::HttpError("test".to_string()).error_code(),
            "HTTP_ERROR"
        );
        assert_eq!(Error::Timeout("test".to_string()).error_code(), "TIMEOUT");
        assert_eq!(
            Error::NotFound("test".to_string()).error_code(),
            "NOT_FOUND"
        );
        assert_eq!(
            Error::InvalidRequest("test".to_string()).error_code(),
            "INVALID_REQUEST"
        );
        assert_eq!(
            Error::BadRequest("test".to_string()).error_code(),
            "BAD_REQUEST"
        );
        assert_eq!(
            Error::ValidationError("test".to_string()).error_code(),
            "VALIDATION_ERROR"
        );
        assert_eq!(Error::Conflict("test".to_string()).error_code(), "CONFLICT");
        assert_eq!(
            Error::ExternalServiceError {
                service: "test".to_string(),
                message: "msg".to_string()
            }
            .error_code(),
            "EXTERNAL_SERVICE_ERROR"
        );
        assert_eq!(
            Error::InternalError("test".to_string()).error_code(),
            "INTERNAL_ERROR"
        );
        assert_eq!(
            Error::CacheError("test".to_string()).error_code(),
            "CACHE_ERROR"
        );
        assert_eq!(
            Error::InvalidEndpoint("test".to_string()).error_code(),
            "INVALID_ENDPOINT"
        );
        assert_eq!(
            Error::NotImplemented("test".to_string()).error_code(),
            "NOT_IMPLEMENTED"
        );
    }

    #[test]
    fn test_error_display() {
        let err = Error::ServiceUnavailable("vmapi".to_string());
        assert_eq!(err.to_string(), "Service unavailable: vmapi");

        let err = Error::ExternalServiceError {
            service: "cnapi".to_string(),
            message: "connection failed".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "External service error: cnapi: connection failed"
        );
    }

    #[test]
    fn test_into_error_response() {
        let err = Error::NotFound("vm-123".to_string());
        let response = err.clone().into_error_response();

        assert_eq!(response.error.code, "NOT_FOUND");
        assert_eq!(response.error.message, "Not found: vm-123");
        assert!(response.request_id.is_none());

        let response_with_id = err.into_error_response_with_id(Some("req-456".to_string()));
        assert_eq!(response_with_id.request_id, Some("req-456".to_string()));
    }

    #[test]
    fn test_should_log() {
        assert!(Error::InternalError("test".to_string()).should_log());
        assert!(Error::ConfigError("test".to_string()).should_log());
        assert!(Error::ExternalServiceError {
            service: "test".to_string(),
            message: "msg".to_string()
        }
        .should_log());

        assert!(!Error::NotFound("test".to_string()).should_log());
        assert!(!Error::InvalidRequest("test".to_string()).should_log());
    }

    // Note: Testing reqwest::Error conversion is difficult without making actual HTTP requests
    // The conversion logic is covered by integration tests

    #[test]
    fn test_from_url_parse_error() {
        let err = url::Url::parse("not a url").unwrap_err();
        let triton_err: Error = err.into();
        assert!(matches!(triton_err, Error::InvalidEndpoint(_)));
    }

    #[test]
    fn test_from_uuid_error() {
        let err = uuid::Uuid::parse_str("not-a-uuid").unwrap_err();
        let triton_err: Error = err.into();
        assert!(matches!(triton_err, Error::InvalidUuid(_)));
        assert_eq!(triton_err.error_code(), "INVALID_UUID");
    }

    #[test]
    fn test_from_serde_json_error() {
        let err = serde_json::from_str::<serde_json::Value>("{invalid json}").unwrap_err();
        let triton_err: Error = err.into();
        assert!(matches!(triton_err, Error::SapiParseError(_)));
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: ErrorDetail {
                code: "TEST_ERROR".to_string(),
                message: "Test message".to_string(),
                details: None,
            },
            request_id: Some("req-123".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("TEST_ERROR"));
        assert!(json.contains("Test message"));
        assert!(json.contains("req-123"));
    }

    #[test]
    fn test_error_response_serialization_no_request_id() {
        let response = ErrorResponse {
            error: ErrorDetail {
                code: "TEST_ERROR".to_string(),
                message: "Test message".to_string(),
                details: None,
            },
            request_id: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("request_id"));
    }

    #[test]
    fn test_error_clone() {
        let err = Error::NotFound("test".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_error_partial_eq() {
        let err1 = Error::NotFound("test".to_string());
        let err2 = Error::NotFound("test".to_string());
        let err3 = Error::NotFound("other".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}
