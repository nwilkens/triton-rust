# triton-core

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Test Coverage](https://img.shields.io/badge/coverage-91.55%25-brightgreen.svg)](https://github.com/TritonDataCenter/triton-rust)

Core types and utilities for building Triton DataCenter integrations in Rust.

## Overview

`triton-core` provides foundational types, error handling, and HTTP client utilities for interacting with Triton DataCenter services. This crate is designed to be used as a building block for higher-level Triton client libraries and applications.

## Features

- **Type-Safe UUID Wrappers** - Strongly-typed UUIDs for different Triton resources (VMs, servers, networks, etc.)
- **Comprehensive Error Handling** - Rich error types with context and conversion from common error sources
- **Service Enumeration** - Type-safe representation of all Triton services
- **Configuration Management** - Validated configuration structures with builder patterns
- **HTTP Client Utilities** - Retry policies with exponential backoff, connection pooling
- **Endpoint Management** - Service endpoint discovery with health tracking
- **Well-Tested** - 107 unit tests with high code coverage

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
triton-core = "0.1.0"
```

## Quick Start

### Working with UUIDs

```rust
use triton_core::uuid::{VmUuid, ServerUuid};

// Create strongly-typed UUIDs
let vm_uuid = VmUuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let server_uuid = ServerUuid::new_v4();

// UUIDs are type-safe - this won't compile:
// let vm_uuid: VmUuid = server_uuid; // ❌ Compile error!

// Serialize/deserialize with serde
let json = serde_json::to_string(&vm_uuid)?;
let parsed: VmUuid = serde_json::from_str(&json)?;
```

### Configuration

```rust
use triton_core::config::{TritonClientConfig, ServiceDiscoveryConfig};
use std::time::Duration;

// Create a basic configuration
let config = TritonClientConfig::new("https://sapi.example.com")?
    .with_api_key("your-api-key")
    .with_timeout(60)
    .with_max_retries(5);

// Configure service discovery
let discovery = ServiceDiscoveryConfig::new()
    .with_cache_ttl(600)
    .with_timeout(10);

let config = config.with_service_discovery(discovery);
```

### Service Types

```rust
use triton_core::types::TritonService;

// Iterate over all services
for service in TritonService::all() {
    println!("Service: {}, Default Port: {}",
        service.name(),
        service.default_port()
    );
}

// Parse service names
let service: TritonService = "vmapi".parse()?;
assert_eq!(service, TritonService::Vmapi);
```

### Retry Policies

```rust
use triton_core::client::RetryPolicy;
use std::time::Duration;

// Create a retry policy with exponential backoff
let policy = RetryPolicy::new()
    .with_max_retries(5)
    .with_initial_delay(Duration::from_millis(100))
    .with_max_delay(Duration::from_secs(5))
    .with_backoff_multiplier(2);

// Calculate delay for each attempt
for attempt in 1..=5 {
    let delay = policy.delay_for_attempt(attempt);
    println!("Attempt {}: wait {:?}", attempt, delay);
}
// Output:
// Attempt 1: wait 100ms
// Attempt 2: wait 200ms
// Attempt 3: wait 400ms
// Attempt 4: wait 800ms
// Attempt 5: wait 1600ms
```

### Service Discovery

```rust
use triton_core::services::DiscoveryStatus;

// Track service discovery health
let status = DiscoveryStatus::new()
    .with_success(10)
    .with_cache_stats(150, 50);

println!("Discovered {} services", status.discovered_services);
println!("Cache hit ratio: {:.1}%", status.cache_hit_ratio() * 100.0);
println!("Healthy: {}", status.is_healthy());
```

### Error Handling

```rust
use triton_core::error::{Error, Result};

fn do_something() -> Result<()> {
    // Errors can be created directly
    if some_condition {
        return Err(Error::ServiceUnavailable("VMAPI is down".to_string()));
    }

    // Or converted from other error types
    let uuid = uuid::Uuid::parse_str("invalid")?; // Automatically converts to Error::InvalidUuid

    Ok(())
}

// Error handling with context
match do_something() {
    Ok(_) => println!("Success!"),
    Err(e) => {
        eprintln!("Error: {}", e);
        eprintln!("Error code: {}", e.error_code());
        if e.should_log() {
            // Log serious errors
        }
    }
}
```

## Module Overview

### `error`
Comprehensive error types for Triton operations with HTTP status code mapping and structured error responses.

### `uuid`
Strongly-typed UUID wrappers preventing UUID mix-ups at compile time:
- `VmUuid` - Virtual Machine UUIDs
- `ServerUuid` - Compute Node UUIDs
- `NetworkUuid` - Network UUIDs
- `ImageUuid` - Image UUIDs
- `PackageUuid` - Package UUIDs
- `OwnerUuid` - Owner/User UUIDs
- `AppUuid` - Application UUIDs (SAPI)
- `InstanceUuid` - Instance UUIDs (SAPI)
- `FirewallRuleUuid` - Firewall Rule UUIDs

### `types`
Core Triton domain types:
- `TritonService` - Enumeration of all Triton services
- `TransportType` - Network transport protocols
- `ServiceEndpoint` - Service endpoint information
- `EndpointList` - Collection of endpoints with filtering

### `config`
Configuration structures with validation:
- `TritonClientConfig` - Main client configuration
- `ServiceDiscoveryConfig` - Service discovery settings
- `ServiceEndpoints` - Static endpoint fallbacks
- `ServiceEndpointConfig` - Individual endpoint configuration

### `client`
HTTP client utilities:
- Service timeout constants
- `RetryPolicy` - Exponential backoff retry logic
- `ClientConfig` - HTTP client configuration

### `services`
Service discovery and integration:
- `UfdsCredentials` - LDAP authentication credentials
- `DiscoveryStatus` - Health and performance tracking
- `ServiceDiscovery` - Trait for discovery implementations

## Constants

The crate provides sensible defaults for all Triton services:

| Service | Default Timeout | Port |
|---------|----------------|------|
| VMAPI   | 30s            | 80   |
| CNAPI   | 30s            | 80   |
| NAPI    | 20s            | 80   |
| IMGAPI  | 60s            | 80   |
| PAPI    | 20s            | 80   |
| FWAPI   | 20s            | 80   |
| SAPI    | 20s            | 80   |
| UFDS    | 15s            | 636  |
| Amon    | 20s            | 80   |
| Workflow| 30s            | 80   |

## Testing

The crate has comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html

# Run specific module tests
cargo test uuid
cargo test config
cargo test client
```

**Test Results:**
- 107 unit tests
- High code coverage
- Zero unsafe code
- All tests pass

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open
```

## Design Principles

1. **Type Safety** - Use the type system to prevent errors at compile time
2. **Zero-Cost Abstractions** - Abstractions that don't impose runtime overhead
3. **Explicit Error Handling** - All errors are explicit and typed
4. **Builder Patterns** - Ergonomic APIs with method chaining
5. **Comprehensive Testing** - High test coverage with both unit and integration tests
6. **Documentation** - All public APIs have doc comments

## Contributing

Contributions are welcome! Please ensure that:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- Linter passes (`cargo clippy`)
- Security audit passes (`cargo audit`)
- New code has tests

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- **triton-admin** - Triton DataCenter administration interface
- **triton-rust** - Collection of Rust crates for Triton integration

## Resources

- [Triton DataCenter Documentation](https://docs.tritondatacenter.com/)
- [SAPI Documentation](https://github.com/TritonDataCenter/sdc-sapi)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
