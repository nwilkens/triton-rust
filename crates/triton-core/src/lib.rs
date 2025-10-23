//! # triton-core
//!
//! Core types and utilities for working with Triton DataCenter.
//!
//! This crate provides foundational types, error handling, and HTTP client utilities
//! for building Triton DataCenter integrations.
//!
//! ## Modules
//!
//! - [`error`] - Error types and HTTP status code mapping
//! - [`uuid`] - Strongly-typed UUID wrappers for Triton resources
//! - [`types`] - Core Triton domain types (VMs, networks, packages, etc.)
//! - [`config`] - Configuration structures for Triton clients
//! - [`client`] - HTTP client utilities, retry logic, and caching
//! - [`services`] - Service discovery and integration patterns

#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod config;
pub mod error;
pub mod services;
pub mod types;
pub mod uuid;

// Re-export commonly used types
pub use error::{Error, Result};
