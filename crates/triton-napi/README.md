# triton-napi

Rust client utilities for the Triton Network API (NAPI). The crate delivers typed network models alongside an asynchronous client with retry support, mirroring the behaviour of the legacy Node.js implementation.

## Features

- `Network`, `NetworkPool`, and `Nic` models with serde support and strong UUID typing.
- `NapiClient` helpers for listing, creating, updating, and deleting networks, network pools, and NICs.
- Query builders (`NetworkQuery`) with ergonomic conversions to query parameters.
- `NapiDiscovery` bridge that reuses SAPI-based service discovery for endpoint lookups.
- Wiremock-backed tests covering success and error scenarios.

## Example

```rust
use triton_napi::{NapiClient, NetworkQuery};
use triton_core::uuid::OwnerUuid;

#[tokio::main]
async fn main() -> triton_napi::Result<()> {
    let client = NapiClient::new("https://napi.example.com")?;

    let query = NetworkQuery {
        owner_uuid: Some(OwnerUuid::new_v4()),
        limit: Some(50),
        ..NetworkQuery::default()
    };

    let networks = client.list_networks(&query).await?;
    for network in networks {
        println!("{} -> {}", network.uuid, network.name);
    }

    Ok(())
}
```

## Testing

```bash
cargo test --package triton-napi --offline
```

## License

Dual licensed under MIT or Apache-2.0, consistent with the triton-rust workspace.
