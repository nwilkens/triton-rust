# Triton Rust Integration Roadmap

This document captures the current state of the shared Rust crates living in `~/workspace/triton-rust` and explains how the admin backend should adopt them. Hand it to the next agent before you dive into the codebase so we maintain continuity.

---

## What’s available today

We now have production-ready client crates for every Triton backend service the admin portal talks to:

| Crate | Path | Purpose | Highlights |
|-------|------|---------|------------|
| `triton-core` | `~/workspace/triton-rust/crates/triton-core` | Shared plumbing | Error type, UUID wrappers, `ServiceClient` HTTP layer, discovery proxy, query builder. |
| `triton-imgapi` | `~/workspace/triton-rust/crates/triton-imgapi` | Image service | Models + client for CRUD, snapshot files, import/export. |
| `triton-papi` | `~/workspace/triton-rust/crates/triton-papi` | Package service | Models + client for package lifecycle. |
| `triton-fwapi` | `~/workspace/triton-rust/crates/triton-fwapi` | Firewall service | Models + client for rule lifecycle. |
| `triton-sapi` | `~/workspace/triton-rust/crates/triton-sapi` | Services API | Discovery-ready client exposing applications, services, instances, and endpoint lookup. |

All clients share the same retry-aware HTTP layer and SAPI discovery proxy, so once admin consumes them we can delete the bespoke Rust clients under `backend/src/services/*`.

---

## Migration game plan for `triton-admin`

We should migrate one slice at a time and keep the service online:

1. **Introduce the crates as dependencies**
   - Add path-based dependencies in `backend/Cargo.toml` pointing at the corresponding crates in `~/workspace/triton-rust/crates`.
   - Run `cargo check -p backend` to verify nothing explodes before changing any code.

2. **Replace the custom SAPI client**
   - Swap `backend/src/services/sapi_client.rs` and the discovery manager to use `triton_sapi::{SapiClient, SapiDiscovery}`.
   - Map existing configuration (`TritonConfig`) onto `triton_core::config::TritonClientConfig`.
   - Remove any duplicate retry/discovery logic once everything compiles.

3. **Adopt IMGAPI / PAPI / FWAPI crates**
   - For each API module (`api/imgapi`, `api/papi`, `api/fwapi`), replace the local `*Client` wrappers with the crate equivalents.
   - Ensure request/response models line up; we may need adapter functions where the REST surface differs.
   - Drop obsolete code in `services/client_factory.rs` after each migration.

4. **Clean up and document**
   - Delete unused structs/tests once a module is fully migrated.
  - Update the admin README / internal docs to explain that the shared crates are now the source of truth.

Take a service at a time and keep the diff manageable—each migration should leave the tests passing and the server runnable (`cargo test --all`, `cargo run -p backend`).

---

## Quick reference for the new crates

```text
~/workspace/triton-rust/
├── crates/
│   ├── triton-core/
│   ├── triton-imgapi/
│   ├── triton-papi/
│   ├── triton-fwapi/
│   └── triton-sapi/
└── services/ (reserved for future Rust service ports)
```

Each crate has:

- `README.md` with a usage snippet.
- Unit tests using `wiremock`.
- Models that use `triton-core` UUID wrappers and serde defaults.
- Clients built on the shared `ServiceClient` for consistent retry/auth handling.

---

## Tips for the next agent

- **Reuse config:** `TritonClientConfig` in `triton-core` maps cleanly from the existing `TritonConfig`—don’t reinvent the wheel.
- **Leverage discovery proxy:** The new crates expect SAPI-backed discovery via the proxy; once SAPI is wired in, the other clients can consume it directly.
- **Validate behaviour:** After swapping a service, hit the corresponding admin routes locally (or run the integration test suite) to confirm we honour the old semantics.
- **Coordinate updates:** If you find missing helpers while porting, add them to `triton-core` so the rest of the ecosystem benefits.

Good luck! Once admin is fully migrated we’ll be ready to pull the service binaries themselves into `~/workspace/triton-rust/services/`.
