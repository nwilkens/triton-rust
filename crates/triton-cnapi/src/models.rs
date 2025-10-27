//! CNAPI data models shared by clients and (eventual) servers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use triton_core::uuid::{ServerUuid, VmUuid};

/// Query parameters supported by CNAPI's `/servers` endpoint.
#[derive(Debug, Default, Clone)]
pub struct ServerListParams {
    /// Filter by datacenter name.
    pub datacenter: Option<String>,
    /// Filter by hostname (exact match).
    pub hostname: Option<String>,
    /// Filter by a single server UUID.
    pub uuid: Option<ServerUuid>,
    /// Filter by multiple UUIDs.
    pub uuids: Option<Vec<ServerUuid>>,
    /// Include only setup servers.
    pub setup: Option<bool>,
    /// Include only reserved servers.
    pub reserved: Option<bool>,
    /// Include only headnodes.
    pub headnode: Option<bool>,
    /// Include only reservoir servers.
    pub reservoir: Option<bool>,
    /// Comma-separated extras (e.g. `agents,vms,sysinfo,capacity,all`).
    pub extras: Option<String>,
    /// Comma-separated list of explicit fields to return.
    pub fields: Option<String>,
    /// Maximum number of results (1-1000).
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

impl ServerListParams {
    /// Convert the parameter struct into URL query pairs.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();

        if let Some(datacenter) = &self.datacenter {
            pairs.push(("datacenter", datacenter.clone()));
        }
        if let Some(hostname) = &self.hostname {
            pairs.push(("hostname", hostname.clone()));
        }
        if let Some(uuid) = &self.uuid {
            pairs.push(("uuid", uuid.to_string()));
        }
        if let Some(uuids) = &self.uuids {
            let combined = uuids
                .iter()
                .map(ServerUuid::to_string)
                .collect::<Vec<_>>()
                .join(",");
            pairs.push(("uuids", combined));
        }
        if let Some(setup) = self.setup {
            pairs.push(("setup", setup.to_string()));
        }
        if let Some(reserved) = self.reserved {
            pairs.push(("reserved", reserved.to_string()));
        }
        if let Some(headnode) = self.headnode {
            pairs.push(("headnode", headnode.to_string()));
        }
        if let Some(reservoir) = self.reservoir {
            pairs.push(("reservoir", reservoir.to_string()));
        }
        if let Some(extras) = &self.extras {
            pairs.push(("extras", extras.clone()));
        }
        if let Some(fields) = &self.fields {
            pairs.push(("fields", fields.clone()));
        }
        if let Some(limit) = self.limit {
            pairs.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            pairs.push(("offset", offset.to_string()));
        }

        pairs
    }
}

/// Representation of a CN server returned by CNAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Server {
    /// Server UUID.
    pub uuid: ServerUuid,
    /// Hostname when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// Current datacenter label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datacenter: Option<String>,
    /// Current operational status (running, unknown, rebooting).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Whether the server has completed setup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup: Option<bool>,
    /// Whether the server is currently being setup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_up: Option<bool>,
    /// Whether this server is a headnode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headnode: Option<bool>,
    /// Whether the server is reserved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserved: Option<bool>,
    /// Whether the server participates in the reservoir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservoir: Option<bool>,
    /// Boot platform information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_platform: Option<String>,
    /// Current platform (dataset) identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_platform: Option<String>,
    /// Human comments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    /// Last heartbeat timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Last boot timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_boot: Option<DateTime<Utc>>,
    /// Transitional status (e.g., rebooting).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transitional_status: Option<String>,
    /// Rack identifier string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rack_identifier: Option<String>,
    /// Datacenter display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datacenter_name: Option<String>,

    /// Total RAM in MiB as reported by sysinfo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram: Option<u64>,
    /// Raw sysinfo payload forwarded from the agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysinfo: Option<serde_json::Value>,

    /// Available memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_available_bytes: Option<u64>,
    /// ARC memory consumption in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_arc_bytes: Option<u64>,
    /// Total physical memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_total_bytes: Option<u64>,
    /// Provisionable memory in bytes (accounts for reservations).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_provisionable_bytes: Option<i64>,

    /// Reservation ratio currently applied to this server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation_ratio: Option<f64>,
    /// Global overprovision ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overprovision_ratio: Option<f64>,
    /// Detailed overprovision ratios per resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overprovision_ratios: Option<HashMap<String, f64>>,

    /// Allocated pool size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_pool_size_bytes: Option<u64>,
    /// Bytes consumed by installed images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_installed_images_used_bytes: Option<u64>,
    /// Zone quota allocation in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_zone_quota_bytes: Option<u64>,
    /// Total KVM quota in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_kvm_quota_bytes: Option<u64>,
    /// Bytes used by KVM zvols.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_kvm_zvol_used_bytes: Option<u64>,
    /// Logical size of KVM zvols in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_kvm_zvol_volsize_bytes: Option<u64>,
    /// Quota for core dumps in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_cores_quota_bytes: Option<u64>,

    /// Unreserved CPU capacity (cores).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unreserved_cpu: Option<i64>,
    /// Unreserved RAM (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unreserved_ram: Option<i64>,
    /// Unreserved disk (GiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unreserved_disk: Option<i64>,

    /// Attached NICs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nics: Option<Vec<ServerNic>>,

    /// Trait map applied to the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traits: Option<HashMap<String, serde_json::Value>>,

    /// Boot parameters blob.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_params: Option<serde_json::Value>,
    /// Kernel flag map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_flags: Option<serde_json::Value>,
    /// Default console device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_console: Option<String>,
    /// Serial console identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,

    /// Optional VM metadata keyed by VM UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vms: Option<HashMap<VmUuid, serde_json::Value>>,
}

