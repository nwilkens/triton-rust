# AI Agent Handoff Guide for triton-rust

This document provides context and guidance for AI agents working on the triton-rust project.

## Project Overview

**triton-rust** is a collection of Rust crates for integrating with Triton DataCenter. The project aims to provide type-safe, well-tested, and idiomatic Rust clients for all Triton services.

### Repository Structure

```
triton-rust/
├── Cargo.toml                    # Workspace configuration
├── .gitignore
├── AGENTS.md                     # This file
├── crates/
│   ├── triton-core/              # ✅ COMPLETE - Foundation crate
│   ├── triton-ufds/              # ✅ COMPLETE - UFDS client + models
│   └── triton-sapi/              # ✅ COMPLETE - SAPI client + discovery
└── services/                     # (Planned) Rust service implementations
    └── ...                       # e.g., triton-sapi-server, triton-cnapi-server
```

## Current Status

### ✅ Completed: triton-core v0.1.0

**Status**: Production-ready, 107 tests passing, well-documented

**What it provides:**
- Strongly-typed UUID wrappers (VmUuid, ServerUuid, NetworkUuid, etc.)
- Comprehensive error handling with HTTP status mapping
- Service enumeration (10 Triton services)
- Configuration structures with validation
- HTTP retry policies with exponential backoff
- Service discovery abstractions
- Endpoint management with health tracking

**Key Design Principles:**
1. Type safety - Use compile-time checks to prevent errors
2. Zero-cost abstractions - No runtime overhead
3. Explicit error handling - All errors are typed
4. Builder patterns - Ergonomic APIs with method chaining
5. Comprehensive testing - High test coverage
6. Documentation - All public APIs documented

### ✅ Completed: triton-ufds v0.1.0

**Status**: Production-ready, async LDAP client with comprehensive unit coverage

**Highlights:**
- Distinguished Name parser with strict validation
- `User`, `Group`, and flag models mapped from UFDS responses
- Configurable `UfdsClient` supporting TLS overrides, retryable operations, and group membership helpers
- Mockable connector/session traits for deterministic tests

### ✅ Completed: triton-sapi v0.1.0

**Status**: Production-ready, async HTTP client + discovery layer with wiremock-backed tests

**Highlights:**
- Typed SAPI models (`Application`, `Service`, `Instance`) including optional fields and serde defaults
- `SapiClient` with retry-aware request executor and fluent query builders
- `SapiDiscovery` implementation of `ServiceDiscovery`, providing caching, fallback endpoints, and status tracking
- Workspace documentation (`README.md`) and usage examples

### ✅ Completed: triton-cnapi v0.1.0

**Status**: Production-ready, async CNAPI client with typed server models and discovery adapter

**Highlights:**
- `Server`, `ServerNic`, and `UpdateServerRequest` models mirroring CNAPI payloads with chrono-backed timestamps
- `CnapiClient` supporting query builders, updates, basic auth or token headers, and retry logic
- `ServerListParams` helpers for composing `/servers` filters and pagination
- `CnapiDiscovery` wrapper that reuses the SAPI discovery implementation to locate CNAPI endpoints

### ✅ Completed: triton-vmapi v0.1.0

**Status**: Production-ready, async VMAPI client with VM lifecycle helpers and batch/snapshot support

**Highlights:**
- `Vm`, `Nic`, `VmapiJob`, and snapshot models reflecting VMAPI responses with tolerant serde handling
- `VmapiClient` implementing list/get/create/update/delete flows plus snapshot and batch operations
- `VmQuery`/`JobListParams` builders for common filtering and pagination use-cases
- `VmapiDiscovery` adapter piggybacking on SAPI discovery to resolve VMAPI endpoints

### ✅ Completed: triton-napi v0.1.0

**Status**: Production-ready, async NAPI client covering network, pool, and NIC management

**Highlights:**
- `Network`, `NetworkPool`, and `Nic` models with strong typing and serde support
- `NapiClient` helpers for network CRUD, network pool lookups, and NIC operations with retry-aware HTTP
- `NetworkQuery` builder for list filters and pagination
- `NapiDiscovery` wrapper delegating endpoint discovery to SAPI

