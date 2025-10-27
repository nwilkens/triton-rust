# triton-cnapi

Rust client utilities for the Triton Compute Node API (CNAPI). The crate provides typed data models for compute servers and a retry-aware asynchronous client that integrates with the shared `triton-core` configuration layer.

## Features

- Strongly typed CNAPI models (`Server`, `ServerNic`, `UpdateServerRequest`) with serde support.
- Fluent `ServerQuery`/`ServerListParams` builder for listing and filtering nodes.
- `CnapiClient` with configurable retries, basic authentication, and token support (`X-Auth-Token`).
- Optional `CnapiDiscovery` adapter that delegates endpoint lookup to the existing `ServiceDiscovery` implementation (e.g., `SapiDiscovery`).
- Wiremock-based tests covering happy paths and error handling.

## Example

```rust
use triton_cnapi::{CnapiClient, ServerListParams};

#[tokio::main]
async fn main() -> triton_cnapi::Result<()> {
    let client = CnapiClient::new("https://cnapi.example.com")?;

    let params = ServerListParams {
        setup: Some(true),
        limit: Some(10),
        ..ServerListParams::default()
    };

    for server in client.list_servers(&params).await? {
        println!("{} -> {:?}", server.uuid, server.status);
    }

    Ok(())
}
```

## Testing

```bash
cargo test --package triton-cnapi --offline
```

## License

Dual licensed under MIT or Apache-2.0, consistent with the rest of the workspace.
