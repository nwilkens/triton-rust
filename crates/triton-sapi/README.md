# triton-sapi

Asynchronous client utilities for Triton DataCenter's Services API (SAPI). The crate combines typed models, resilient HTTP helpers, and optional discovery caching to integrate Triton service metadata into Rust applications.

## Highlights

- Strongly typed SAPI models (`Application`, `Service`, `Instance`) with automatic serde handling.
- Configurable `SapiClient` built from `TritonClientConfig`, including TLS, retries, and custom query helpers.
- Service discovery support via `SapiDiscovery`, leveraging SAPI for endpoint lookups with in-memory caching and fallback endpoints.
- Comprehensive unit tests powered by `wiremock` for end-to-end request validation.

## Quick Start

```rust
use triton_core::config::TritonClientConfig;
use triton_core::types::TritonService;
use triton_sapi::{SapiClient, ServiceQuery};

#[tokio::main]
async fn main() -> triton_sapi::Result<()> {
    let config = TritonClientConfig::new("https://sapi.example.com")?
        .with_api_key("super-secret-key");

    let client = SapiClient::from_config(&config)?;

    // Fetch SAPI services filtered by name
    let services = client
        .list_services(&ServiceQuery::new().with_name("vmapi"))
        .await?;

    // Discover runtime endpoints for a Triton service
    let discovery = client.discovery();
    let endpoints = discovery.discover_service(TritonService::Vmapi.name()).await?;

    println!("Found endpoints: {endpoints:?}");
    Ok(())
}
```

## Running Tests

```bash
cargo test --package triton-sapi --offline
```

## License

Dual licensed under MIT or Apache-2.0, matching the wider Triton Rust workspace.
