//! Core SAPI domain models.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use triton_core::uuid::{AppUuid, InstanceUuid, JobUuid, OwnerUuid, ServiceUuid};

/// Represents a SAPI application definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Application {
    /// Application UUID.
    pub uuid: AppUuid,
    /// Application name.
    pub name: String,
    /// Owning account UUID.
    pub owner_uuid: OwnerUuid,
    /// Arbitrary application parameters.
    #[serde(
        default,
        deserialize_with = "deserialize_map",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub params: BTreeMap<String, serde_json::Value>,
    /// Application metadata.
    #[serde(
        default,
        deserialize_with = "deserialize_map",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_json::Value>,
    /// Manifest definitions.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub manifests: serde_json::Value,
    /// Timestamp when the application was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Timestamp when the application was updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Optional master definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master: Option<serde_json::Value>,
    /// Optional application description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Enumeration of known instance types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceType {
    /// VM instance that runs on a CN.
    Vm,
    /// Non-VM agent instance.
    Agent,
    /// Other/unknown instance type.
    #[serde(other)]
    Other,
}

impl Default for InstanceType {
    fn default() -> Self {
        Self::Vm
    }
}

impl InstanceType {
    /// Returns the instance type as a lowercase string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Vm => "vm",
            Self::Agent => "agent",
            Self::Other => "other",
        }
    }
}

/// Represents a SAPI service definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    /// Service UUID.
    pub uuid: ServiceUuid,
    /// Human-readable service name.
    pub name: String,
    /// Owning application UUID.
    pub application_uuid: AppUuid,
    /// Arbitrary service parameters.
    #[serde(
        default,
        deserialize_with = "deserialize_map",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub params: BTreeMap<String, serde_json::Value>,
    /// Service metadata.
    #[serde(
        default,
        deserialize_with = "deserialize_map",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_json::Value>,
    /// Manifest data.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub manifests: serde_json::Value,
    /// Optional master data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master: Option<serde_json::Value>,
    /// Service type (vm, agent, etc.).
    #[serde(default)]
    pub r#type: Option<InstanceType>,
    /// Optional creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Optional update timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Represents a SAPI service instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Instance {
    /// Instance UUID.
    pub uuid: InstanceUuid,
    /// Parent service UUID.
    pub service_uuid: ServiceUuid,
    /// Parent application UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_uuid: Option<AppUuid>,
    /// Instance name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Service name for this instance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    /// Published version of the service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Hostname or IP address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// Physical server hosting the instance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_uuid: Option<triton_core::uuid::ServerUuid>,
    /// Instance parameters.
    #[serde(
        default,
        deserialize_with = "deserialize_map",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub params: BTreeMap<String, serde_json::Value>,
    /// Instance metadata.
    #[serde(
        default,
        deserialize_with = "deserialize_map",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_json::Value>,
    /// Instance manifests.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub manifests: serde_json::Value,
    /// Indicates whether this instance is the master.
    #[serde(default)]
    pub master: bool,
    /// Optional instance state (running, stopped, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Associated provisioning job UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_uuid: Option<JobUuid>,
    /// Instance type.
    #[serde(default)]
    pub r#type: Option<InstanceType>,
    /// Optional creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Optional update timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

fn deserialize_map<'de, D>(deserializer: D) -> Result<BTreeMap<String, serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let option = Option::<BTreeMap<String, serde_json::Value>>::deserialize(deserializer)?;
    Ok(option.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn application_roundtrip() {
        let app = Application {
            uuid: AppUuid::new_v4(),
            name: "example".into(),
            owner_uuid: OwnerUuid::new_v4(),
            params: BTreeMap::new(),
            metadata: BTreeMap::new(),
            manifests: serde_json::Value::Null,
            created_at: Some("2024-01-01T00:00:00Z".into()),
            updated_at: None,
            master: None,
            description: Some("Example application".into()),
        };

        let json = serde_json::to_string(&app).unwrap();
        let value: Application = serde_json::from_str(&json).unwrap();
        assert_eq!(value.name, "example");
    }

    #[test]
    fn instance_serialization_defaults() {
        let instance = Instance {
            uuid: InstanceUuid::new_v4(),
            service_uuid: ServiceUuid::new_v4(),
            application_uuid: None,
            name: Some("vmapi0".into()),
            service_name: Some("vmapi".into()),
            version: Some("1.0.0".into()),
            hostname: Some("vmapi.local".into()),
            server_uuid: None,
            params: BTreeMap::new(),
            metadata: BTreeMap::new(),
            manifests: json!({}),
            master: false,
            state: Some("running".into()),
            job_uuid: None,
            r#type: Some(InstanceType::Vm),
            created_at: None,
            updated_at: None,
        };

        let json = serde_json::to_string(&instance).unwrap();
        let decoded: Instance = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.r#type, Some(InstanceType::Vm));
    }

    #[test]
    fn service_serialization() {
        let mut metadata = BTreeMap::new();
        metadata.insert("role".into(), json!("api"));

        let service = Service {
            uuid: ServiceUuid::new_v4(),
            name: "sapi".into(),
            application_uuid: AppUuid::new_v4(),
            params: BTreeMap::new(),
            metadata,
            manifests: serde_json::Value::Null,
            master: None,
            r#type: Some(InstanceType::Vm),
            created_at: None,
            updated_at: None,
        };

        let json = serde_json::to_string(&service).unwrap();
        let decoded: Service = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "sapi");
        assert_eq!(decoded.r#type, Some(InstanceType::Vm));
        assert_eq!(
            decoded.metadata.get("role").and_then(|v| v.as_str()),
            Some("api")
        );
    }
}
