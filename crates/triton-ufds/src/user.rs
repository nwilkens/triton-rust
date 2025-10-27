//! UFDS user representation and helpers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dn::DistinguishedName;
use triton_core::uuid::OwnerUuid;

/// Account status flags that reflect the LDAP operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountStatus {
    is_admin: bool,
    is_locked: bool,
    password_expired: bool,
}

impl AccountStatus {
    /// Creates a new status with all flags disabled.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            is_admin: false,
            is_locked: false,
            password_expired: false,
        }
    }

    /// Marks the account as an administrator.
    #[must_use]
    pub const fn with_admin(mut self, is_admin: bool) -> Self {
        self.is_admin = is_admin;
        self
    }

    /// Marks the account as locked.
    #[must_use]
    pub const fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = is_locked;
        self
    }

    /// Marks the account password as expired.
    #[must_use]
    pub const fn with_password_expired(mut self, password_expired: bool) -> Self {
        self.password_expired = password_expired;
        self
    }

    /// Returns true if the account is an administrator.
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        self.is_admin
    }

    /// Returns true if the account is locked.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Returns true if the password is expired.
    #[must_use]
    pub const fn is_password_expired(&self) -> bool {
        self.password_expired
    }

    /// Returns true if the account is active (not locked and password valid).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !self.is_locked && !self.password_expired
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Additional UFDS feature flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserFlags {
    /// Whether the user is approved for provisioning Triton resources.
    pub approved_for_provisioning: bool,
    /// Whether the user is a registered developer.
    pub registered_developer: bool,
    /// Whether Triton CNS (naming service) is enabled for the user.
    pub triton_cns_enabled: bool,
}

impl UserFlags {
    /// Enables or disables provisioning approval.
    #[must_use]
    pub const fn with_provisioning(mut self, approved: bool) -> Self {
        self.approved_for_provisioning = approved;
        self
    }

    /// Enables or disables the registered developer flag.
    #[must_use]
    pub const fn with_registered_developer(mut self, registered: bool) -> Self {
        self.registered_developer = registered;
        self
    }

    /// Enables or disables Triton CNS.
    #[must_use]
    pub const fn with_triton_cns(mut self, enabled: bool) -> Self {
        self.triton_cns_enabled = enabled;
        self
    }
}

/// Representation of a UFDS user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// Distinguished name of the user entry.
    pub dn: DistinguishedName,
    /// Unique identifier mapped to `OwnerUuid`.
    pub uuid: OwnerUuid,
    /// Login name (usually the `uid` attribute).
    pub login: String,
    /// Primary email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Common name.
    #[serde(default)]
    pub cn: Option<String>,
    /// Surname (last name).
    #[serde(default)]
    pub sn: Option<String>,
    /// Given name (first name).
    #[serde(default)]
    pub given_name: Option<String>,
    /// Company affiliation.
    #[serde(default)]
    pub company: Option<String>,
    /// Phone number.
    #[serde(default)]
    pub phone: Option<String>,
    /// Account status flags.
    pub status: AccountStatus,
    /// Feature flags.
    pub flags: UserFlags,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Group names the user belongs to.
    #[serde(default)]
    pub groups: Vec<String>,
}

impl User {
    /// Creates a builder for a new user instance.
    #[must_use]
    pub fn builder(
        dn: DistinguishedName,
        uuid: OwnerUuid,
        login: impl Into<String>,
    ) -> UserBuilder {
        UserBuilder {
            dn,
            uuid,
            login: login.into(),
            email: None,
            cn: None,
            sn: None,
            given_name: None,
            company: None,
            phone: None,
            status: AccountStatus::default(),
            flags: UserFlags::default(),
            created_at: None,
            updated_at: None,
            groups: Vec::new(),
        }
    }

