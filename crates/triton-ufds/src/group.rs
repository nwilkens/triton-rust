//! UFDS group representation.

use serde::{Deserialize, Serialize};

use crate::dn::DistinguishedName;

/// Representation of a UFDS group entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Distinguished name of the group.
    pub dn: DistinguishedName,
    /// Canonical group name (usually the `cn` attribute).
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Distinguished names for group members.
    #[serde(default)]
    pub members: Vec<DistinguishedName>,
}

impl Group {
    /// Creates a new builder with the required fields.
    #[must_use]
    pub fn builder(dn: DistinguishedName, name: impl Into<String>) -> GroupBuilder {
        GroupBuilder {
            dn,
            name: name.into(),
            description: None,
            members: Vec::new(),
        }
    }

    /// Returns the number of members in the group.
    #[must_use]
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Checks whether the given distinguished name is a member of this group.
    #[must_use]
    pub fn has_member(&self, member_dn: &DistinguishedName) -> bool {
        self.members.iter().any(|dn| dn == member_dn)
    }
}

/// Builder for [`Group`].
#[derive(Debug)]
pub struct GroupBuilder {
    dn: DistinguishedName,
    name: String,
    description: Option<String>,
    members: Vec<DistinguishedName>,
}

impl GroupBuilder {
    /// Sets the group description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends a member distinguished name.
    #[must_use]
    pub fn add_member(mut self, dn: DistinguishedName) -> Self {
        self.members.push(dn);
        self
    }

    /// Appends multiple members.
    #[must_use]
    pub fn members<I>(mut self, members: I) -> Self
    where
        I: IntoIterator<Item = DistinguishedName>,
    {
        self.members.extend(members);
        self
    }

    /// Builds the [`Group`].
    #[must_use]
    pub fn build(self) -> Group {
        Group {
            dn: self.dn,
            name: self.name,
            description: self.description,
            members: self.members,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dn::DistinguishedName;

    #[test]
    fn builder_creates_group() {
        let dn = DistinguishedName::parse("cn=admins,ou=Groups,dc=example,dc=com").unwrap();
        let member = DistinguishedName::parse("uid=jane,ou=People,dc=example,dc=com").unwrap();
        let group = Group::builder(dn.clone(), "admins")
            .description("Administrators")
            .add_member(member.clone())
            .build();

        assert_eq!(group.dn, dn);
        assert_eq!(group.name, "admins");
        assert_eq!(group.member_count(), 1);
        assert!(group.has_member(&member));
    }
}
