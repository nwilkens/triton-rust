# triton-fwapi

Typed models and async client utilities for Triton DataCenter's Firewall API (FWAPI). The crate mirrors the existing Node.js implementation and exposes a Rust-native interface backed by the shared `triton-core` primitives.

## Highlights

- Strongly typed `FirewallRule` models with optional metadata, UUID wrappers, and serde support.
- `FwapiClient` built on `ServiceClient`, covering list/get/create/update/delete flows with retry-aware HTTP requests.
- `FirewallRuleListParams` builder leveraging `QueryParams` for flexible filtering.
- `FwapiDiscovery` adapter using the shared `ServiceDiscoveryProxy` to resolve endpoints via SAPI.
- Wiremock-backed tests covering common operations and discovery delegation.

## Example

```rust
use triton_core::uuid::FirewallRuleUuid;
use triton_fwapi::{FirewallRuleListParams, FwapiClient};

#[tokio::main]
async fn main() -> triton_fwapi::Result<()> {
    let client = FwapiClient::new("https://fwapi.example.com")?;

    let rules = client.list_rules(&FirewallRuleListParams::default()).await?;
    println!("Found {} firewall rules", rules.len());

    if let Some(rule) = rules.first() {
        let fetched = client.get_rule(rule.uuid).await?;
        println!("Fetched rule {} -> {}", fetched.uuid, fetched.rule);
    }

    client.delete_rule(FirewallRuleUuid::new_v4()).await?;
    Ok(())
}
```

## Testing

```bash
cargo test --package triton-fwapi --offline
```

## License

Dual licensed under MIT or Apache-2.0, consistent with the triton-rust workspace.
