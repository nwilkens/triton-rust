# triton-papi

Typed models and async client utilities for Triton DataCenter's Package API (PAPI). This crate mirrors the legacy Node.js implementation while offering a Rust-native surface built on the shared `triton-core` utilities.

## Highlights

- Strongly typed `Package` models with serde support for tags, traits, and network definitions.
- `PapiClient` built on the shared `ServiceClient`, providing configurable retries plus optional basic/X-Auth token authentication.
- Fluent helpers for listing, retrieving, creating, updating, and deleting packages.
- `PapiDiscovery` wrapper that plugs into the workspace-wide `ServiceDiscovery` trait via the reusable proxy.
- Wiremock-backed tests covering happy paths, error mapping, and discovery delegation.

## Example

```rust
use triton_core::uuid::PackageUuid;
use triton_papi::{PapiClient, PackageListParams};

#[tokio::main]
async fn main() -> triton_papi::Result<()> {
    let client = PapiClient::new("https://papi.example.com")?;

    let packages = client.list_packages(&PackageListParams::default()).await?;
    for package in packages {
        println!("Package {} ({})", package.uuid, package.name);
    }

    let pkg = client.get_package(PackageUuid::new_v4()).await?;
    println!("Fetched package {}", pkg.name);

    Ok(())
}
```

## Testing

```bash
cargo test --package triton-papi --offline
```

## License

Dual licensed under MIT or Apache-2.0, consistent with the triton-rust workspace.
