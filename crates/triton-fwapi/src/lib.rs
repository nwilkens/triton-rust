//! FWAPI client and data models for Triton DataCenter.
//!
//! Provides typed structures and asynchronous client utilities for interacting
//! with the Triton Firewall API (FWAPI).

#![deny(missing_docs)]

pub mod client;
pub mod models;

pub use client::{FwapiClient, FwapiClientBuilder, FwapiDiscovery};
pub use models::{
    CreateFirewallRuleRequest, FirewallRule, FirewallRuleListParams, UpdateFirewallRuleRequest,
};

/// Convenient result alias that reuses the shared Triton error type.
pub type Result<T> = triton_core::Result<T>;
