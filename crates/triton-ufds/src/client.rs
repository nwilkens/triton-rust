//! UFDS LDAP client implementation.

use crate::{
    config::UfdsConfig,
    dn::DistinguishedName,
    group::Group,
    user::{AccountStatus, User, UserFlags},
    Result,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ldap3::{
    result::Result as LdapResult, LdapConnAsync, LdapConnSettings, Mod, Scope, SearchEntry,
};
use native_tls::{Certificate, TlsConnector};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::warn;
use triton_core::error::Error;
use triton_core::uuid::OwnerUuid;

const USER_ATTRIBUTES: &[&str] = &[
    "uuid",
    "login",
    "uid",
    "email",
    "cn",
    "sn",
    "givenName",
    "company",
    "phone",
    "approved_for_provisioning",
    "registered_developer",
    "triton_cns_enabled",
    "pwdAccountLockedTime",
    "password_expired",
    "memberof",
    "created",
    "updated",
];

const GROUP_ATTRIBUTES: &[&str] = &["cn", "description", "member"];

/// Represents the search scope for LDAP queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    /// Base object only.
    Base,
    /// One level below the base.
    OneLevel,
    /// Entire subtree.
    Subtree,
}

impl From<SearchScope> for Scope {
    fn from(scope: SearchScope) -> Self {
        match scope {
            SearchScope::Base => Scope::Base,
            SearchScope::OneLevel => Scope::OneLevel,
            SearchScope::Subtree => Scope::Subtree,
        }
    }
}

/// LDAP entry representation used by the client.
#[derive(Debug, Clone)]
pub struct LdapEntry {
    /// Distinguished name of the entry.
    pub dn: String,
    /// Attribute map (values preserved order from server).
    pub attributes: HashMap<String, Vec<String>>,
}

impl LdapEntry {
    /// Returns the first value of the attribute if present.
    #[must_use]
    pub fn first(&self, attribute: &str) -> Option<&str> {
        self.attributes
            .get(attribute)
            .and_then(|values| values.first().map(|value| value.as_str()))
    }

    /// Returns all values for the attribute.
    #[must_use]
    pub fn values(&self, attribute: &str) -> Option<&[String]> {
        self.attributes
            .get(attribute)
            .map(|values| values.as_slice())
    }

    /// Parses the attribute as boolean (`true` / `1`).
    #[must_use]
    pub fn bool_value(&self, attribute: &str) -> bool {
        self.first(attribute)
            .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
            .unwrap_or(false)
    }
}

