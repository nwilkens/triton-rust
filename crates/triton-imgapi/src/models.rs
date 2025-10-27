//! IMGAPI models shared by client and prospective server implementations.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use triton_core::query::QueryParams;
use triton_core::uuid::{ImageUuid, OwnerUuid};

/// Deserialize a map whose values may be strings, booleans, or numbers into string values.
pub fn deserialize_string_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<HashMap<String, Value>> = Option::deserialize(deserializer)?;
    Ok(value.map(|map| {
        map.into_iter()
            .filter_map(|(key, value)| value_to_string(value).map(|v| (key, v)))
            .collect()
    }))
}

/// Deserialize a map whose values may be booleans or boolean-like strings into booleans.
pub fn deserialize_bool_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, bool>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<HashMap<String, Value>> = Option::deserialize(deserializer)?;
    Ok(value.map(|map| {
        map.into_iter()
            .filter_map(|(key, value)| value_to_bool(value).map(|v| (key, v)))
            .collect()
    }))
}

fn value_to_string(value: Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Null => None,
        Value::Array(_) | Value::Object(_) => None,
    }
}

fn value_to_bool(value: Value) -> Option<bool> {
    match value {
        Value::Bool(b) => Some(b),
        Value::String(s) => match s.to_lowercase().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Parameters supported by the `/images` list endpoint.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ImageListParams {
    /// Filter by name.
    pub name: Option<String>,
    /// Filter by version.
    pub version: Option<String>,
    /// Filter by operating system.
    pub os: Option<String>,
    /// Filter by owner UUID.
    pub owner: Option<OwnerUuid>,
    /// Account visibility scope.
    pub account: Option<OwnerUuid>,
    /// Filter by image state.
    pub state: Option<String>,
    /// Filter by public flag.
    pub public: Option<bool>,
    /// Filter by type.
    pub type_filter: Option<String>,
    /// Filter by tag key/value.
    pub tag: Option<String>,
    /// Filter by billing tag.
    pub billing_tag: Option<String>,
    /// Filter by trait name.
    pub trait_filter: Option<String>,
    /// Filter by channel.
    pub channel: Option<String>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Pagination marker.
    pub marker: Option<String>,
    /// Only return latest version.
    pub latest_only: Option<bool>,
    /// Sort field.
    pub sort_by: Option<String>,
    /// Sort order.
    pub sort_order: Option<String>,
}

impl ImageListParams {
    /// Convert the parameters into URL query pairs.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = QueryParams::new();
        params.push_opt("name", self.name.as_deref());
        params.push_opt("version", self.version.as_deref());
        params.push_opt("os", self.os.as_deref());
        params.push_opt("owner", self.owner.as_ref());
        params.push_opt("account", self.account.as_ref());
        params.push_opt("state", self.state.as_deref());
        params.push_opt("public", self.public);
        params.push_opt("type", self.type_filter.as_deref());
        params.push_opt("tag", self.tag.as_deref());
        params.push_opt("billing_tag", self.billing_tag.as_deref());
        params.push_opt("trait", self.trait_filter.as_deref());
        params.push_opt("channel", self.channel.as_deref());
        params.push_opt("limit", self.limit);
        params.push_opt("offset", self.offset);
        params.push_opt("marker", self.marker.as_deref());
        params.push_opt("latest_only", self.latest_only);
        params.push_opt("sort_by", self.sort_by.as_deref());
        params.push_opt("sort_order", self.sort_order.as_deref());

        params.into_pairs()
    }
}

/// Representation of an image as returned by IMGAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    /// Image UUID.
    pub uuid: ImageUuid,
    /// Image name.
    pub name: String,
    /// Image version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Operating system.
    pub os: String,
    /// Image type.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Current state.
    pub state: String,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Homepage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Published timestamp (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// Owner UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<OwnerUuid>,
    /// Public flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Access control list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acl: Option<Vec<OwnerUuid>>,
    /// Key-value tags.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_string_map"
    )]
    pub tags: Option<HashMap<String, String>>,
    /// Disabled flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Origin image UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<ImageUuid>,
    /// Error payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ImageError>,
    /// File descriptors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<ImageFile>>,
    /// Icon path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<ImageRequirements>,
    /// Provisioned users.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<ImageUser>>,
    /// Billing tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_tags: Option<Vec<String>>,
    /// Traits.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_bool_map"
    )]
    pub traits: Option<HashMap<String, bool>>,
    /// Channels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    /// Expiration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Inherited directories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherited_directories: Option<Vec<String>>,
    /// Generate passwords flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_passwords: Option<bool>,
    /// NIC driver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_driver: Option<String>,
    /// Disk driver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_driver: Option<String>,
    /// CPU type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_type: Option<String>,
    /// Image size (bytes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_size: Option<u64>,
    /// Virtual size (bytes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_size: Option<u64>,
    /// Minimum memory (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_memory: Option<u64>,
    /// Minimum disk (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_disk: Option<u64>,
    /// Minimum platform requirements.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_string_map"
    )]
    pub min_platform: Option<HashMap<String, String>>,
    /// Created timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Updated timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

/// Error details embedded within image responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageError {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
}

