//! FWAPI models shared by client and prospective server implementations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use triton_core::query::QueryParams;
use triton_core::uuid::{FirewallRuleUuid, OwnerUuid};

/// Representation of a firewall rule as returned by FWAPI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FirewallRule {
    /// Rule UUID.
    pub uuid: FirewallRuleUuid,
    /// Rule string (FWAPI DSL).
    pub rule: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Rule version string.
    pub version: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<OwnerUuid>,
    /// Whether the rule is global.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
    /// Associated VM UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vms: Option<Vec<OwnerUuid>>,
    /// Creator identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Creation timestamp.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "chrono::serde::ts_seconds_option"
    )]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "chrono::serde::ts_seconds_option"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request payload for creating a firewall rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateFirewallRuleRequest {
    /// Rule string.
    pub rule: String,
    /// Enabled flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<OwnerUuid>,
    /// Global flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
    /// VM UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vms: Option<Vec<OwnerUuid>>,
}

/// Request payload for updating a firewall rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UpdateFirewallRuleRequest {
    /// Rule string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// Enabled flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<OwnerUuid>,
    /// Global flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
    /// VM UUIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vms: Option<Vec<OwnerUuid>>,
}

/// Query parameters for listing firewall rules.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FirewallRuleListParams {
    /// Filter by owner UUID.
    pub owner_uuid: Option<OwnerUuid>,
    /// Filter global rules.
    pub global: Option<bool>,
    /// Filter enabled rules.
    pub enabled: Option<bool>,
    /// Filter by VM UUID.
    pub vm: Option<OwnerUuid>,
    /// Limit.
    pub limit: Option<u32>,
    /// Offset.
    pub offset: Option<u32>,
}

impl FirewallRuleListParams {
    /// Convert to URL query pairs.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = QueryParams::new();
        params.push_opt("owner_uuid", self.owner_uuid.as_ref());
        params.push_opt("global", self.global);
        params.push_opt("enabled", self.enabled);
        params.push_opt("vm", self.vm.as_ref());
        params.push_opt("limit", self.limit);
        params.push_opt("offset", self.offset);
        params.into_pairs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firewall_rule_list_params_to_pairs() {
        let params = FirewallRuleListParams {
            owner_uuid: Some(OwnerUuid::new_v4()),
            global: Some(true),
            limit: Some(25),
            ..FirewallRuleListParams::default()
        };

        let pairs = params.to_pairs();
        assert!(pairs.iter().any(|(k, _)| *k == "owner_uuid"));
        assert!(pairs.contains(&("global", "true".into())));
        assert!(pairs.contains(&("limit", "25".into())));
    }
}
