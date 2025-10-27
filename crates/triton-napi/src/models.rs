//! NAPI data models for networks, pools, and NICs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use triton_core::uuid::{NetworkUuid, OwnerUuid};

/// Query parameters supported by `/networks`.
#[derive(Debug, Default, Clone)]
pub struct NetworkListParams {
    /// Filter by human-friendly network name.
    pub name: Option<String>,
    /// Filter by network UUID.
    pub uuid: Option<NetworkUuid>,
    /// Filter by VLAN identifier.
    pub vlan_id: Option<u16>,
    /// Filter by owning account UUID.
    pub owner_uuid: Option<OwnerUuid>,
    /// Filter by UFDS user allowed to provision.
    pub provisionable_by: Option<String>,
    /// Limit to fabric networks.
    pub fabric: Option<bool>,
    /// Maximum number of results (1-1000).
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

impl NetworkListParams {
    /// Convert the params into URL query pairs.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();

        if let Some(name) = &self.name {
            pairs.push(("name", name.clone()));
        }
        if let Some(uuid) = &self.uuid {
            pairs.push(("uuid", uuid.to_string()));
        }
        if let Some(vlan_id) = self.vlan_id {
            pairs.push(("vlan_id", vlan_id.to_string()));
        }
        if let Some(owner_uuid) = &self.owner_uuid {
            pairs.push(("owner_uuid", owner_uuid.to_string()));
        }
        if let Some(provisionable_by) = &self.provisionable_by {
            pairs.push(("provisionable_by", provisionable_by.clone()));
        }
        if let Some(fabric) = self.fabric {
            pairs.push(("fabric", fabric.to_string()));
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

/// Network representation returned by NAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Network {
    /// Network UUID.
    pub uuid: NetworkUuid,
    /// Network name.
    pub name: String,
    /// VLAN identifier.
    pub vlan_id: u16,
    /// Subnet in CIDR notation.
    pub subnet: String,
    /// Netmask string.
    pub netmask: String,
    /// Default gateway.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    /// Start of the provisionable IP range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provision_start_ip: Option<String>,
    /// End of the provisionable IP range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provision_end_ip: Option<String>,
    /// NIC tag associated with the network.
    pub nic_tag: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner UUIDs that can manage the network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuids: Option<Vec<OwnerUuid>>,
    /// Static routes keyed by destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routes: Option<HashMap<String, String>>,
    /// Resolver IP addresses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolvers: Option<Vec<String>>,
    /// Indicates whether the network is a fabric network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric: Option<bool>,
    /// Internet NAT flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internet_nat: Option<bool>,
    /// MTU value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    /// Address family (ipv4/ipv6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// VNET identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vnet_id: Option<u32>,
    /// Gateway provisioning flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_provisioned: Option<bool>,
}

/// Request payload to create a network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateNetworkRequest {
    /// Network name.
    pub name: String,
    /// VLAN identifier.
    pub vlan_id: u16,
    /// Subnet in CIDR notation.
    pub subnet: String,
    /// Netmask string.
    pub netmask: String,
    /// Default gateway.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    /// Provision start IP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provision_start_ip: Option<String>,
    /// Provision end IP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provision_end_ip: Option<String>,
    /// NIC tag.
    pub nic_tag: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuids: Option<Vec<OwnerUuid>>,
    /// Static routes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routes: Option<HashMap<String, String>>,
    /// DNS resolvers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolvers: Option<Vec<String>>,
    /// Fabric network flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric: Option<bool>,
    /// Internet NAT flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internet_nat: Option<bool>,
    /// MTU value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
}

/// Request payload to update a network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateNetworkRequest {
    /// New network name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Updated provision start IP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provision_start_ip: Option<String>,
    /// Updated provision end IP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provision_end_ip: Option<String>,
    /// Updated resolvers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolvers: Option<Vec<String>>,
    /// Updated static routes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routes: Option<HashMap<String, String>>,
    /// Updated owner UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuids: Option<Vec<OwnerUuid>>,
}

/// Network pool representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPool {
    /// Pool UUID.
    pub uuid: String,
    /// Pool name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Networks in the pool.
    pub networks: Vec<NetworkUuid>,
    /// Primary NIC tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_tag: Option<String>,
    /// Present NIC tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_tags_present: Option<Vec<String>>,
    /// Address family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Owner UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuids: Option<Vec<OwnerUuid>>,
}

/// NIC metadata returned by NAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Nic {
    /// MAC address.
    pub mac: String,
    /// Primary flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    /// Owning account UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<OwnerUuid>,
    /// UUID of the resource this NIC belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belongs_to_uuid: Option<String>,
    /// Type of resource (vm, server, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub belongs_to_type: Option<String>,
    /// Primary IPv4 address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// Netmask.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netmask: Option<String>,
    /// VLAN identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<u16>,
    /// NIC tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_tag: Option<String>,
    /// Backing network UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_uuid: Option<NetworkUuid>,
    /// Current state string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Provided NIC tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nic_tags_provided: Option<Vec<String>>,
    /// Gateway address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    /// Resolver list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolvers: Option<Vec<String>>,
    /// Creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_timestamp: Option<DateTime<Utc>>,
    /// Last modified timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_timestamp: Option<DateTime<Utc>>,
    /// Whether DHCP spoofing is allowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_dhcp_spoofing: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_list_params_pairs() {
        let params = NetworkListParams {
            name: Some("admin".into()),
            vlan_id: Some(100),
            limit: Some(20),
            ..NetworkListParams::default()
        };

        let pairs = params.to_pairs();
        assert!(pairs.iter().any(|(k, v)| *k == "name" && v == "admin"));
        assert!(pairs.iter().any(|(k, v)| *k == "vlan_id" && v == "100"));
        assert!(pairs.iter().any(|(k, v)| *k == "limit" && v == "20"));
    }
}
