//! PAPI models shared by client and prospective server implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use triton_core::query::QueryParams;
use triton_core::uuid::{NetworkUuid, OwnerUuid, PackageUuid};

/// Representation of a package as returned by PAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Package {
    /// Package UUID.
    pub uuid: PackageUuid,
    /// Package name.
    pub name: String,
    /// Package version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Maximum physical memory (MiB).
    pub max_physical_memory: u64,
    /// Disk quota (GiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,
    /// CPU cap value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_cap: Option<u32>,
    /// CPU shares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<u32>,
    /// Maximum swap (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_swap: Option<u64>,
    /// Maximum LWPs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_lwps: Option<u32>,
    /// ZFS IO priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zfs_io_priority: Option<u32>,
    /// vCPU count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<u32>,
    /// Memory (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    /// Operating system type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Brand (joyent/kvm/bhyve/etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// Disk size (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<u64>,
    /// Network definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<PackageNetwork>>,
    /// Package group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Whether the package is active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether the package is the default choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// RAM ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram_ratio: Option<f64>,
    /// CPU burst ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_burst_ratio: Option<f64>,
    /// CPU burst duty cycle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_burst_duty_cycle: Option<f64>,
    /// IO priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_priority: Option<u32>,
    /// IO throttle value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_throttle: Option<i32>,
    /// Billing tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_tags: Option<Vec<String>>,
    /// Arbitrary tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
    /// Common name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub common_name: Option<String>,
    /// Owner UUIDs with access.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuids: Option<Vec<OwnerUuid>>,
    /// Trait flags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traits: Option<HashMap<String, bool>>,
}

/// Package network definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageNetwork {
    /// Network name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Gateway address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    /// IP address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// Netmask.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netmask: Option<String>,
    /// Network UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_uuid: Option<NetworkUuid>,
    /// Whether this NIC should be primary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    /// Subnet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
    /// VLAN ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<u16>,
    /// NIC tag name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_tag: Option<String>,
    /// Whether the NIC is physical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical: Option<bool>,
    /// CIDR representation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cidr: Option<String>,
}

/// Request payload for creating a package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreatePackageRequest {
    /// Package name.
    pub name: String,
    /// Optional version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Maximum physical memory (MiB).
    pub max_physical_memory: u64,
    /// Disk quota (GiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,
    /// CPU cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_cap: Option<u32>,
    /// CPU shares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<u32>,
    /// Maximum swap (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_swap: Option<u64>,
    /// Maximum LWPs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_lwps: Option<u32>,
    /// ZFS IO priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zfs_io_priority: Option<u32>,
    /// vCPU count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<u32>,
    /// Memory (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    /// Operating system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Brand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// Disk size (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<u64>,
    /// Network definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<PackageNetwork>>,
    /// Group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Active flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Default flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// RAM ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram_ratio: Option<f64>,
    /// CPU burst ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_burst_ratio: Option<f64>,
    /// CPU burst duty cycle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_burst_duty_cycle: Option<f64>,
    /// IO priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_priority: Option<u32>,
    /// IO throttle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_throttle: Option<i32>,
    /// Billing tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_tags: Option<Vec<String>>,
    /// Arbitrary tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
    /// Common name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub common_name: Option<String>,
    /// Owner UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuids: Option<Vec<OwnerUuid>>,
    /// Traits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traits: Option<HashMap<String, bool>>,
}

/// Request payload for updating a package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UpdatePackageRequest {
    /// Package name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Package version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Maximum physical memory (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_physical_memory: Option<u64>,
    /// Disk quota (GiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,
    /// CPU cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_cap: Option<u32>,
    /// CPU shares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<u32>,
    /// Maximum swap (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_swap: Option<u64>,
    /// Maximum LWPs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_lwps: Option<u32>,
    /// ZFS IO priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zfs_io_priority: Option<u32>,
    /// vCPU count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<u32>,
    /// Memory (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    /// Operating system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Brand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// Disk size (MiB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<u64>,
    /// Network definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<PackageNetwork>>,
    /// Group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Active flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Default flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// RAM ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram_ratio: Option<f64>,
    /// CPU burst ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_burst_ratio: Option<f64>,
    /// CPU burst duty cycle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_burst_duty_cycle: Option<f64>,
    /// IO priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_priority: Option<u32>,
    /// IO throttle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_throttle: Option<i32>,
    /// Billing tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_tags: Option<Vec<String>>,
    /// Arbitrary tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
    /// Owner UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuids: Option<Vec<OwnerUuid>>,
    /// Traits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traits: Option<HashMap<String, bool>>,
}

/// Query parameters for listing packages.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PackageListParams {
    /// Filter by name.
    pub name: Option<String>,
    /// Filter by version.
    pub version: Option<String>,
    /// Filter by memory requirement.
    pub memory: Option<u64>,
    /// Filter by vCPU count.
    pub vcpus: Option<u32>,
    /// Filter by brand.
    pub brand: Option<String>,
    /// Filter by OS.
    pub os: Option<String>,
    /// Filter by group.
    pub group: Option<String>,
    /// Active flag.
    pub active: Option<bool>,
    /// Default flag.
    pub default: Option<bool>,
    /// Owner UUID filter.
    pub owner_uuid: Option<OwnerUuid>,
    /// Trait identifier filter.
    pub trait_id: Option<String>,
    /// Trait value filter.
    pub trait_val: Option<bool>,
    /// Limit results.
    pub limit: Option<u32>,
    /// Offset.
    pub offset: Option<u32>,
}

impl PackageListParams {
    /// Convert the parameters into URL query pairs.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = QueryParams::new();

        params.push_opt("name", self.name.as_deref());
        params.push_opt("version", self.version.as_deref());
        params.push_opt("memory", self.memory);
        params.push_opt("vcpus", self.vcpus);
        params.push_opt("brand", self.brand.as_deref());
        params.push_opt("os", self.os.as_deref());
        params.push_opt("group", self.group.as_deref());
        params.push_opt("active", self.active);
        params.push_opt("default", self.default);
        params.push_opt("owner_uuid", self.owner_uuid.as_ref());
        params.push_opt("trait", self.trait_id.as_deref());
        params.push_opt("trait_val", self.trait_val);
        params.push_opt("limit", self.limit);
        params.push_opt("offset", self.offset);

        params.into_pairs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_list_params_to_pairs() {
        let params = PackageListParams {
            name: Some("standard".into()),
            memory: Some(4096),
            active: Some(true),
            limit: Some(50),
            ..PackageListParams::default()
        };

        let pairs = params.to_pairs();
        assert!(pairs.contains(&("name", "standard".into())));
        assert!(pairs.contains(&("memory", "4096".into())));
        assert!(pairs.contains(&("active", "true".into())));
        assert!(pairs.contains(&("limit", "50".into())));
    }
}
