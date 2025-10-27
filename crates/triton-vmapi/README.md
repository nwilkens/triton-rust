# triton-vmapi

Typed models and async client utilities for Triton DataCenter's VMAPI. The crate mirrors the legacy Node.js VMAPI behaviour with strongly typed Rust structures, retry-aware HTTP helpers, and optional SAPI-backed discovery.

## Highlights

- `Vm`/`Nic`/`VmapiJob` models with serde support for the many shapes returned by VMAPI.
- `VmapiClient` featuring configurable retries, basic/X-Auth token authentication, and helpers for VM lifecycle, snapshots, and batch operations.
- Fluent builders (`VmQuery`, `JobListParams`) for list endpoints.
- `VmapiDiscovery` wrapper so consumers can plug VMAPI discovery into the shared `ServiceDiscovery` trait.
- Wiremock-based tests covering happy paths and common failure scenarios.

## Example

```rust
use triton_vmapi::{VmapiClient, VmQuery};
use triton_core::uuid::OwnerUuid;

#[tokio::main]
async fn main() -> triton_vmapi::Result<()> {
    let client = VmapiClient::new("https://vmapi.example.com")?;

    let query = VmQuery {
        owner_uuid: Some(OwnerUuid::new_v4()),
        limit: Some(20),
        ..VmQuery::default()
    };

    let vms = client.list_vms(&query).await?;
    for vm in vms {
        println!("{} {:?}", vm.uuid, vm.state);
    }

    Ok(())
}
```

## Testing

```bash
cargo test --package triton-vmapi --offline
```

## License

Dual licensed under MIT or Apache-2.0, consistent with the triton-rust workspace.