### ✅ Completed: triton-papi v0.1.0

**Status**: Production-ready, async PAPI client with package lifecycle helpers

**Highlights:**
- `Package`, `PackageNetwork`, and request models with typed UUID handling and serde support
- `PapiClient` leveraging the shared `ServiceClient` for list/get/create/update/delete flows
- `PackageListParams` query builder using the core `QueryParams` helper
- `PapiDiscovery` wrapper reusing the shared discovery proxy to resolve endpoints via SAPI

### ✅ Completed: triton-fwapi v0.1.0

**Status**: Production-ready, async FWAPI client managing firewall rule lifecycle

**Highlights:**
- `FirewallRule` models with typed UUID wrappers and serde defaults for metadata
- `FwapiClient` leveraging `ServiceClient` for list/get/create/update/delete rule operations
- `FirewallRuleListParams` builder using shared query helpers for filtering
- `FwapiDiscovery` adapter built on the core `ServiceDiscoveryProxy` for SAPI-backed endpoint resolution

### Git History

```
015ea31 Remove unused CacheStats type
ac3e5a1 Add comprehensive README and finalize triton-core
3d3295e Implement client.rs module with 19 comprehensive tests
4d81c6a Implement services.rs module with 13 comprehensive tests
e27a2d3 Implement config.rs module with 23 comprehensive tests
2c498f4 Add types module with TritonService enum and endpoint types
20364e0 Add triton-core crate with error and UUID modules
```

## Reference Materials

### Source Analysis Documents

Located in `/tmp/` (from initial exploration):
- `triton-core-extraction-analysis.md` - Comprehensive analysis of what to extract
- `extraction-summary.txt` - Quick reference with file paths and line numbers
- `triton-core-structure.txt` - Proposed directory structure and types

### Reference Implementation

**triton-admin backend** at `/Users/nwilkens/workspace/triton-admin/backend/src/`:
- `error.rs` - Original error handling (214 lines)
- `config.rs` - Configuration structures (448 lines)
- `services/discovery_manager.rs` - Service discovery (450+ lines)
- `services/client_factory.rs` - HTTP client patterns (969 lines)
- `services/endpoint_cache.rs` - Endpoint caching (200+ lines)
- `services/sapi_client.rs` - SAPI client implementation (425 lines)
- `models/*.rs` - Domain model types (VMAPI, CNAPI, NAPI, PAPI, IMGAPI, FWAPI)

## Mono-repo Architecture Direction

While the immediate focus remains on Rust client libraries, the long-term vision includes porting the NodeJS Triton services into this mono-repo. With that future work in mind, keep the following layering pattern in consideration (no server-side code lives here yet):

1. **Shared types crates** – e.g. `crates/triton-sapi-types`, `crates/triton-cnapi-types`. These expose serde-friendly models that both the client and server can share.
2. **Client crates** – e.g. `crates/triton-sapi` (already in place), depending on the types crate and offering reusable integration logic.
3. **Server crates/binaries (future)** – when the service ports land, place them under `services/<name>/` (e.g., `services/triton-sapi-server`). They can depend on the shared types and clients without introducing circular dependencies. For now this is informational only; no server code is tracked in this repository.

Keeping these boundaries in mind lets us evolve clients independently today while leaving a clear runway for the eventual server migrations.

## Next Steps

1. **Add shared SAPI types crate (optional, prep work)**
   - If we anticipate reusing the SAPI models beyond the client, extract them into `crates/triton-sapi-types/` and have the client re-export from there.
   - Keep this lightweight; the move can wait until another crate needs the types.

2. **Next client targets (IMGAPI → PAPI → FWAPI)**
   - Build dedicated async clients for each API in that order, following the established retry, discovery, and builder patterns.
   - Use the corresponding NodeJS services in `~/workspace/triton/sdc-imgapi`, `~/workspace/triton/sdc-papi`, and `~/workspace/triton/sdc-fwapi` as authoritative server-side references while translating models and behaviours.
   - Mirror the testing approach used for UFDS/SAPI (serde round-trips, wiremock stubs, trait mocks), capturing any gaps needed to support downstream reuse.

