//! VMAPI models shared by client and prospective server implementations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use triton_core::query::QueryParams;
use triton_core::uuid::{ImageUuid, InstanceUuid, NetworkUuid, OwnerUuid, PackageUuid, ServerUuid};

/// Parameters supported by the `/vms` list endpoint.
#[derive(Debug, Default, Clone)]
pub struct VMListParams {
    /// Filter by owning account UUID.
    pub owner_uuid: Option<OwnerUuid>,
    /// Filter by VM state (running, stopped, etc.).
    pub state: Option<String>,
    /// Filter by alias (hostname).
    pub alias: Option<String>,
    /// Filter by compute node UUID.
    pub server_uuid: Option<ServerUuid>,
    /// Filter by image UUID.
    pub image_uuid: Option<ImageUuid>,
    /// Filter by VM brand (joyent, kvm, bhyve, etc.).
    pub brand: Option<String>,
    /// Maximum number of results (1-1000).
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
    /// Comma-separated list of fields to return.
    pub fields: Option<String>,
}

impl VMListParams {
    /// Convert the parameters into URL query pairs.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = QueryParams::new();
        params.push_opt("owner_uuid", self.owner_uuid.as_ref());
        params.push_opt("state", self.state.as_deref());
        params.push_opt("alias", self.alias.as_deref());
        params.push_opt("server_uuid", self.server_uuid.as_ref());
        params.push_opt("image_uuid", self.image_uuid.as_ref());
        params.push_opt("brand", self.brand.as_deref());
        params.push_opt("limit", self.limit);
        params.push_opt("offset", self.offset);
        params.push_opt("fields", self.fields.as_deref());

        params.into_pairs()
    }
}

/// Representation of a VM as returned by VMAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vm {
    /// VM UUID.
    pub uuid: InstanceUuid,
    /// VM alias (hostname-like identifier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// VM brand (joyent, kvm, bhyve, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// Current VM state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Provisioned RAM in MiB (serde_json::Value to preserve floats/ints).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram: Option<serde_json::Value>,
    /// Maximum physical memory in MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_physical_memory: Option<serde_json::Value>,
    /// Disk quota in MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<serde_json::Value>,
    /// CPU shares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<serde_json::Value>,
    /// CPU cap percentage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_cap: Option<serde_json::Value>,
    /// vCPU count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<serde_json::Value>,
    /// Disk size in MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<serde_json::Value>,
    /// Maximum swap in MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_swap: Option<serde_json::Value>,
    /// Maximum locked memory in MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_locked_memory: Option<serde_json::Value>,
    /// Maximum LWPs allowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_lwps: Option<serde_json::Value>,

    /// Zone state string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_state: Option<String>,
    /// Zone path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zonepath: Option<String>,
    /// Zpool name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zpool: Option<String>,
    /// ZFS dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zfs_filesystem: Option<String>,

    /// Hosting server UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_uuid: Option<ServerUuid>,
    /// Image UUID used for provisioning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_uuid: Option<ImageUuid>,
    /// Package name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    /// Package UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_uuid: Option<PackageUuid>,
    /// Owning account UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<OwnerUuid>,

    /// Customer metadata (arbitrary JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customer_metadata: Option<serde_json::Value>,
    /// Internal metadata (arbitrary JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_metadata: Option<serde_json::Value>,

    /// Firewall enabled flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firewall_enabled: Option<bool>,
    /// Autoboot flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autoboot: Option<bool>,
    /// Docker brand indicator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker: Option<bool>,

    /// Network interfaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nics: Option<Vec<Nic>>,
    /// Disk description objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disks: Option<Vec<serde_json::Value>>,

    /// Snapshot list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<Vec<serde_json::Value>>,

    /// Tag map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,

    /// Creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_timestamp: Option<DateTime<Utc>>,
    /// Last modified timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<DateTime<Utc>>,
    /// Boot timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_timestamp: Option<DateTime<Utc>>,
    /// Destroyed timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destroyed: Option<DateTime<Utc>>,

    /// Hosting compute node name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute_node: Option<String>,
    /// Platform build stamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_buildstamp: Option<String>,
    /// Limit priv string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_priv: Option<String>,

    /// DNS domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_domain: Option<String>,
    /// DNS resolvers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolvers: Option<Vec<String>>,
    /// Filesystem policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs_allowed: Option<String>,
    /// Maintain resolvers toggle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintain_resolvers: Option<bool>,
    /// Additional filesystems.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filesystems: Option<Vec<serde_json::Value>>,
    /// Dataset list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datasets: Option<Vec<String>>,

    /// Primary IP convenience field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_ip: Option<String>,

    /// Deletion protection flags.
    /// Deletion protection toggle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_protection: Option<bool>,
    /// Indicates if the zoneroot is indestructible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indestructible_zoneroot: Option<bool>,
}

