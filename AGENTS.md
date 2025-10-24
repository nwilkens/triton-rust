# AI Agent Handoff Guide for triton-rust

This document provides context and guidance for AI agents working on the triton-rust project.

## Project Overview

**triton-rust** is a collection of Rust crates for integrating with Triton DataCenter. The project aims to provide type-safe, well-tested, and idiomatic Rust clients for all Triton services.

### Repository Structure

```
triton-rust/
├── Cargo.toml              # Workspace configuration
├── .gitignore
├── AGENTS.md               # This file
└── crates/
    └── triton-core/        # ✅ COMPLETE - Foundation crate
        ├── Cargo.toml
        ├── README.md       # Comprehensive documentation
        └── src/
            ├── lib.rs      # Main library entry
            ├── error.rs    # Error types (42 lines, 11 tests)
            ├── uuid.rs     # UUID wrappers (24 lines, 26 tests)
            ├── types.rs    # Core types (74 lines, 21 tests)
            ├── config.rs   # Configuration (108 lines, 23 tests)
            ├── services.rs # Service discovery (41 lines, 13 tests)
            └── client.rs   # Client utilities (66 lines, 13 tests)
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

## Next Steps: Future Crates

### Priority 1: triton-ufds (UFDS/LDAP Client)

**Why first?** Authentication is foundational - needed by other services.

**What to implement:**
- LDAP client for UFDS (User, Forensics, Directory Services)
- User authentication and management
- Group management
- DN (Distinguished Name) handling

**Reference:**
- `/Users/nwilkens/workspace/triton-admin/backend/src/auth/ldap.rs`
- Use `UfdsCredentials` from triton-core

**Key types:**
- `UfdsClient` - Main client
- `User` - User representation
- `Group` - Group representation
- Authentication methods

### Priority 2: triton-sapi (Service API Client)

**Why second?** Required for service discovery - needed by other clients.

**What to implement:**
- Service API client for discovering Triton services
- Application and instance management
- Endpoint discovery

**Reference:**
- `/Users/nwilkens/workspace/triton-admin/backend/src/services/sapi_client.rs`
- `/Users/nwilkens/workspace/triton-admin/backend/src/services/discovery_manager.rs`

**Key types:**
- `SapiClient` - Main client
- `Application` - SAPI application
- `Instance` - Service instance
- `DiscoveryManager` - Implements `ServiceDiscovery` trait from triton-core

### Priority 3: triton-vmapi (Virtual Machine API)

**What to implement:**
- VM lifecycle management (create, start, stop, delete)
- VM listing and filtering
- VM metadata and tags
- NIC management

**Reference:**
- `/Users/nwilkens/workspace/triton-admin/backend/src/models/vmapi.rs`
- `/Users/nwilkens/workspace/triton-admin/backend/src/api/vms.rs`

**Key types:**
- `VmapiClient` - Main client
- `Vm` - Virtual machine representation
- `VmState` enum - VM states (Running, Stopped, etc.)
- `VmBrand` enum - VM types (Joyent, KVM, Bhyve, LX)
- `Nic` - Network interface

### Priority 4: triton-cnapi (Compute Node API)

**What to implement:**
- Server (compute node) management
- Server capacity and utilization
- Server tasks and jobs
- Hardware information

**Reference:**
- `/Users/nwilkens/workspace/triton-admin/backend/src/models/cnapi.rs`

**Key types:**
- `CnapiClient` - Main client
- `Server` - Compute node representation
- Server status and health

### Priority 5: triton-napi (Network API)

**What to implement:**
- Network creation and management
- Network pools
- NIC tags
- IP address management

**Reference:**
- `/Users/nwilkens/workspace/triton-admin/backend/src/models/napi.rs`

**Key types:**
- `NapiClient` - Main client
- `Network` - Network representation
- `NicTag` - NIC tag representation

### Priority 6: Other Services

- **triton-papi** - Package API (VM sizing/packages)
- **triton-imgapi** - Image API (OS images)
- **triton-fwapi** - Firewall API (firewall rules)
- **triton-amon** - Monitoring API
- **triton-workflow** - Workflow API

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
**Next Priority:** Implement triton-ufds for authentication

## Notes for AI Agents

- **Be proactive with tests** - Write tests as you write code
- **Use atomic commits** - Commit frequently with clear messages
- **Run cargo audit** - After every significant change
- **Keep modules small** - Break large files into logical modules
- **Check for reusability** - Look for patterns to extract to triton-core
- **Document as you go** - Write doc comments for all public APIs
- **Follow existing patterns** - Look at triton-core for examples

Good luck! The foundation is solid. Time to build the service clients! 🚀