/// LDAP modification request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryModification {
    /// Add attribute values.
    Add {
        /// Attribute to modify.
        attribute: String,
        /// Values to add.
        values: Vec<String>,
    },
    /// Delete attribute values.
    Delete {
        /// Attribute to modify.
        attribute: String,
        /// Values to delete (empty removes attribute).
        values: Vec<String>,
    },
    /// Replace attribute values.
    Replace {
        /// Attribute to modify.
        attribute: String,
        /// Replacement values.
        values: Vec<String>,
    },
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(crate) trait LdapSession: Send {
    async fn simple_bind(&mut self, dn: &str, password: &str) -> Result<()>;
    async fn search(
        &mut self,
        base_dn: &str,
        scope: SearchScope,
        filter: &str,
        attributes: &[&'static str],
    ) -> Result<Vec<LdapEntry>>;
    async fn modify(&mut self, dn: &str, modifications: &[DirectoryModification]) -> Result<()>;
    async fn unbind(&mut self) -> Result<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(crate) trait LdapConnector: Send + Sync {
    async fn connect(&self) -> Result<Box<dyn LdapSession>>;
}

/// UFDS client with pluggable LDAP backend.
pub struct UfdsClient {
    config: Arc<UfdsConfig>,
    connector: Box<dyn LdapConnector>,
}

impl UfdsClient {
    /// Creates a UFDS client that uses the real LDAP connector.
    #[must_use]
    pub fn new(config: UfdsConfig) -> Self {
        let config = Arc::new(config);
        let connector: Box<dyn LdapConnector> = Box::new(RealLdapConnector::new(config.clone()));
        Self { config, connector }
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_connector(config: UfdsConfig, connector: Box<dyn LdapConnector>) -> Self {
        Self {
            config: Arc::new(config),
            connector,
        }
    }

    /// Authenticates a user and returns their UFDS entry on success.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the user does not exist or [`Error::InvalidRequest`]
    /// when credentials are invalid.
    pub async fn authenticate(&self, login: &str, password: &str) -> Result<User> {
        let mut admin_session = self.admin_session().await?;
        let user_entry = self.lookup_user(&mut *admin_session, login).await?;
        admin_session.unbind().await?;

        // Verify user credentials by binding as the user.
        let mut user_session = self.connector.connect().await?;
        self.execute_with_timeout(async {
            user_session
                .simple_bind(user_entry.dn.as_str(), password)
                .await
        })
        .await
        .map_err(|_| Error::InvalidRequest("invalid credentials".to_string()))?;
        user_session.unbind().await?;

        parse_user_entry(&user_entry)
    }

    /// Fetches a user entry without performing authentication.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the user does not exist.
    pub async fn fetch_user(&self, login: &str) -> Result<User> {
        let mut admin_session = self.admin_session().await?;
        let entry = self.lookup_user(&mut *admin_session, login).await?;
        admin_session.unbind().await?;
        parse_user_entry(&entry)
    }

    /// Lists groups within the configured group search base.
    pub async fn list_groups(&self) -> Result<Vec<Group>> {
        let mut session = self.admin_session().await?;
        let entries = self
            .execute_with_timeout(session.search(
                self.config.group_base_dn().as_str(),
                SearchScope::Subtree,
                "(|(objectClass=groupOfNames)(objectClass=groupOfUniqueNames))",
                GROUP_ATTRIBUTES,
            ))
            .await?;
        session.unbind().await?;

        entries
            .into_iter()
            .map(|entry| parse_group_entry(&entry))
            .collect()
    }

    /// Adds a user to the specified group.
    pub async fn add_user_to_group(
        &self,
        user_dn: &DistinguishedName,
        group_dn: &DistinguishedName,
    ) -> Result<()> {
        let mut session = self.admin_session().await?;
        session
            .modify(
                group_dn.as_str(),
                &[DirectoryModification::Add {
                    attribute: "member".to_string(),
                    values: vec![user_dn.as_str().to_string()],
                }],
            )
            .await?;
        session.unbind().await?;
        Ok(())
    }

    /// Removes a user from the specified group.
    pub async fn remove_user_from_group(
        &self,
        user_dn: &DistinguishedName,
        group_dn: &DistinguishedName,
    ) -> Result<()> {
        let mut session = self.admin_session().await?;
        session
            .modify(
                group_dn.as_str(),
                &[DirectoryModification::Delete {
                    attribute: "member".to_string(),
                    values: vec![user_dn.as_str().to_string()],
                }],
            )
            .await?;
        session.unbind().await?;
        Ok(())
    }

    async fn admin_session(&self) -> Result<Box<dyn LdapSession>> {
        let mut session = self.connector.connect().await?;
        self.execute_with_timeout(session.simple_bind(
            self.config.credentials().bind_dn(),
            self.config.credentials().bind_password(),
        ))
        .await?;
        Ok(session)
    }

    async fn lookup_user(&self, session: &mut dyn LdapSession, login: &str) -> Result<LdapEntry> {
        let filter = self.user_search_filter(login);
        let entries = self
            .execute_with_timeout(session.search(
                self.config.user_base_dn().as_str(),
                SearchScope::Subtree,
                &filter,
                USER_ATTRIBUTES,
            ))
            .await?;

        entries
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotFound(format!("user `{login}` not found in UFDS")))
    }

    fn user_search_filter(&self, login: &str) -> String {
        let escaped = escape_filter_value(login);
        self.config
            .user_filter_template()
            .replace("{login}", &escaped)
    }

    async fn execute_with_timeout<F, T>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        timeout(self.config.operation_timeout(), fut)
            .await
            .map_err(|_| Error::Timeout("UFDS operation timed out".to_string()))?
    }
}

/// Real LDAP connector backed by `ldap3`.
pub struct RealLdapConnector {
    config: Arc<UfdsConfig>,
}

impl RealLdapConnector {
    /// Creates a new connector instance.
    #[must_use]
    pub fn new(config: Arc<UfdsConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl LdapConnector for RealLdapConnector {
    async fn connect(&self) -> Result<Box<dyn LdapSession>> {
        let settings = build_ldap_settings(&self.config)?;
        let (conn, ldap) = LdapConnAsync::with_settings(settings, self.config.url())
            .await
            .map_err(map_ldap_error)?;
        ldap3::drive!(conn);
        Ok(Box::new(RealLdapSession {
            inner: ldap,
            operation_timeout: self.config.operation_timeout(),
        }))
    }
}

struct RealLdapSession {
    inner: ldap3::Ldap,
    operation_timeout: Duration,
}

#[async_trait]
impl LdapSession for RealLdapSession {
    async fn simple_bind(&mut self, dn: &str, password: &str) -> Result<()> {
        let result = timeout(self.operation_timeout, self.inner.simple_bind(dn, password))
            .await
            .map_err(|_| Error::Timeout("UFDS bind timed out".to_string()))?
            .map_err(map_ldap_error)?;
        ensure_ldap_success(result)?;
        Ok(())
    }

    async fn search(
        &mut self,
        base_dn: &str,
        scope: SearchScope,
        filter: &str,
        attributes: &[&'static str],
    ) -> Result<Vec<LdapEntry>> {
        let result = timeout(
            self.operation_timeout,
            self.inner
                .search(base_dn, scope.into(), filter, attributes.to_vec()),
        )
        .await
        .map_err(|_| Error::Timeout("UFDS search timed out".to_string()))?
        .map_err(map_ldap_error)?;
        let (entries, _) = result.success().map_err(map_ldap_error)?;
        Ok(entries
            .into_iter()
            .map(SearchEntry::construct)
            .map(|entry| LdapEntry {
                dn: entry.dn,
                attributes: entry.attrs,
            })
            .collect())
    }

    async fn modify(&mut self, dn: &str, modifications: &[DirectoryModification]) -> Result<()> {
        let mods = modifications
            .iter()
            .map(|m| match m {
                DirectoryModification::Add { attribute, values } => Mod::Add(
                    attribute.clone(),
                    values.iter().cloned().collect::<HashSet<_>>(),
                ),
                DirectoryModification::Delete { attribute, values } => Mod::Delete(
                    attribute.clone(),
                    values.iter().cloned().collect::<HashSet<_>>(),
                ),
                DirectoryModification::Replace { attribute, values } => Mod::Replace(
                    attribute.clone(),
                    values.iter().cloned().collect::<HashSet<_>>(),
                ),
            })
            .collect::<Vec<_>>();

        let result = timeout(self.operation_timeout, self.inner.modify(dn, mods))
            .await
            .map_err(|_| Error::Timeout("UFDS modify timed out".to_string()))?
            .map_err(map_ldap_error)?;
        ensure_ldap_success(result)?;
        Ok(())
    }

    async fn unbind(&mut self) -> Result<()> {
        timeout(self.operation_timeout, self.inner.unbind())
            .await
            .map_err(|_| Error::Timeout("UFDS unbind timed out".to_string()))?
            .map_err(map_ldap_error)?;
        Ok(())
    }
}

fn build_ldap_settings(config: &UfdsConfig) -> Result<LdapConnSettings> {
    let mut settings = LdapConnSettings::new().set_conn_timeout(config.connection_timeout());

    if !config.tls_verify() {
        let connector = TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|err| {
                Error::ConfigError(format!("failed to construct TLS connector: {err}"))
            })?;
        settings = settings.set_connector(connector).set_no_tls_verify(true);
    } else if let Some(cert_path) = config.tls_ca_cert() {
        let pem = fs::read(cert_path).map_err(|err| {
            Error::ConfigError(format!(
                "failed to read UFDS CA certificate {}: {err}",
                cert_path.display()
            ))
        })?;
        let certificate = Certificate::from_pem(&pem)
            .map_err(|err| Error::ConfigError(format!("invalid UFDS CA certificate: {err}")))?;
        let connector = TlsConnector::builder()
            .add_root_certificate(certificate)
            .build()
            .map_err(|err| {
                Error::ConfigError(format!("failed to load UFDS CA certificate: {err}"))
            })?;
        settings = settings.set_connector(connector);
    }

    Ok(settings)
}

fn handle_ldap_result<T>(result: LdapResult<T>) -> Result<T> {
    result.map_err(|err| Error::ExternalServiceError {
        service: "ufds".to_string(),
        message: err.to_string(),
    })
}

fn map_ldap_error(err: ldap3::LdapError) -> Error {
    Error::ExternalServiceError {
        service: "ufds".to_string(),
        message: err.to_string(),
    }
}

fn ensure_ldap_success(result: ldap3::LdapResult) -> Result<()> {
    handle_ldap_result::<ldap3::LdapResult>(Ok(result)).map(|_| ())
}

fn parse_user_entry(entry: &LdapEntry) -> Result<User> {
    let dn = DistinguishedName::parse(&entry.dn)?;
    let uuid_str = entry
        .first("uuid")
        .ok_or_else(|| missing_attribute("uuid"))?;
    let uuid = OwnerUuid::parse_str(uuid_str)?;

    let login = entry
        .first("login")
        .or_else(|| entry.first("uid"))
        .ok_or_else(|| missing_attribute("login"))?;

    let groups = entry
        .values("memberof")
        .map(|values| {
            values
                .iter()
                .filter_map(|value| DistinguishedName::parse(value).ok())
                .filter_map(|dn| dn.get("cn").map(str::to_owned))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let mut status = AccountStatus::new()
        .with_locked(entry.first("pwdAccountLockedTime").is_some())
        .with_password_expired(entry.bool_value("password_expired"));

    let is_admin = groups.iter().any(|group| matches_admin_group(group));
    status = status.with_admin(is_admin);

    let flags = UserFlags::default()
        .with_provisioning(entry.bool_value("approved_for_provisioning"))
        .with_registered_developer(entry.bool_value("registered_developer"))
        .with_triton_cns(entry.bool_value("triton_cns_enabled"));

    // Build user object using the builder for clarity.
    let mut builder = User::builder(dn.clone(), uuid, login.to_string());

    if let Some(email) = entry.first("email") {
        builder = builder.email(email.to_string());
    }
    if let Some(cn) = entry.first("cn") {
        builder = builder.cn(cn.to_string());
    }
    if let Some(sn) = entry.first("sn") {
        builder = builder.sn(sn.to_string());
    }
    if let Some(given) = entry.first("givenName") {
        builder = builder.given_name(given.to_string());
    }
    if let Some(company) = entry.first("company") {
        builder = builder.company(company.to_string());
    }
    if let Some(phone) = entry.first("phone") {
        builder = builder.phone(phone.to_string());
    }

    if let Some(created) = parse_timestamp(entry.first("created")) {
        builder = builder.created_at(created);
    }
    if let Some(updated) = parse_timestamp(entry.first("updated")) {
        builder = builder.updated_at(updated);
    }

    builder = builder.status(status).flags(flags).groups(groups);

    Ok(builder.build())
}

fn parse_group_entry(entry: &LdapEntry) -> Result<Group> {
    let dn = DistinguishedName::parse(&entry.dn)?;
    let name = entry
        .first("cn")
        .ok_or_else(|| missing_attribute("cn"))?
        .to_string();

    let mut builder = Group::builder(dn.clone(), name);
    if let Some(description) = entry.first("description") {
        builder = builder.description(description.to_string());
    }

    if let Some(members) = entry.values("member") {
        let parsed_members = members
            .iter()
            .filter_map(|dn_str| match DistinguishedName::parse(dn_str) {
                Ok(member_dn) => Some(member_dn),
                Err(err) => {
                    warn!("Failed to parse member DN `{dn_str}`: {err}");
                    None
                }
            })
            .collect::<Vec<_>>();
        builder = builder.members(parsed_members);
    }

    Ok(builder.build())
}

fn parse_timestamp(value: Option<&str>) -> Option<DateTime<Utc>> {
    value
        .and_then(|val| DateTime::parse_from_rfc3339(val).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn matches_admin_group(group: &str) -> bool {
    group.eq_ignore_ascii_case("admins") || group.eq_ignore_ascii_case("operators")
}

fn missing_attribute(attribute: &str) -> Error {
    Error::InvalidRequest(format!("UFDS entry missing attribute `{attribute}`"))
}

fn escape_filter_value(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '*' => "\\2a".chars().collect::<Vec<_>>(),
            '(' => "\\28".chars().collect(),
            ')' => "\\29".chars().collect(),
            '\\' => "\\5c".chars().collect(),
            '\0' => "\\00".chars().collect(),
            _ => vec![ch],
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dn::DistinguishedName;
    use triton_core::services::UfdsCredentials;
    use triton_core::uuid::AppUuid;

    fn sample_config() -> UfdsConfig {
        let credentials = UfdsCredentials::new(
            "cn=admin,dc=example,dc=com".to_string(),
            "secret".to_string(),
            AppUuid::new_v4(),
        );
        let base_dn = DistinguishedName::parse("dc=example,dc=com").unwrap();
        UfdsConfig::new("ldaps://example.com", credentials, base_dn).unwrap()
    }

    fn sample_entry() -> LdapEntry {
        let mut attributes = HashMap::new();
        attributes.insert("uuid".to_string(), vec![OwnerUuid::new_v4().to_string()]);
        attributes.insert("login".to_string(), vec!["jdoe".to_string()]);
        attributes.insert("cn".to_string(), vec!["John Doe".to_string()]);
        attributes.insert(
            "memberof".to_string(),
            vec!["cn=admins,dc=example,dc=com".to_string()],
        );
        LdapEntry {
            dn: "uid=jdoe,dc=example,dc=com".to_string(),
            attributes,
        }
    }

    #[tokio::test]
    async fn authenticate_success() {
        let mut connector = MockLdapConnector::new();
        let mut sequence = mockall::Sequence::new();
        let mut admin_session = MockLdapSession::new();
        admin_session.expect_simple_bind().returning(|_, _| Ok(()));
        admin_session
            .expect_search()
            .returning(|_, _, _, _| Ok(vec![sample_entry()]));
        admin_session.expect_unbind().returning(|| Ok(()));

        let mut user_session = MockLdapSession::new();
        user_session.expect_simple_bind().returning(|_, _| Ok(()));
        user_session.expect_unbind().returning(|| Ok(()));

        connector
            .expect_connect()
            .times(1)
            .in_sequence(&mut sequence)
            .return_once(move || Ok(Box::new(admin_session)));
        connector
            .expect_connect()
            .times(1)
            .in_sequence(&mut sequence)
            .return_once(move || Ok(Box::new(user_session)));

        let client = UfdsClient::with_connector(sample_config(), Box::new(connector));
        let user = client.authenticate("jdoe", "password").await.unwrap();
        assert_eq!(user.login, "jdoe");
        assert!(user.is_admin());
    }

    #[tokio::test]
    async fn authenticate_unknown_user() {
        let mut connector = MockLdapConnector::new();
        let mut session = MockLdapSession::new();
        session.expect_simple_bind().returning(|_, _| Ok(()));
        session
            .expect_search()
            .returning(|_, _, _, _| Ok(Vec::new()));
        session.expect_unbind().returning(|| Ok(()));

        connector
            .expect_connect()
            .return_once(move || Ok(Box::new(session)));

        let client = UfdsClient::with_connector(sample_config(), Box::new(connector));
        let result = client.authenticate("unknown", "password").await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }
}
