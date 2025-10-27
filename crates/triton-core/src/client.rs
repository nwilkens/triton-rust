//! HTTP client utilities and retry logic.
//!
//! This module provides HTTP client configuration and retry policies
//! for reliable communication with Triton DataCenter services.

use crate::error::Error;
use crate::types::TritonService;
use reqwest::{Client, ClientBuilder, Method, RequestBuilder, Response, StatusCode};
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;
use url::Url;

// Service-specific timeout configurations (in seconds)

/// Default timeout for VMAPI requests
pub const VMAPI_DEFAULT_TIMEOUT: u64 = 30;

/// Default timeout for CNAPI requests
pub const CNAPI_DEFAULT_TIMEOUT: u64 = 30;

/// Default timeout for NAPI requests
pub const NAPI_DEFAULT_TIMEOUT: u64 = 20;

/// Default timeout for IMGAPI requests (larger for image operations)
pub const IMGAPI_DEFAULT_TIMEOUT: u64 = 60;

/// Default timeout for PAPI requests
pub const PAPI_DEFAULT_TIMEOUT: u64 = 20;

/// Default timeout for FWAPI requests
pub const FWAPI_DEFAULT_TIMEOUT: u64 = 20;

/// Default timeout for SAPI requests
pub const SAPI_DEFAULT_TIMEOUT: u64 = 20;

/// Default timeout for UFDS requests
pub const UFDS_DEFAULT_TIMEOUT: u64 = 15;

/// Default timeout for Amon requests
pub const AMON_DEFAULT_TIMEOUT: u64 = 20;

/// Default timeout for Workflow requests
pub const WORKFLOW_DEFAULT_TIMEOUT: u64 = 30;

// Connection pool settings

/// Default idle timeout for connection pools
pub const DEFAULT_POOL_IDLE_TIMEOUT: u64 = 90;

/// Default maximum idle connections per host
pub const DEFAULT_POOL_MAX_IDLE_PER_HOST: usize = 10;

// Retry settings

/// Default maximum number of retry attempts
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default initial retry delay in milliseconds
pub const DEFAULT_RETRY_DELAY_MS: u64 = 500;

/// Default maximum retry delay in milliseconds (for exponential backoff)
pub const DEFAULT_RETRY_MAX_DELAY_MS: u64 = 5000;

/// Retry policy with exponential backoff.
///
/// Configures how HTTP requests should be retried on failure, using exponential
/// backoff to avoid overwhelming failing services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial delay before first retry
    pub initial_delay: Duration,

    /// Maximum delay between retries (cap for exponential backoff)
    pub max_delay: Duration,

    /// Backoff multiplier (typically 2.0 for exponential backoff)
    pub backoff_multiplier: u32,
}

impl RetryPolicy {
    /// Create a new retry policy with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_delay: Duration::from_millis(DEFAULT_RETRY_DELAY_MS),
            max_delay: Duration::from_millis(DEFAULT_RETRY_MAX_DELAY_MS),
            backoff_multiplier: 2,
        }
    }

    /// Create a retry policy with no retries.
    #[must_use]
    pub const fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            backoff_multiplier: 1,
        }
    }

    /// Set the maximum number of retries.
    #[must_use]
    pub const fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set the initial delay.
    #[must_use]
    pub const fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay.
    #[must_use]
    pub const fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff multiplier.
    #[must_use]
    pub const fn with_backoff_multiplier(mut self, multiplier: u32) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Calculate delay for a given attempt number.
    ///
    /// Uses exponential backoff: delay = min(initial_delay * multiplier^attempt, max_delay)
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let multiplier = self.backoff_multiplier.saturating_pow(attempt - 1);
        let delay_ms = self.initial_delay.as_millis() as u64 * u64::from(multiplier);
        let delay = Duration::from_millis(delay_ms);

        std::cmp::min(delay, self.max_delay)
    }

    /// Check if retries are enabled.
    #[must_use]
    pub const fn has_retries(&self) -> bool {
        self.max_retries > 0
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP client configuration.
///
/// Configures HTTP client behavior including timeouts, retries, and connection pooling.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout
    pub timeout: Duration,

    /// Retry policy
    pub retry_policy: RetryPolicy,

    /// Connection pool idle timeout
    pub pool_idle_timeout: Duration,

    /// Maximum idle connections per host
    pub pool_max_idle_per_host: usize,

    /// Enable request/response logging
    pub enable_logging: bool,

    /// Enable response compression
    pub enable_compression: bool,
}

