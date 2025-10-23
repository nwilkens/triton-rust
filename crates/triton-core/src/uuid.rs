//! Strongly-typed UUID wrappers for Triton resources.
//!
//! This module provides type-safe UUID wrappers for different Triton resources,
//! preventing UUID mix-ups at compile time.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{Error, Result};

/// Macro to generate strongly-typed UUID wrapper types.
macro_rules! uuid_type {
    ($(#[$meta:meta])* $name:ident, $doc:expr) => {
        $(#[$meta])*
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new UUID wrapper from a [`Uuid`].
            #[must_use]
            pub const fn new(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Creates a new random UUID (v4).
            #[must_use]
            pub fn new_v4() -> Self {
                Self(Uuid::new_v4())
            }

            /// Returns the inner [`Uuid`].
            #[must_use]
            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// Converts to the inner [`Uuid`].
            #[must_use]
            pub const fn into_uuid(self) -> Uuid {
                self.0
            }

            /// Parses a UUID from a string.
            ///
            /// # Errors
            ///
            /// Returns an error if the string is not a valid UUID.
            pub fn parse_str(input: &str) -> Result<Self> {
                Uuid::parse_str(input)
                    .map(Self)
                    .map_err(|_| Error::InvalidUuid(input.to_string()))
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(wrapper: $name) -> Self {
                wrapper.0
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self> {
                Self::parse_str(s)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }
    };
}

// Generate all UUID types
uuid_type!(VmUuid, "Virtual Machine UUID");
uuid_type!(ServerUuid, "Server/Compute Node UUID");
uuid_type!(NetworkUuid, "Network UUID");
uuid_type!(ImageUuid, "Image UUID");
uuid_type!(PackageUuid, "Package UUID");
uuid_type!(OwnerUuid, "Owner/User UUID");
uuid_type!(AppUuid, "Application UUID (SAPI)");
uuid_type!(InstanceUuid, "Instance UUID (SAPI)");
uuid_type!(FirewallRuleUuid, "Firewall Rule UUID");

/// Validates a UUID string.
///
/// # Errors
///
/// Returns an error if the string is not a valid UUID.
pub fn validate_uuid(s: &str) -> Result<Uuid> {
    Uuid::parse_str(s).map_err(|_| Error::InvalidUuid(s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_UUID: &str = "550e8400-e29b-41d4-a716-446655440000";
    const INVALID_UUID: &str = "not-a-uuid";

    #[test]
    fn test_vm_uuid_new() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid = VmUuid::new(uuid);
        assert_eq!(vm_uuid.as_uuid(), &uuid);
    }

    #[test]
    fn test_vm_uuid_new_v4() {
        let vm_uuid = VmUuid::new_v4();
        assert!(vm_uuid.as_uuid().get_version_num() == 4);
    }

    #[test]
    fn test_vm_uuid_parse_str_valid() {
        let result = VmUuid::parse_str(VALID_UUID);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), VALID_UUID);
    }

    #[test]
    fn test_vm_uuid_parse_str_invalid() {
        let result = VmUuid::parse_str(INVALID_UUID);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidUuid(_)));
    }

    #[test]
    fn test_vm_uuid_from_str() {
        let result: Result<VmUuid> = VALID_UUID.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_vm_uuid_display() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid = VmUuid::new(uuid);
        assert_eq!(vm_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_vm_uuid_into_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid = VmUuid::new(uuid);
        let converted: Uuid = vm_uuid.into_uuid();
        assert_eq!(converted, uuid);
    }

    #[test]
    fn test_vm_uuid_from_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid: VmUuid = uuid.into();
        assert_eq!(vm_uuid.as_uuid(), &uuid);
    }

    #[test]
    fn test_vm_uuid_as_ref() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid = VmUuid::new(uuid);
        let uuid_ref: &Uuid = vm_uuid.as_ref();
        assert_eq!(uuid_ref, &uuid);
    }

    #[test]
    fn test_vm_uuid_serialize() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid = VmUuid::new(uuid);
        let json = serde_json::to_string(&vm_uuid).unwrap();
        assert_eq!(json, format!("\"{}\"", VALID_UUID));
    }

    #[test]
    fn test_vm_uuid_deserialize() {
        let json = format!("\"{}\"", VALID_UUID);
        let vm_uuid: VmUuid = serde_json::from_str(&json).unwrap();
        assert_eq!(vm_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_vm_uuid_eq() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid1 = VmUuid::new(uuid);
        let vm_uuid2 = VmUuid::new(uuid);
        assert_eq!(vm_uuid1, vm_uuid2);
    }

    #[test]
    fn test_vm_uuid_clone() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid = VmUuid::new(uuid);
        let cloned = vm_uuid;
        assert_eq!(vm_uuid, cloned);
    }

    #[test]
    fn test_server_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let server_uuid = ServerUuid::new(uuid);
        assert_eq!(server_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_network_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let network_uuid = NetworkUuid::new(uuid);
        assert_eq!(network_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_image_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let image_uuid = ImageUuid::new(uuid);
        assert_eq!(image_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_package_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let package_uuid = PackageUuid::new(uuid);
        assert_eq!(package_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_owner_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let owner_uuid = OwnerUuid::new(uuid);
        assert_eq!(owner_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_app_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let app_uuid = AppUuid::new(uuid);
        assert_eq!(app_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_instance_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let instance_uuid = InstanceUuid::new(uuid);
        assert_eq!(instance_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_firewall_rule_uuid() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let rule_uuid = FirewallRuleUuid::new(uuid);
        assert_eq!(rule_uuid.to_string(), VALID_UUID);
    }

    #[test]
    fn test_validate_uuid_valid() {
        let result = validate_uuid(VALID_UUID);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        let result = validate_uuid(INVALID_UUID);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidUuid(_)));
    }

    #[test]
    fn test_different_uuid_types_are_different() {
        let uuid = Uuid::parse_str(VALID_UUID).unwrap();
        let vm_uuid = VmUuid::new(uuid);
        let server_uuid = ServerUuid::new(uuid);

        // These are different types, so we test that they serialize the same way
        // but are incompatible types at compile time
        assert_eq!(vm_uuid.to_string(), server_uuid.to_string());
    }

    #[test]
    fn test_uuid_hash() {
        use std::collections::HashSet;

        let uuid1 = Uuid::parse_str(VALID_UUID).unwrap();
        let uuid2 = Uuid::new_v4();

        let vm_uuid1 = VmUuid::new(uuid1);
        let vm_uuid2 = VmUuid::new(uuid2);
        let vm_uuid3 = VmUuid::new(uuid1);

        let mut set = HashSet::new();
        set.insert(vm_uuid1);
        set.insert(vm_uuid2);
        set.insert(vm_uuid3);

        // vm_uuid1 and vm_uuid3 are equal, so should only be counted once
        assert_eq!(set.len(), 2);
    }
}
