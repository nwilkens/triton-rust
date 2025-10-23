//! HTTP client utilities and retry logic.
//!
//! This module provides HTTP client configuration and retry policies
//! for reliable communication with Triton DataCenter services.

use std::time::Duration;

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
        assert_eq!(policy.initial_delay, Duration::from_millis(DEFAULT_RETRY_DELAY_MS));
        assert_eq!(policy.max_delay, Duration::from_millis(DEFAULT_RETRY_MAX_DELAY_MS));
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