impl ClientConfig {
    /// Create a new client configuration with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_policy: RetryPolicy::new(),
            pool_idle_timeout: Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT),
            pool_max_idle_per_host: DEFAULT_POOL_MAX_IDLE_PER_HOST,
            enable_logging: true,
            enable_compression: true,
        }
    }

    /// Set request timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set retry policy.
    #[must_use]
    pub const fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Disable retries.
    #[must_use]
    pub const fn without_retries(mut self) -> Self {
        self.retry_policy = RetryPolicy::no_retry();
        self
    }

    /// Set connection pool idle timeout.
    #[must_use]
    pub const fn with_pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }

    /// Set maximum idle connections per host.
    #[must_use]
    pub const fn with_pool_max_idle(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }

    /// Enable or disable logging.
    #[must_use]
    pub const fn with_logging(mut self, enabled: bool) -> Self {
        self.enable_logging = enabled;
        self
    }

    /// Enable or disable compression.
    #[must_use]
    pub const fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`ServiceClient`].
#[derive(Debug, Clone)]
pub struct ServiceClientBuilder {
    service: TritonService,
    base_url: Url,
    http_config: ClientConfig,
    retry_policy: RetryPolicy,
    basic_auth: Option<(String, String)>,
    token: Option<String>,
    user_agent: String,
}

impl ServiceClientBuilder {
    /// Create a builder for the specified service and base URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid.
    pub fn new(
        service: TritonService,
        base_url: impl AsRef<str>,
        default_timeout: Duration,
    ) -> crate::Result<Self> {
        let url = Url::parse(base_url.as_ref()).map_err(|err| {
            Error::ConfigError(format!("Invalid base URL `{}`: {err}", base_url.as_ref()))
        })?;

        let config = ClientConfig::new().with_timeout(default_timeout);
        let user_agent = format!(
            "triton-{}-client/{}",
            service.name(),
            env!("CARGO_PKG_VERSION")
        );

        Ok(Self {
            service,
            base_url: url,
            retry_policy: config.retry_policy,
            http_config: config,
            basic_auth: None,
            token: None,
            user_agent,
        })
    }

    /// Override the retry policy.
    #[must_use]
    pub fn with_retry_policy(mut self, retry: RetryPolicy) -> Self {
        self.retry_policy = retry;
        self.http_config.retry_policy = retry;
        self
    }

    /// Override the HTTP client configuration.
    #[must_use]
    pub fn with_http_config(mut self, config: ClientConfig) -> Self {
        self.retry_policy = config.retry_policy;
        self.http_config = config;
        self
    }

    /// Configure HTTP basic authentication credentials.
    #[must_use]
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.basic_auth = Some((username.into(), password.into()));
        self
    }

    /// Configure an X-Auth-Token header.
    #[must_use]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Override the default user agent.
    #[must_use]
    pub fn with_user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = agent.into();
        self
    }

    /// Build the service client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn build(self) -> crate::Result<ServiceClient> {
        let mut builder = ClientBuilder::new()
            .timeout(self.http_config.timeout)
            .user_agent(&self.user_agent)
            .pool_idle_timeout(self.http_config.pool_idle_timeout)
            .pool_max_idle_per_host(self.http_config.pool_max_idle_per_host)
            .connect_timeout(Duration::from_secs(10));

        if !self.http_config.enable_compression {
            builder = builder.no_gzip();
        }

        let http = builder
            .build()
            .map_err(|err| Error::ConfigError(format!("Failed to build HTTP client: {err}")))?;

        Ok(ServiceClient {
            http,
            base_url: self.base_url,
            retry_policy: self.retry_policy,
            basic_auth: self.basic_auth,
            token: self.token,
            service: self.service,
        })
    }
}

