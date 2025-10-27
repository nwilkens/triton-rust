//! CNAPI client and data models for Triton DataCenter.
//!
//! This crate exposes strongly typed structures and an asynchronous HTTP client for interacting
//! with Triton's Compute Node API (CNAPI).

#![deny(missing_docs)]

pub mod client;
pub mod models;

pub use client::{CnapiClient, CnapiClientBuilder, ServerQuery};
pub use models::{Server, ServerCapacity, ServerListParams, ServerNic, UpdateServerRequest};

/// Convenient result alias matching the shared Triton error type.
pub type Result<T> = triton_core::Result<T>;
