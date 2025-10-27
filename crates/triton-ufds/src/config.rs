//! Configuration types for UFDS client usage.

use crate::{dn::DistinguishedName, Result};
use std::path::PathBuf;
use std::time::Duration;
use triton_core::services::UfdsCredentials;
use url::Url;

/// Default connection timeout (seconds).
pub const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 10;
/// Default operation timeout (seconds).
pub const DEFAULT_OPERATION_TIMEOUT_SECS: u64 = 10;

/// Configuration for connecting to UFDS.
#[derive(Debug, Clone)]
pub struct UfdsConfig {
    url: String,
    credentials: UfdsCredentials,
    base_dn: DistinguishedName,
    user_base_dn: DistinguishedName,
    group_base_dn: Option<DistinguishedName>,
    user_filter_template: String,
    tls_verify: bool,
    tls_ca_cert: Option<PathBuf>,
    connection_timeout_secs: u64,
    operation_timeout_secs: u64,
}

impl UfdsConfig {
    /// Creates a new UFDS configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided URL is invalid.
    pub fn new(
        url: impl Into<String>,
        credentials: UfdsCredentials,
        base_dn: DistinguishedName,
    ) -> Result<Self> {
        let url_string = url.into();
        Url::parse(&url_string)?;

        Ok(Self {
            url: url_string,
            credentials,
            user_base_dn: base_dn.clone(),
            base_dn,
            group_base_dn: None,
            user_filter_template: "(&(objectClass=person)(|(uid={login})(login={login})))"
                .to_string(),
            tls_verify: true,
            tls_ca_cert: None,
            connection_timeout_secs: DEFAULT_CONNECTION_TIMEOUT_SECS,
            operation_timeout_secs: DEFAULT_OPERATION_TIMEOUT_SECS,
        })
    }

    /// Returns the UFDS endpoint URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the admin credentials.
    #[must_use]
    pub const fn credentials(&self) -> &UfdsCredentials {
        &self.credentials
    }

    /// Returns the base distinguished name.
    #[must_use]
    pub const fn base_dn(&self) -> &DistinguishedName {
        &self.base_dn
    }

    /// Returns the user search base distinguished name.
    #[must_use]
    pub const fn user_base_dn(&self) -> &DistinguishedName {
        &self.user_base_dn
    }

    /// Returns the group search base distinguished name.
    #[must_use]
    pub fn group_base_dn(&self) -> &DistinguishedName {
        self.group_base_dn.as_ref().unwrap_or(&self.base_dn)
    }

    /// Returns the connection timeout duration.
    #[must_use]
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }

    /// Returns the operation timeout duration.
    #[must_use]
    pub fn operation_timeout(&self) -> Duration {
        Duration::from_secs(self.operation_timeout_secs)
    }

    /// Returns the filter template used for user lookups.
    #[must_use]
    pub fn user_filter_template(&self) -> &str {
        &self.user_filter_template
    }

    /// Returns whether TLS certificate verification is enabled.
    #[must_use]
    pub const fn tls_verify(&self) -> bool {
        self.tls_verify
    }

    /// Optional custom CA certificate path.
    #[must_use]
    pub fn tls_ca_cert(&self) -> Option<&PathBuf> {
        self.tls_ca_cert.as_ref()
    }

    /// Overrides the user search base distinguished name.
    #[must_use]
    pub fn with_user_base_dn(mut self, dn: DistinguishedName) -> Self {
        self.user_base_dn = dn;
        self
    }

    /// Overrides the group search base distinguished name.
    #[must_use]
    pub fn with_group_base_dn(mut self, dn: DistinguishedName) -> Self {
        self.group_base_dn = Some(dn);
        self
    }

    /// Overrides the user lookup filter template.
    ///
    /// The string should contain `{login}` where the user login will be substituted.
    #[must_use]
    pub fn with_user_filter_template(mut self, template: impl Into<String>) -> Self {
        self.user_filter_template = template.into();
        self
    }

    /// Enables or disables TLS certificate verification.
    #[must_use]
    pub const fn with_tls_verification(mut self, verify: bool) -> Self {
        self.tls_verify = verify;
        self
    }

    /// Sets the custom CA certificate path for TLS verification.
    #[must_use]
    pub fn with_tls_ca_cert(mut self, path: PathBuf) -> Self {
        self.tls_ca_cert = Some(path);
        self
    }

    /// Overrides the connection timeout in seconds.
    #[must_use]
    pub const fn with_connection_timeout_secs(mut self, seconds: u64) -> Self {
        self.connection_timeout_secs = seconds;
        self
    }

    /// Overrides the operation timeout in seconds.
    #[must_use]
    pub const fn with_operation_timeout_secs(mut self, seconds: u64) -> Self {
        self.operation_timeout_secs = seconds;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dn::DistinguishedName;
    use triton_core::services::UfdsCredentials;
    use triton_core::uuid::AppUuid;

    #[test]
    fn builder_overrides() {
        let creds = UfdsCredentials::new(
            "cn=admin,dc=example,dc=com".to_string(),
            "secret".to_string(),
            AppUuid::new_v4(),
        );
        let base_dn = DistinguishedName::parse("dc=example,dc=com").unwrap();
        let user_dn = DistinguishedName::parse("ou=People,dc=example,dc=com").unwrap();

        let config = UfdsConfig::new("ldaps://ufds.example.com", creds, base_dn.clone())
            .unwrap()
            .with_user_base_dn(user_dn.clone())
            .with_group_base_dn(DistinguishedName::parse("ou=Groups,dc=example,dc=com").unwrap())
            .with_user_filter_template("(uid={login})")
            .with_connection_timeout_secs(20)
            .with_operation_timeout_secs(30)
            .with_tls_verification(false);

        assert_eq!(config.user_base_dn(), &user_dn);
        assert_eq!(
            config.group_base_dn().as_str(),
            "ou=Groups,dc=example,dc=com"
        );
        assert_eq!(config.user_filter_template(), "(uid={login})");
        assert_eq!(config.connection_timeout(), Duration::from_secs(20));
        assert_eq!(config.operation_timeout(), Duration::from_secs(30));
        assert!(!config.tls_verify());
        assert_eq!(config.base_dn(), &base_dn);
    }
}
