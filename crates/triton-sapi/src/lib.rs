//! SAPI client utilities for Triton DataCenter.
//!
//! This crate provides typed models and asynchronous clients for interacting with the
//! Triton Services API (SAPI), including service discovery integrations.

#![deny(missing_docs)]

pub mod client;
pub mod models;

pub use client::{InstanceQuery, SapiClient, SapiClientBuilder, SapiDiscovery, ServiceQuery};
pub use models::{Application, Instance, InstanceType, Service};

/// Convenient result alias that reuses the shared Triton error type.
pub type Result<T> = triton_core::Result<T>;