3. **Common utilities**
   - Note any shared patterns (auth headers, pagination helpers, config loaders) that should live in `triton-core` before more clients arrive.
   - Shared HTTP/service plumbing now lives in `triton-core` (`ServiceClient`, `ServiceDiscoveryProxy`, `QueryParams`); new clients should build on these instead of re-implementing retry/discovery logic.

4. **Server-port awareness (future)**
   - Continue designing clients and shared types with the eventual server migration in mind (e.g., avoid naming collisions, keep builder APIs extensible).

These client extractions directly support the ongoing work to tease apart `~/workspace/triton-admin`, letting other Triton projects depend on reusable crates as we migrate services from the NodeJS implementations toward Rust.

### IMGAPI client design snapshot

- **Crate shape**: mirror `triton-vmapi`/`triton-cnapi` with `client.rs` and `models.rs`, re-exported from `lib.rs`, and depend on `triton-core` for errors, UUIDs, retry defaults, and `TritonService::Imgapi` wiring.
- **Client API**: `ImgapiClientBuilder` (configurable retry/auth/token), methods for list/get/create/update/delete images, action helpers (activate/enable/disable), file import/export flows, and streaming file access backed by `reqwest`.
- **Discovery**: `ImgapiDiscovery` wrapping SAPI discovery with `DiscoveryStatus` tracking, matching the pattern used by VMAPI (`Arc<RwLock<DiscoveryStatus>>` plus success/error instrumentation).
- **Models**: port `ImageListParams`, `Image`, `CreateImageRequest`, `UpdateImageRequest`, `ImportImageRequest`, `ExportImageRequest`, and support structs from `triton-admin` (`backend/src/models/imgapi.rs`) while upgrading to strongly-typed UUID wrappers and serde helpers for tag/trait maps.
- **Testing**: wiremock-driven client tests covering happy paths, error retries, and auth headers, alongside serde round-trips for every model; include discovery cache tests using mocked `ServiceDiscovery`.
- **References**: source behaviour from `~/workspace/triton/sdc-imgapi` (Node service) and the Rust handlers in `~/workspace/triton-admin/backend/src/api/imgapi/`, ensuring parity during the migration from NodeJS to Rust.

### PAPI client design snapshot

- **Crate shape**: add `crates/triton-papi` with the usual `client.rs` + `models.rs` re-exported from `lib.rs`, depending on `triton-core` for error types, UUID wrappers (`PackageUuid`, `OwnerUuid`), query helpers, `ServiceClient`, and discovery proxy support.
- **Client API**: `PapiClientBuilder` reusing `ServiceClientBuilder` defaults (`PAPI_DEFAULT_TIMEOUT`), methods for listing packages with filters, retrieving by UUID, creating/updating/deleting packages, plus any trait/ACL helpers surfaced by the Node implementation. Authentication hooks remain optional via `with_basic_auth` / `with_token`.
- **Models**: port `Package`, `PackageNetwork`, `CreatePackageRequest`, `UpdatePackageRequest`, and `PackageListParams` from `triton-admin/backend/src/models/papi.rs`, tightening types where possible (UUID wrappers, bool maps) and using `triton_core::query::QueryParams` for list filters.
- **Discovery**: `PapiDiscovery` constructed via `ServiceDiscoveryProxy::for_service(…, TritonService::Papi)` to mirror IMGAPI/VMAPI consistency and preserve status metrics.
- **Testing**: wiremock-backed client tests for list/get/create/update/delete flows plus error mapping, and serde round-trips for the key request/response models. Reuse the shared discovery proxy tests to ensure delegation works.
- **References**: implementation details drawn from `~/workspace/triton-admin/backend/src/api/papi/` and the legacy service code in `~/workspace/triton/sdc-papi`, keeping parity with existing behaviour as functionality migrates to Rust.

## Development Workflow

### Creating a New Crate

1. **Add crate to workspace:**
   ```toml
   # In triton-rust/Cargo.toml
   members = ["crates/*", "crates/new-crate"]
   ```