/// Network interface representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Nic {
    /// MAC address.
    pub mac: String,
    /// Primary flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    /// NIC tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_tag: Option<String>,
    /// Primary IPv4 address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// List of IPs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ips: Option<Vec<String>>,
    /// Netmask.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netmask: Option<String>,
    /// Default gateway.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    /// Additional gateways.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateways: Option<Vec<String>>,
    /// Backing network UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_uuid: Option<NetworkUuid>,
    /// Operational state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// NIC model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// VLAN identifier (may be int or string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<serde_json::Value>,
    /// MTU (int or string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtu: Option<serde_json::Value>,
    /// Interface name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    /// Allowed IP list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<String>>,
    /// Blocked IP list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_ips: Option<Vec<String>>,
}

/// Request payload for VM creation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateVMRequest {
    /// Optional VM alias.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// VM brand.
    pub brand: String,
    /// Owning account UUID.
    pub owner_uuid: OwnerUuid,
    /// Requested RAM in MiB.
    pub ram: u32,
    /// Optional CPU shares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<u32>,
    /// Optional CPU cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_cap: Option<u32>,
    /// Disk quota in MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<u32>,
    /// vCPU count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<u32>,
    /// Image UUID.
    pub image_uuid: ImageUuid,
    /// Optional server pinning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_uuid: Option<ServerUuid>,
    /// Optional package UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_uuid: Option<PackageUuid>,
    /// Network configuration.
    pub networks: serde_json::Value,
    /// Tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
    /// Customer metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customer_metadata: Option<HashMap<String, String>>,
    /// Internal metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_metadata: Option<HashMap<String, String>>,
    /// Firewall enabled flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firewall_enabled: Option<bool>,
}

/// Minimal network configuration details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkConfig {
    /// Network UUID.
    pub uuid: NetworkUuid,
    /// Primary flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    /// Assigned IP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
}

/// Request payload for updating a VM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateVMRequest {
    /// Alias override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// RAM override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram: Option<u32>,
    /// CPU shares override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<u32>,
    /// CPU cap override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_cap: Option<u32>,
    /// Disk quota override in MiB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<u32>,
    /// Updated tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
    /// Updated customer metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customer_metadata: Option<HashMap<String, String>>,
    /// Updated internal metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_metadata: Option<HashMap<String, String>>,
    /// Firewall enable/disable toggle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firewall_enabled: Option<bool>,
}

/// Representation of a VM snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VmSnapshot {
    /// Snapshot name.
    pub name: String,
    /// Snapshot state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// Update timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request payload for creating a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateSnapshotRequest {
    /// Snapshot name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Response returned from snapshot actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotActionResponse {
    /// Snapshot name.
    pub name: String,
    /// Snapshot state.
    pub state: String,
    /// Optional message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// Associated job UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_uuid: Option<String>,
    /// VM uuid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vm_uuid: Option<String>,
    /// Action type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    /// Action name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Raw response payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<serde_json::Value>,
}

/// Supported batch VM actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BatchVMAction {
    /// Start VMs.
    Start,
    /// Stop VMs.
    Stop,
    /// Reboot VMs.
    Reboot,
    /// Delete VMs.
    Delete,
}

/// Request payload for batch VM actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchVMRequest {
    /// VM UUIDs to operate on.
    pub vm_uuids: Vec<InstanceUuid>,
    /// Maximum concurrent operations.
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
}

