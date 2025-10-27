//! VMAPI client and data models for Triton DataCenter.
//!
//! Provides typed structures and asynchronous clients for interacting with the Triton Virtual
//! Machine API (VMAPI).

#![deny(missing_docs)]

pub mod client;
pub mod models;

pub use client::{VmQuery, VmapiClient, VmapiClientBuilder};
pub use models::{
    BatchSummary, BatchVMRequest, BatchVMResponse, ChainResult, CreateSnapshotRequest,
    CreateVMRequest, JobListParams, NetworkConfig, Nic, SnapshotActionResponse, UpdateVMRequest,
    VMListParams, Vm, VmSnapshot, VmapiJob,
};

/// Convenient result alias that reuses the shared Triton error type.
pub type Result<T> = triton_core::Result<T>;