2. **Create crate structure:**
   ```bash
   cd triton-rust/crates
   cargo new --lib triton-service-name
   ```

3. **Add triton-core dependency:**
   ```toml
   [dependencies]
   triton-core = { path = "../triton-core" }
   ```

4. **Follow triton-core patterns:**
   - Use strongly-typed UUIDs from triton-core
   - Use Error type from triton-core
   - Use TritonService enum from triton-core
   - Follow builder pattern for configuration
   - Write comprehensive tests (aim for 90%+ coverage)

### Testing Requirements

**Per the project's CLAUDE.md requirements:**

1. **Atomic commits** - One logical change per commit
2. **Update documentation** - With each commit
3. **Build test** - Run after each commit: `cargo build`
4. **Security audit** - Run: `cargo audit`
5. **No "Generated with Claude"** - Don't add co-authorship tags
6. **Small modules** - Keep code organized and focused
7. **Reusable components** - Check for redundancy
8. **Tests for every change** - Write tests for all new code

### Testing Commands

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test --package triton-core

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Security audit
cargo audit

# Format check
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Build release
cargo build --release

# Generate documentation
cargo doc --no-deps --open
```

### Commit Message Format

```
<module>: <action> <target>

- Bullet point of what changed
- Another change
- Test results
```

Example:
```
triton-core: Add UUID types module

- Implement 9 strongly-typed UUID wrappers
- Add validation and parsing utilities
- Include 26 comprehensive unit tests
- All tests passing
```

## Important Constants

### Service Timeouts (from triton-core)

```rust
VMAPI_DEFAULT_TIMEOUT = 30 seconds
CNAPI_DEFAULT_TIMEOUT = 30 seconds
NAPI_DEFAULT_TIMEOUT = 20 seconds
IMGAPI_DEFAULT_TIMEOUT = 60 seconds
PAPI_DEFAULT_TIMEOUT = 20 seconds
FWAPI_DEFAULT_TIMEOUT = 20 seconds
SAPI_DEFAULT_TIMEOUT = 20 seconds
UFDS_DEFAULT_TIMEOUT = 15 seconds
AMON_DEFAULT_TIMEOUT = 20 seconds
WORKFLOW_DEFAULT_TIMEOUT = 30 seconds
```

### Service Ports

```rust
DEFAULT_HTTP_PORT = 80
DEFAULT_HTTPS_PORT = 443
DEFAULT_LDAPS_PORT = 636  // UFDS uses this
```

### Retry Configuration

```rust
DEFAULT_MAX_RETRIES = 3
DEFAULT_RETRY_DELAY_MS = 500
DEFAULT_RETRY_MAX_DELAY_MS = 5000
```

## Code Patterns

### Using triton-core Types

```rust
use triton_core::{
    error::{Error, Result},
    uuid::VmUuid,
    types::TritonService,
    config::TritonClientConfig,
};

// Parse UUIDs
let vm_uuid = VmUuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;

// Handle errors
fn do_something() -> Result<()> {
    Err(Error::ServiceUnavailable("VMAPI down".to_string()))
}

// Use service enum
let service = TritonService::Vmapi;
println!("Service: {}, Port: {}", service.name(), service.default_port());
```

### Configuration Pattern

```rust
use triton_core::config::TritonClientConfig;

let config = TritonClientConfig::new("https://sapi.example.com")?
    .with_api_key("api-key")
    .with_timeout(60)
    .with_max_retries(5);
```

### Retry Pattern

```rust
use triton_core::client::RetryPolicy;
use std::time::Duration;

let policy = RetryPolicy::new()
    .with_max_retries(5)
    .with_initial_delay(Duration::from_millis(500))
    .with_backoff_multiplier(2);

for attempt in 1..=policy.max_retries {
    let delay = policy.delay_for_attempt(attempt);
    // Use delay for retry logic
}
```

## Common Pitfalls

### 1. Don't Mix UUID Types
```rust
// ❌ WRONG - Won't compile (good!)
let vm_uuid: VmUuid = ServerUuid::new_v4();