const fn default_concurrency() -> usize {
    10
}

/// Result of an individual VM action inside a batch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VMBatchResult {
    /// VM UUID.
    pub vm_uuid: InstanceUuid,
    /// Whether the action succeeded.
    pub success: bool,
    /// Optional error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Summary of batch execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchSummary {
    /// Total VMs targeted.
    pub total: usize,
    /// Number of successful actions.
    pub succeeded: usize,
    /// Number of failed actions.
    pub failed: usize,
}

/// Response payload for batch VM operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchVMResponse {
    /// Summary details.
    pub summary: BatchSummary,
    /// Individual results.
    pub results: Vec<VMBatchResult>,
}

/// VMAPI job representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VmapiJob {
    /// Job UUID.
    pub uuid: String,
    /// Job name.
    pub name: String,
    /// Execution state (running/succeeded/failed/canceled).
    pub execution: String,
    /// Arbitrary parameters payload.
    pub params: serde_json::Value,
    /// Optional execution delay timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exec_after: Option<String>,
    /// Creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Timeout seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    /// Optional chain results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_results: Option<Vec<ChainResult>>,
}

/// Result of an individual job step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChainResult {
    /// Result message.
    pub result: String,
    /// Error message, if any.
    pub error: String,
    /// Start timestamp.
    pub started_at: String,
    /// Finish timestamp.
    pub finished_at: String,
}

/// Parameters for listing jobs.
#[derive(Debug, Default, Clone)]
pub struct JobListParams {
    /// VM UUID filter.
    pub vm_uuid: Option<InstanceUuid>,
    /// Execution filter (running/succeeded/failed).
    pub execution: Option<String>,
    /// Task filter (provision, start, stop, etc.).
    pub task: Option<String>,
    /// Limit.
    pub limit: Option<u32>,
    /// Offset.
    pub offset: Option<u32>,
}

impl JobListParams {
    /// Convert to URL pairs.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = QueryParams::new();
        params.push_opt("vm_uuid", self.vm_uuid.as_ref());
        params.push_opt("execution", self.execution.as_deref());
        params.push_opt("task", self.task.as_deref());
        params.push_opt("limit", self.limit);
        params.push_opt("offset", self.offset);

        params.into_pairs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn vm_list_params_to_pairs() {
        let params = VMListParams {
            owner_uuid: Some(OwnerUuid::new_v4()),
            state: Some("running".into()),
            limit: Some(50),
            ..VMListParams::default()
        };

        let pairs = params.to_pairs();
        assert!(pairs.iter().any(|(k, _)| *k == "owner_uuid"));
        assert!(pairs.iter().any(|(k, v)| *k == "state" && v == "running"));
        assert!(pairs.iter().any(|(k, v)| *k == "limit" && v == "50"));
    }

    #[test]
    fn vm_deserialize_basic() {
        let uuid = InstanceUuid::new_v4();
        let json = json!({
            "uuid": uuid,
            "alias": "vm1",
            "brand": "joyent",
            "state": "running"
        });

        let vm: Vm = serde_json::from_value(json).unwrap();
        assert_eq!(vm.uuid, uuid);
        assert_eq!(vm.alias.as_deref(), Some("vm1"));
        assert_eq!(vm.state.as_deref(), Some("running"));
    }

    #[test]
    fn job_list_params_pairs() {
        let params = JobListParams {
            vm_uuid: Some(InstanceUuid::new_v4()),
            execution: Some("succeeded".into()),
            task: Some("provision".into()),
            limit: Some(5),
            offset: Some(10),
        };

        let pairs = params.to_pairs();
        assert_eq!(pairs.len(), 5);
    }

    #[test]
    fn batch_vm_response_roundtrip() {
        let response = BatchVMResponse {
            summary: BatchSummary {
                total: 2,
                succeeded: 1,
                failed: 1,
            },
            results: vec![VMBatchResult {
                vm_uuid: InstanceUuid::new_v4(),
                success: true,
                error: None,
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        let back: BatchVMResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.summary.total, 2);
        assert_eq!(back.results.len(), 1);
    }
}