/// Shared HTTP client wrapper used by Triton service clients.
#[derive(Clone)]
pub struct ServiceClient {
    http: Client,
    base_url: Url,
    retry_policy: RetryPolicy,
    basic_auth: Option<(String, String)>,
    token: Option<String>,
    service: TritonService,
}

impl ServiceClient {
    /// Returns the service associated with this client.
    #[must_use]
    pub const fn service(&self) -> TritonService {
        self.service
    }

    /// Returns the base URL for the service.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Access the underlying reqwest client.
    #[must_use]
    pub fn http_client(&self) -> &Client {
        &self.http
    }

    /// Construct a request builder for the given method/path with optional query parameters.
    pub fn request(
        &self,
        method: Method,
        path: &str,
        params: &[(&'static str, String)],
    ) -> crate::Result<RequestBuilder> {
        let url = self.build_url(path)?;
        let mut request = self.http.request(method, url).query(params);

        if let Some((user, pass)) = &self.basic_auth {
            request = request.basic_auth(user, Some(pass));
        }
        if let Some(token) = &self.token {
            request = request.header("X-Auth-Token", token);
        }

        Ok(request)
    }

    /// Execute a request with retry semantics.
    pub async fn execute_with_retry<F, G>(
        &self,
        method: Method,
        path: &str,
        params: &[(&'static str, String)],
        mut configure: F,
        mut map_error: G,
    ) -> crate::Result<Response>
    where
        F: FnMut(RequestBuilder) -> RequestBuilder,
        G: FnMut(StatusCode, String) -> Error,
    {
        let mut attempt = 0;
        #[allow(unused_assignments)]
        let mut last_error: Option<Error> = None;

        loop {
            let builder = self.request(method.clone(), path, params)?;
            let request = configure(builder);

            debug!(
                service = self.service.name(),
                path, attempt, "Service request"
            );

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return Ok(response);
                    }

                    let text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = map_error(status, text);
                    if should_retry(status) {
                        last_error = Some(error);
                    } else {
                        return Err(error);
                    }
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
                debug!(
                    service = self.service.name(),
                    ?delay,
                    "Retrying service request"
                );
                sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::ServiceUnavailable(format!(
                "{} request failed after retries",
                self.service.name()
            ))
        }))
    }

    fn build_url(&self, path: &str) -> crate::Result<Url> {
        let normalized = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        self.base_url
            .join(normalized)
            .map_err(|err| Error::InvalidEndpoint(format!("Invalid path `{path}`: {err}")))
    }
}

fn should_retry(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    ) || status.is_server_error()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_constants() {
        assert_eq!(VMAPI_DEFAULT_TIMEOUT, 30);
        assert_eq!(CNAPI_DEFAULT_TIMEOUT, 30);
        assert_eq!(NAPI_DEFAULT_TIMEOUT, 20);
        assert_eq!(IMGAPI_DEFAULT_TIMEOUT, 60);
        assert_eq!(PAPI_DEFAULT_TIMEOUT, 20);
        assert_eq!(FWAPI_DEFAULT_TIMEOUT, 20);
        assert_eq!(SAPI_DEFAULT_TIMEOUT, 20);
        assert_eq!(UFDS_DEFAULT_TIMEOUT, 15);
        assert_eq!(AMON_DEFAULT_TIMEOUT, 20);
        assert_eq!(WORKFLOW_DEFAULT_TIMEOUT, 30);
    }