// ✅ CORRECT
let vm_uuid = VmUuid::new_v4();
```

### 2. Always Use Result Type
```rust
// ❌ WRONG
use triton_core::Error;
fn foo() -> std::result::Result<(), Error> { }

// ✅ CORRECT
use triton_core::Result;
fn foo() -> Result<()> { }
```

### 3. Builder Methods Should Be Const When Possible
```rust
// ✅ CORRECT
#[must_use]
pub const fn with_timeout(mut self, seconds: u64) -> Self {
    self.timeout_secs = seconds;
    self
}
```

### 4. Use #[must_use] for Builders
```rust
// ✅ CORRECT - Warns if return value ignored
#[must_use]
pub fn with_api_key(mut self, key: String) -> Self { }
```

## Architecture Decisions

### Why Separate Crates?

1. **Modularity** - Use only what you need
2. **Compilation speed** - Don't rebuild everything
3. **Clear boundaries** - Well-defined interfaces
4. **Testability** - Easier to test in isolation
5. **Reusability** - Mix and match as needed

### Why Strongly-Typed UUIDs?

Prevents bugs like:
```rust
// With String UUIDs - compiles but wrong! 🐛
let vm = get_vm(server_uuid);

// With typed UUIDs - won't compile! ✅
let vm = get_vm(server_uuid); // ❌ Type error!
let vm = get_vm(vm_uuid);     // ✅ Correct
```

### Why Builder Patterns?

```rust
// Readable and flexible
let config = TritonClientConfig::new("https://sapi.example.com")?
    .with_api_key("key")
    .with_timeout(60);

// vs. this mess:
let config = TritonClientConfig::new(
    "https://sapi.example.com",
    Some("key"),
    None,
    60,
    3,
    // ... many more parameters
)?;
```

## Questions & Contact

### Common Questions

**Q: Should I add a new type to triton-core or my service crate?**
A: Add to triton-core if it's used by multiple services. Keep service-specific types in their own crate.

**Q: What test coverage should I aim for?**
A: Aim for 90%+ coverage. triton-core achieved 91.55% before removing unused code.

**Q: Should I use async or sync?**
A: Use async - Triton operations are I/O bound. Use tokio as the runtime (already in workspace).

**Q: How do I handle optional fields in API responses?**
A: Use `Option<T>` and `#[serde(skip_serializing_if = "Option::is_none")]`

### Getting Started Checklist

When starting work on this project:

- [ ] Read this entire document
- [ ] Review triton-core README
- [ ] Look at triton-core source code for patterns
- [ ] Review reference implementation in triton-admin
- [ ] Run `cargo test` to ensure everything works
- [ ] Run `cargo audit` to check security
- [ ] Familiarize yourself with workspace structure

### Useful Commands

```bash
# See all crates in workspace
cargo workspaces list

# Build everything
cargo build --workspace

# Test everything
cargo test --workspace

# Update all dependencies
cargo update

# Check what would be published
cargo publish --dry-run --package triton-core
```

## Resources

### Documentation
- [Triton DataCenter Docs](https://docs.tritondatacenter.com/)
- [SAPI Documentation](https://github.com/TritonDataCenter/sdc-sapi)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### triton-admin Backend
Location: `/Users/nwilkens/workspace/triton-admin/backend/`
- Reference implementation for all Triton integrations
- Models, services, and API handlers to extract from

### Analysis Documents
Location: `/tmp/`
- Detailed extraction analysis with code examples
- File paths and line numbers for reference
- Architecture recommendations

---

**Last Updated:** 2025-10-24
**Project Status:** triton-core complete, ready for service crates
**Next Priority:** Implement triton-imgapi, triton-papi, and triton-fwapi clients

## Notes for AI Agents

- **Be proactive with tests** - Write tests as you write code
- **Use atomic commits** - Commit frequently with clear messages
- **Run cargo audit** - After every significant change
- **Keep modules small** - Break large files into logical modules
- **Check for reusability** - Look for patterns to extract to triton-core
- **Document as you go** - Write doc comments for all public APIs
- **Follow existing patterns** - Look at triton-core for examples

Good luck! The foundation is solid. Time to build the service clients! 🚀