/// Metadata describing an individual image file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageFile {
    /// Compression type (gzip, bzip2, etc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
    /// SHA1 hash.
    pub sha1: String,
    /// File size in bytes.
    pub size: u64,
    /// Storage back-end identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    /// MD5 hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
    /// Optional download path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Requirements enforcing how the image must be provisioned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageRequirements {
    /// Network requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<ImageNetworkRequirement>>,
    /// SSH key requirement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<bool>,
    /// Password requirement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<bool>,
    /// Maximum physical memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_physical_memory: Option<u64>,
    /// Minimum platform requirements.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_string_map"
    )]
    pub min_platform: Option<HashMap<String, String>>,
}

/// Network requirement for an image.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageNetworkRequirement {
    /// Network name.
    pub name: String,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// IP address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// Primary flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

/// Provisioned system user included with the image.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUser {
    /// Username.
    pub name: String,
    /// UID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    /// GID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
    /// Shell path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    /// Home directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_dir: Option<String>,
}

/// Request payload for creating a new image record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateImageRequest {
    /// Name for the image.
    pub name: String,
    /// Version identifier.
    pub version: String,
    /// Operating system.
    pub os: String,
    /// Image type.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Homepage URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Public flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Owner UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<OwnerUuid>,
    /// Key-value tags.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_string_map"
    )]
    pub tags: Option<HashMap<String, String>>,
    /// Origin UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<ImageUuid>,
    /// Files metadata (manifest).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<ImageFile>>,
    /// Requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<ImageRequirements>,
    /// Users.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<ImageUser>>,
    /// Billing tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_tags: Option<Vec<String>>,
    /// Traits.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_bool_map"
    )]
    pub traits: Option<HashMap<String, bool>>,
    /// Channels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    /// NIC driver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_driver: Option<String>,
    /// Disk driver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_driver: Option<String>,
    /// CPU type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_type: Option<String>,
    /// Image size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_size: Option<u64>,
    /// Virtual size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_size: Option<u64>,
    /// Minimum memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_memory: Option<u64>,
    /// Minimum disk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_disk: Option<u64>,
    /// Generate passwords flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_passwords: Option<bool>,
    /// Inherited directories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherited_directories: Option<Vec<String>>,
}

/// Request payload for updating an image.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UpdateImageRequest {
    /// Name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Homepage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Public flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// State.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Disabled flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Tags.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_string_map"
    )]
    pub tags: Option<HashMap<String, String>>,
    /// Billing tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_tags: Option<Vec<String>>,
    /// Requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<ImageRequirements>,
    /// Traits.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_bool_map"
    )]
    pub traits: Option<HashMap<String, bool>>,
    /// Channels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    /// Generate passwords flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_passwords: Option<bool>,
    /// Inherited directories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherited_directories: Option<Vec<String>>,
}

/// Request payload for importing an image file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageImportRequest {
    /// Desired image UUID.
    pub uuid: ImageUuid,
    /// Compression algorithm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
    /// SHA1 checksum.
    pub sha1: String,
    /// Storage backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    /// File path.
    pub file_path: String,
    /// File size.
    pub size: u64,
    /// Optional source descriptor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ImportImageSource>,
    /// Optional MD5 checksum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
}

/// Describes the origin of an imported image.
pub type ImportImageSource = String;

/// Request payload for exporting an image to Manta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportImageRequest {
    /// Destination Manta path.
    pub manta_path: String,
    /// Optional storage backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
}

/// Supported actions for image lifecycle endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAction {
    /// Activate an image.
    Activate,
    /// Disable an image.
    Disable,
    /// Enable an image.
    Enable,
}

impl ImageAction {
    /// Return the path segment for this action.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Activate => "activate",
            Self::Disable => "disable",
            Self::Enable => "enable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use triton_core::uuid::OwnerUuid;

    #[test]
    fn image_list_params_to_pairs_includes_values() {
        let params = ImageListParams {
            name: Some("ubuntu".into()),
            version: Some("22.04".into()),
            owner: Some(OwnerUuid::new_v4()),
            limit: Some(10),
            latest_only: Some(true),
            ..ImageListParams::default()
        };

        let pairs = params.to_pairs();
        assert!(pairs.contains(&("name", "ubuntu".into())));
        assert!(pairs.iter().any(|(k, _)| *k == "owner"));
        assert!(pairs.contains(&("limit", "10".into())));
        assert!(pairs.contains(&("latest_only", "true".into())));
    }

    #[test]
    fn deserialize_string_map_handles_booleans_and_numbers() {
        let value = json!({
            "tags": {
                "alpha": true,
                "beta": 42,
                "gamma": "delta"
            }
        });

        #[derive(Debug, Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "deserialize_string_map")]
            tags: Option<HashMap<String, String>>,
        }

        let wrapper: Wrapper = serde_json::from_value(value).unwrap();
        let tags = wrapper.tags.unwrap();
        assert_eq!(tags.get("alpha").unwrap(), "true");
        assert_eq!(tags.get("beta").unwrap(), "42");
        assert_eq!(tags.get("gamma").unwrap(), "delta");
    }

    #[test]
    fn deserialize_bool_map_handles_string_values() {
        let value = json!({
            "traits": {
                "virtio": "true",
                "uefi": false,
                "other": "invalid"
            }
        });

        #[derive(Debug, Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "deserialize_bool_map")]
            traits: Option<HashMap<String, bool>>,
        }

        let wrapper: Wrapper = serde_json::from_value(value).unwrap();
        let traits = wrapper.traits.unwrap();
        assert_eq!(traits.get("virtio"), Some(&true));
        assert_eq!(traits.get("uefi"), Some(&false));
        assert!(!traits.contains_key("other"));
    }
}
