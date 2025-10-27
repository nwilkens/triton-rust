//! NAPI client and data models for Triton DataCenter networking.
//!
//! Provides typed structures and asynchronous clients for interacting with Triton's Network API.

#![deny(missing_docs)]

pub mod client;
pub mod models;

pub use client::{NapiClient, NapiClientBuilder, NetworkQuery};
pub use models::{
    CreateNetworkRequest, Network, NetworkListParams, NetworkPool, Nic, UpdateNetworkRequest,
};

/// Convenient result alias sharing the `triton-core` error type.
pub type Result<T> = triton_core::Result<T>;
