# triton-ufds

UFDS (User, Forensics, and Directory Services) client utilities for Triton DataCenter. The crate provides strongly typed helpers and an asynchronous client for integrating with Triton's LDAP directory.

## Features

- Distinguished name parsing and manipulation (`DistinguishedName`).
- Rich user and group domain models (`User`, `Group`, `AccountStatus`).
- Configurable UFDS client with timeout and TLS options (`UfdsConfig`, `UfdsClient`).
- LDAP abstraction layer for testing with mocked sessions.
- Comprehensive unit tests covering critical behaviour.

## Quick Start

```rust
use triton_ufds::{DistinguishedName, UfdsClient, UfdsConfig};
use triton_core::services::UfdsCredentials;
use triton_core::uuid::AppUuid;

#[tokio::main]
async fn main() -> triton_ufds::Result<()> {
    let credentials = UfdsCredentials::new(
        "cn=admin,dc=example,dc=com".to_string(),
        "secret".to_string(),
        AppUuid::new_v4(),
    );

    let config = UfdsConfig::new(
        "ldaps://ufds.example.com",
        credentials,
        DistinguishedName::parse("dc=example,dc=com")?,
    )?;

    let client = UfdsClient::new(config);
    let user = client.authenticate("jdoe", "password").await?;
    println!(
        "Authenticated {}",
        user
            .display_name()
            .unwrap_or_else(|| user.login.clone()),
    );
    Ok(())
}
```

## Development

Run the crate's unit tests:

```bash
cargo test --package triton-ufds --offline
```

## License

Dual licensed under MIT or Apache-2.0, consistent with the Triton Rust workspace.