/// Compute node capacity summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ServerCapacity {
    /// Unreserved CPU (cores).
    #[serde(default)]
    pub unreserved_cpu: i64,
    /// Unreserved RAM in MiB.
    #[serde(default)]
    pub unreserved_ram: i64,
    /// Unreserved disk in GiB.
    #[serde(default)]
    pub unreserved_disk: i64,
}

impl From<&Server> for ServerCapacity {
    fn from(server: &Server) -> Self {
        Self {
            unreserved_cpu: server.unreserved_cpu.unwrap_or_default(),
            unreserved_ram: server.unreserved_ram.unwrap_or_default(),
            unreserved_disk: server.unreserved_disk.unwrap_or_default(),
        }
    }
}

/// Update server request payload for CNAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateServerRequest {
    /// Update the reserved flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserved: Option<bool>,
    /// Update the reservation ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation_ratio: Option<f64>,
    /// Update the global overprovision ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overprovision_ratio: Option<f64>,
    /// Free-form comments for operators.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Trait overrides (true/false per trait name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traits: Option<HashMap<String, bool>>,
}

/// Representation of a server NIC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerNic {
    /// Logical interface name.
    pub interface: String,
    /// MAC address.
    pub mac: String,
    /// IPv4 address if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip4addr: Option<String>,
    /// Netmask string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netmask: Option<String>,
    /// Supplied NIC tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_tags_provided: Option<Vec<String>>,
    /// Link status (up/down).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn server_list_params_to_pairs() {
        let params = ServerListParams {
            datacenter: Some("us-east-1".into()),
            uuid: Some(ServerUuid::new_v4()),
            setup: Some(true),
            limit: Some(10),
            ..ServerListParams::default()
        };

        let pairs = params.to_pairs();
        assert!(pairs.iter().any(|(k, _)| *k == "datacenter"));
        assert!(pairs.iter().any(|(k, _)| *k == "uuid"));
        assert!(pairs.iter().any(|(k, v)| *k == "setup" && v == "true"));
        assert!(pairs.iter().any(|(k, v)| *k == "limit" && v == "10"));
    }

    #[test]
    fn server_capacity_from_server() {
        let mut server: Server = serde_json::from_value(json!({
            "uuid": ServerUuid::new_v4(),
            "unreserved_cpu": 8,
            "unreserved_ram": 16384,
            "unreserved_disk": 1024
        }))
        .unwrap();
        server.hostname = Some("cn01".into());

        let capacity = ServerCapacity::from(&server);
        assert_eq!(capacity.unreserved_cpu, 8);
        assert_eq!(capacity.unreserved_ram, 16384);
        assert_eq!(capacity.unreserved_disk, 1024);
    }
}
