# triton-imgapi

Typed models and async client utilities for Triton DataCenter's IMGAPI. This crate mirrors the legacy Node.js IMGAPI behaviour with ergonomic Rust APIs, retry-aware HTTP helpers, and optional SAPI-backed discovery.

## Highlights

- Strongly typed `Image` models with tolerant serde helpers for mixed-value metadata (tags, traits, platform requirements).
- `ImgapiClient` featuring configurable retries, basic/X-Auth token authentication, and helpers for listing, mutating, and activating images.
- Convenience methods for streaming image files and kicking off import/export flows.
- `ImgapiDiscovery` adapter so consumers can plug IMGAPI discovery into the shared `ServiceDiscovery` trait.
- Wiremock-backed tests covering happy-path scenarios, error handling, and discovery delegation.

## Example

```rust
use triton_core::uuid::ImageUuid;
use triton_imgapi::{ImageAction, ImgapiClient, ImageListParams};

#[tokio::main]
async fn main() -> triton_imgapi::Result<()> {
    let client = ImgapiClient::new("https://imgapi.example.com")?;

    let images = client.list_images(&ImageListParams::default()).await?;
    for image in images {
        println!("{} {}", image.uuid, image.state);
    }

    let uuid = ImageUuid::new_v4();
    client.perform_action(uuid, ImageAction::Activate).await?;

    Ok(())
}
```

## Testing

```bash
cargo test --package triton-imgapi --offline
```

## License

Dual licensed under MIT or Apache-2.0, consistent with the triton-rust workspace.