    #[test]
    fn test_retry_policy_new() {
        let policy = RetryPolicy::new();
        assert_eq!(policy.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(
            policy.initial_delay,
            Duration::from_millis(DEFAULT_RETRY_DELAY_MS)
        );
        assert_eq!(
            policy.max_delay,
            Duration::from_millis(DEFAULT_RETRY_MAX_DELAY_MS)
        );
        assert_eq!(policy.backoff_multiplier, 2);
    }

    #[test]
    fn test_retry_policy_no_retry() {
        let policy = RetryPolicy::no_retry();
        assert_eq!(policy.max_retries, 0);
        assert!(!policy.has_retries());
    }

    #[test]
    fn test_retry_policy_builder() {
        let policy = RetryPolicy::new()
            .with_max_retries(5)
            .with_initial_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10))
            .with_backoff_multiplier(3);

        assert_eq!(policy.max_retries, 5);
        assert_eq!(policy.initial_delay, Duration::from_millis(100));
        assert_eq!(policy.max_delay, Duration::from_secs(10));
        assert_eq!(policy.backoff_multiplier, 3);
    }

    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy::new();

        // Attempt 0 should return 0
        assert_eq!(policy.delay_for_attempt(0), Duration::from_secs(0));

        // Attempt 1: initial_delay * 2^0 = 500ms
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(500));

        // Attempt 2: initial_delay * 2^1 = 1000ms
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(1000));

        // Attempt 3: initial_delay * 2^2 = 2000ms
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(2000));

        // Attempt 4: initial_delay * 2^3 = 4000ms
        assert_eq!(policy.delay_for_attempt(4), Duration::from_millis(4000));

        // Attempt 5: would be 8000ms but capped at max_delay (5000ms)
        assert_eq!(policy.delay_for_attempt(5), Duration::from_millis(5000));
    }

    #[test]
    fn test_retry_policy_has_retries() {
        assert!(RetryPolicy::new().has_retries());
        assert!(!RetryPolicy::no_retry().has_retries());
    }

    #[test]
    fn test_client_config_new() {
        let config = ClientConfig::new();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.retry_policy.max_retries, DEFAULT_MAX_RETRIES);
        assert!(config.enable_logging);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_client_config_builder() {
        let config = ClientConfig::new()
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(RetryPolicy::no_retry())
            .with_pool_idle_timeout(Duration::from_secs(120))
            .with_pool_max_idle(20)
            .with_logging(false)
            .with_compression(false);

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.retry_policy.max_retries, 0);
        assert_eq!(config.pool_idle_timeout, Duration::from_secs(120));
        assert_eq!(config.pool_max_idle_per_host, 20);
        assert!(!config.enable_logging);
        assert!(!config.enable_compression);
    }

    #[test]
    fn test_client_config_without_retries() {
        let config = ClientConfig::new().without_retries();
        assert_eq!(config.retry_policy.max_retries, 0);
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy::new()
            .with_initial_delay(Duration::from_millis(100))
            .with_backoff_multiplier(2)
            .with_max_delay(Duration::from_secs(5));

        // Test exponential growth
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(400));
        assert_eq!(policy.delay_for_attempt(4), Duration::from_millis(800));
        assert_eq!(policy.delay_for_attempt(5), Duration::from_millis(1600));
        assert_eq!(policy.delay_for_attempt(6), Duration::from_millis(3200));

        // Should cap at max_delay (5000ms)
        assert_eq!(policy.delay_for_attempt(7), Duration::from_millis(5000));
        assert_eq!(policy.delay_for_attempt(10), Duration::from_millis(5000));
    }

    #[test]
    fn test_pool_constants() {
        assert_eq!(DEFAULT_POOL_IDLE_TIMEOUT, 90);
        assert_eq!(DEFAULT_POOL_MAX_IDLE_PER_HOST, 10);
    }

    #[test]
    fn test_retry_constants() {
        assert_eq!(DEFAULT_MAX_RETRIES, 3);
        assert_eq!(DEFAULT_RETRY_DELAY_MS, 500);
        assert_eq!(DEFAULT_RETRY_MAX_DELAY_MS, 5000);
    }
}
