//! UFDS (LDAP) client utilities for Triton DataCenter.
//!
//! This crate provides strongly-typed primitives and client abstractions for interacting with
//! UFDS, Triton's LDAP directory service.

#![deny(missing_docs)]

mod client;
mod config;
mod dn;
mod group;
mod user;

pub use client::{DirectoryModification, LdapEntry, SearchScope, UfdsClient};
pub use config::{UfdsConfig, DEFAULT_CONNECTION_TIMEOUT_SECS, DEFAULT_OPERATION_TIMEOUT_SECS};
pub use dn::{DistinguishedName, DistinguishedNameError, RelativeDistinguishedName};
pub use group::Group;
pub use user::{AccountStatus, User, UserFlags};

/// Convenient result alias that reuses the core error type.
pub type Result<T> = triton_core::Result<T>;