    /// Returns true if the account is active (not locked and password valid).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// Returns true if the user is an administrator.
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        self.status.is_admin()
    }

    /// Returns true if the user belongs to the provided group (case-insensitive).
    #[must_use]
    pub fn in_group(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g.eq_ignore_ascii_case(group))
    }

    /// Returns the preferred display name (common name when available).
    #[must_use]
    pub fn display_name(&self) -> Option<String> {
        if let Some(cn) = &self.cn {
            return Some(cn.clone());
        }

        match (&self.given_name, &self.sn) {
            (Some(given), Some(sn)) => Some(format!("{given} {sn}")),
            (Some(given), None) => Some(given.clone()),
            (None, Some(sn)) => Some(sn.clone()),
            _ => None,
        }
    }
}

/// Builder for [`User`].
#[derive(Debug)]
pub struct UserBuilder {
    dn: DistinguishedName,
    uuid: OwnerUuid,
    login: String,
    email: Option<String>,
    cn: Option<String>,
    sn: Option<String>,
    given_name: Option<String>,
    company: Option<String>,
    phone: Option<String>,
    status: AccountStatus,
    flags: UserFlags,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    groups: Vec<String>,
}

impl UserBuilder {
    /// Sets the email address.
    #[must_use]
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the common name.
    #[must_use]
    pub fn cn(mut self, cn: impl Into<String>) -> Self {
        self.cn = Some(cn.into());
        self
    }

    /// Sets the surname.
    #[must_use]
    pub fn sn(mut self, sn: impl Into<String>) -> Self {
        self.sn = Some(sn.into());
        self
    }

    /// Sets the given name.
    #[must_use]
    pub fn given_name(mut self, given_name: impl Into<String>) -> Self {
        self.given_name = Some(given_name.into());
        self
    }

    /// Sets the company.
    #[must_use]
    pub fn company(mut self, company: impl Into<String>) -> Self {
        self.company = Some(company.into());
        self
    }

    /// Sets the phone number.
    #[must_use]
    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    /// Overrides the account status.
    #[must_use]
    pub fn status(mut self, status: AccountStatus) -> Self {
        self.status = status;
        self
    }

    /// Overrides the feature flags.
    #[must_use]
    pub fn flags(mut self, flags: UserFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the creation timestamp.
    #[must_use]
    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Sets the update timestamp.
    #[must_use]
    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = Some(updated_at);
        self
    }

    /// Appends a group name.
    #[must_use]
    pub fn add_group(mut self, group: impl Into<String>) -> Self {
        self.groups.push(group.into());
        self
    }

    /// Replaces the group list.
    #[must_use]
    pub fn groups<I>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.groups = groups.into_iter().collect();
        self
    }

    /// Finalises the builder and returns the [`User`].
    #[must_use]
    pub fn build(self) -> User {
        User {
            dn: self.dn,
            uuid: self.uuid,
            login: self.login,
            email: self.email,
            cn: self.cn,
            sn: self.sn,
            given_name: self.given_name,
            company: self.company,
            phone: self.phone,
            status: self.status,
            flags: self.flags,
            created_at: self.created_at,
            updated_at: self.updated_at,
            groups: self.groups,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dn::DistinguishedName;

    #[test]
    fn account_status_helpers() {
        let status = AccountStatus::new()
            .with_admin(true)
            .with_locked(false)
            .with_password_expired(false);
        assert!(status.is_admin());
        assert!(status.is_active());
    }

    #[test]
    fn user_builder_constructs_expected_user() {
        let dn = DistinguishedName::parse("uid=jdoe,ou=People,dc=example,dc=com").unwrap();
        let uuid = OwnerUuid::new_v4();
        let status = AccountStatus::new().with_locked(true);
        let flags = UserFlags::default().with_provisioning(true);
        let created_at = Utc::now();

        let user = User::builder(dn.clone(), uuid, "jdoe")
            .email("jdoe@example.com")
            .cn("John Doe")
            .sn("Doe")
            .given_name("John")
            .company("Example Inc.")
            .phone("+1-555-0100")
            .status(status)
            .flags(flags)
            .created_at(created_at)
            .add_group("admins")
            .build();

        assert_eq!(user.dn, dn);
        assert_eq!(user.login, "jdoe");
        assert_eq!(user.email.as_deref(), Some("jdoe@example.com"));
        assert!(!user.is_active());
        assert!(user.in_group("Admins"));
        assert_eq!(user.display_name().unwrap(), "John Doe");
    }
}
