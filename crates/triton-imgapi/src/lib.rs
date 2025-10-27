//! IMGAPI client and data models for Triton DataCenter.
//!
//! Provides strongly typed models and an asynchronous client for interacting
//! with the Triton Image API (IMGAPI).

#![deny(missing_docs)]

pub mod client;
pub mod models;

pub use client::{ImgapiClient, ImgapiClientBuilder, ImgapiDiscovery};
pub use models::{
    CreateImageRequest, ExportImageRequest, Image, ImageAction, ImageFile, ImageImportRequest,
    ImageListParams, ImageRequirements, ImageUser, ImportImageSource, UpdateImageRequest,
};

/// Convenient result alias using the shared Triton error type.
pub type Result<T> = triton_core::Result<T>;
