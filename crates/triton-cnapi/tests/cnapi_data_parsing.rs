//! Integration tests for parsing CNAPI data.
//!
//! These tests validate that the triton-cnapi models can correctly deserialize
//! actual CNAPI response data.

use std::fs;
use std::path::PathBuf;
use triton_cnapi::models::Server;

/// Get the path to the test fixtures directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Load the server list fixture from disk.
fn load_server_list_fixture() -> String {
    let fixture_path = fixtures_dir().join("server_list.json");
    fs::read_to_string(&fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read server list fixture at {}: {}",
            fixture_path.display(),
            e
        )
    })
}

/// Load the server detail fixture from disk.
fn load_server_detail_fixture() -> String {
    let fixture_path = fixtures_dir().join("server_detail.json");
    fs::read_to_string(&fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read server detail fixture at {}: {}",
            fixture_path.display(),
            e
        )
    })
}

#[test]
fn test_deserialize_server_list() {
    let json_data = load_server_list_fixture();

    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap_or_else(|e| {
        panic!(
            "Failed to deserialize server list data: {}\nJSON: {}",
            e, json_data
        )
    });

    // Validate we got the expected number of servers
    assert_eq!(servers.len(), 81, "Expected 81 servers in test data");
}

#[test]
fn test_deserialize_server_detail() {
    let json_data = load_server_detail_fixture();

    let server: Server = serde_json::from_str(&json_data).unwrap_or_else(|e| {
        panic!(
            "Failed to deserialize server detail data: {}\nJSON: {}",
            e, json_data
        )
    });

    // Validate key fields
    assert_eq!(
        server.uuid.to_string(),
        "df04a288-c841-4b48-9112-4c38d51e0edd"
    );
    assert_eq!(server.hostname.as_deref(), Some("b16"));
    assert_eq!(server.datacenter.as_deref(), Some("dc-test-1"));
}

#[test]
fn test_all_servers_have_required_fields() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    for server in &servers {
        // Every server should have a UUID (required field)
        assert!(
            !server.uuid.to_string().is_empty(),
            "Server should have a valid UUID"
        );

        // Validate all servers have essential operational fields
        assert!(
            server.datacenter.is_some(),
            "Server {} should have a datacenter",
            server.uuid
        );
        assert!(
            server.status.is_some(),
            "Server {} should have a status",
            server.uuid
        );

        // Validate setup-related fields
        assert!(
            server.setup.is_some(),
            "Server {} should have a setup flag",
            server.uuid
        );

        // Validate basic resource fields
        assert!(
            server.ram.is_some(),
            "Server {} should have RAM allocation",
            server.uuid
        );
    }
}

#[test]
fn test_server_detail_complete_fields() {
    let json_data = load_server_detail_fixture();
    let server: Server = serde_json::from_str(&json_data).unwrap();

    // Validate basic server info
    assert_eq!(server.hostname.as_deref(), Some("b16"));
    assert_eq!(server.datacenter.as_deref(), Some("dc-test-1"));
    assert_eq!(server.status.as_deref(), Some("running"));
    assert_eq!(server.setup, Some(true));
    assert_eq!(server.setting_up, Some(false));
    assert_eq!(server.headnode, Some(false));
    assert_eq!(server.reserved, Some(false));
    assert_eq!(server.reservoir, Some(false));

    // Validate platform info
    assert_eq!(
        server.boot_platform.as_deref(),
        Some("20241212T000748Z")
    );
    assert_eq!(
        server.current_platform.as_deref(),
        Some("20241112T202951Z")
    );

    // Validate timestamps
    assert!(server.created.is_some(), "Should have created timestamp");
    assert!(
        server.last_boot.is_some(),
        "Should have last_boot timestamp"
    );
    assert!(
        server.last_heartbeat.is_some(),
        "Should have last_heartbeat timestamp"
    );

    // Validate reservation and overprovision ratios
    assert_eq!(server.reservation_ratio, Some(0.15));
    assert!(
        server.overprovision_ratios.is_some(),
        "Should have overprovision_ratios"
    );
    if let Some(ratios) = &server.overprovision_ratios {
        assert_eq!(ratios.get("cpu"), Some(&4.0));
        assert_eq!(ratios.get("ram"), Some(&1.0));
        assert_eq!(ratios.get("disk"), Some(&1.0));
    }

    // Validate memory fields
    assert_eq!(server.ram, Some(391857));
    assert!(
        server.memory_total_bytes.is_some(),
        "Should have memory_total_bytes"
    );
    assert!(
        server.memory_available_bytes.is_some(),
        "Should have memory_available_bytes"
    );
    assert!(
        server.memory_arc_bytes.is_some(),
        "Should have memory_arc_bytes"
    );
    assert!(
        server.memory_provisionable_bytes.is_some(),
        "Should have memory_provisionable_bytes"
    );

    // Validate disk fields
    assert!(
        server.disk_pool_size_bytes.is_some(),
        "Should have disk_pool_size_bytes"
    );
    assert!(
        server.disk_installed_images_used_bytes.is_some(),
        "Should have disk_installed_images_used_bytes"
    );
    assert!(
        server.disk_zone_quota_bytes.is_some(),
        "Should have disk_zone_quota_bytes"
    );
    assert!(
        server.disk_kvm_quota_bytes.is_some(),
        "Should have disk_kvm_quota_bytes"
    );
    assert!(
        server.disk_cores_quota_bytes.is_some(),
        "Should have disk_cores_quota_bytes"
    );

    // Validate capacity fields
    assert_eq!(server.unreserved_cpu, Some(2250));
    assert_eq!(server.unreserved_ram, Some(53135));
    assert_eq!(server.unreserved_disk, Some(1876292));

    // Validate console and boot params
    assert_eq!(server.default_console.as_deref(), Some("serial"));
    assert_eq!(server.serial.as_deref(), Some("ttyb"));
    assert!(server.boot_params.is_some(), "Should have boot_params");
    assert!(server.kernel_flags.is_some(), "Should have kernel_flags");
}

#[test]
fn test_server_sysinfo_parsing() {
    let json_data = load_server_detail_fixture();
    let server: Server = serde_json::from_str(&json_data).unwrap();

    // Validate sysinfo is present and parseable
    assert!(server.sysinfo.is_some(), "Should have sysinfo");

    if let Some(sysinfo) = &server.sysinfo {
        // Validate key sysinfo fields
        assert_eq!(
            sysinfo.get("Hostname").and_then(|v| v.as_str()),
            Some("b16")
        );
        assert_eq!(
            sysinfo.get("UUID").and_then(|v| v.as_str()),
            Some("df04a288-c841-4b48-9112-4c38d51e0edd")
        );
        assert_eq!(
            sysinfo.get("System Type").and_then(|v| v.as_str()),
            Some("SunOS")
        );
        assert_eq!(
            sysinfo.get("Datacenter Name").and_then(|v| v.as_str()),
            Some("dc-test-1")
        );
        assert_eq!(
            sysinfo.get("Manufacturer").and_then(|v| v.as_str()),
            Some("Generic Manufacturer")
        );
        assert_eq!(
            sysinfo.get("Setup").and_then(|v| v.as_str()),
            Some("true")
        );

        // Validate CPU info
        assert_eq!(
            sysinfo
                .get("CPU Type")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("Intel")),
            Some(true)
        );
        assert_eq!(
            sysinfo.get("CPU Count").and_then(|v| v.as_u64()),
            Some(40)
        );
        assert_eq!(
            sysinfo.get("CPU Core Count").and_then(|v| v.as_u64()),
            Some(20)
        );

        // Validate memory info
        assert_eq!(
            sysinfo.get("MiB of Memory").and_then(|v| v.as_str()),
            Some("391857")
        );

        // Validate storage info
        assert_eq!(
            sysinfo.get("Zpool").and_then(|v| v.as_str()),
            Some("zones")
        );
        assert_eq!(
            sysinfo.get("Zpool Encrypted").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            sysinfo.get("Zpool Profile").and_then(|v| v.as_str()),
            Some("raidz")
        );

        // Validate VM capabilities
        assert_eq!(
            sysinfo.get("VM Capable").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            sysinfo.get("Bhyve Capable").and_then(|v| v.as_bool()),
            Some(true)
        );

        // Validate network interfaces are present
        assert!(sysinfo.get("Network Interfaces").is_some());

        // Validate boot parameters are present
        assert!(sysinfo.get("Boot Parameters").is_some());

        // Validate SDC agents are present
        assert!(sysinfo.get("SDC Agents").is_some());
    }
}

#[test]
fn test_server_vms_parsing() {
    let json_data = load_server_detail_fixture();
    let server: Server = serde_json::from_str(&json_data).unwrap();

    // Validate VMs are present
    assert!(server.vms.is_some(), "Should have VMs");

    if let Some(vms) = &server.vms {
        // Should have 7 VMs based on the test data
        assert_eq!(vms.len(), 7, "Should have 7 VMs");

        // Check that VM entries have expected fields
        for (vm_uuid, vm_data) in vms {
            assert!(!vm_uuid.to_string().is_empty(), "VM should have UUID");

            // Validate VM data has essential fields
            assert!(
                vm_data.get("uuid").is_some(),
                "VM should have uuid field"
            );
            assert!(
                vm_data.get("owner_uuid").is_some(),
                "VM should have owner_uuid field"
            );
            assert!(
                vm_data.get("state").is_some(),
                "VM should have state field"
            );
            assert!(
                vm_data.get("zone_state").is_some(),
                "VM should have zone_state field"
            );
            assert!(
                vm_data.get("brand").is_some(),
                "VM should have brand field"
            );
        }

        // Test specific VM brands are present
        let has_bhyve = vms.values().any(|vm| {
            vm.get("brand")
                .and_then(|v| v.as_str())
                .map(|b| b == "bhyve")
                .unwrap_or(false)
        });
        let has_joyent = vms.values().any(|vm| {
            vm.get("brand")
                .and_then(|v| v.as_str())
                .map(|b| b.contains("joyent"))
                .unwrap_or(false)
        });

        assert!(has_bhyve, "Should have bhyve brand VMs");
        assert!(has_joyent, "Should have joyent brand VMs");
    }
}

#[test]
fn test_server_traits_parsing() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    // Find servers with different traits
    let has_m5d_trait = servers.iter().any(|s| {
        s.traits
            .as_ref()
            .and_then(|t| t.get("m5d"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });

    let has_manta_trait = servers.iter().any(|s| {
        s.traits
            .as_ref()
            .and_then(|t| t.get("manta"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });

    let has_storage_trait = servers.iter().any(|s| {
        s.traits
            .as_ref()
            .and_then(|t| t.get("storage"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });

    let has_headnode_trait = servers.iter().any(|s| {
        s.traits
            .as_ref()
            .and_then(|t| t.get("headnode"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });

    let has_development_trait = servers.iter().any(|s| {
        s.traits
            .as_ref()
            .and_then(|t| t.get("development"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });

    let has_amd_trait = servers.iter().any(|s| {
        s.traits
            .as_ref()
            .and_then(|t| t.get("amd"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });

    assert!(has_m5d_trait, "Should have servers with m5d trait");
    assert!(has_manta_trait, "Should have servers with manta trait");
    assert!(
        has_storage_trait,
        "Should have servers with storage trait"
    );
    assert!(
        has_headnode_trait,
        "Should have servers with headnode trait"
    );
    assert!(
        has_development_trait,
        "Should have servers with development trait"
    );
    assert!(has_amd_trait, "Should have servers with amd trait");
}

#[test]
fn test_server_boot_params_parsing() {
    let json_data = load_server_detail_fixture();
    let server: Server = serde_json::from_str(&json_data).unwrap();

    assert!(server.boot_params.is_some(), "Should have boot_params");

    if let Some(boot_params) = &server.boot_params {
        // Validate boot params contain expected fields
        assert!(
            boot_params.get("rabbitmq").is_some(),
            "Should have rabbitmq boot param"
        );
        assert!(
            boot_params.get("smt_enabled").is_some(),
            "Should have smt_enabled boot param"
        );

        // Validate specific values
        assert_eq!(
            boot_params
                .get("rabbitmq")
                .and_then(|v| v.as_str()),
            Some("guest:guest:rabbitmq.dc-test-1.example.local:5672")
        );
        assert_eq!(
            boot_params.get("smt_enabled").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}

#[test]
fn test_server_status_variations() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    let has_running = servers
        .iter()
        .any(|s| s.status.as_deref() == Some("running"));
    let has_unknown = servers
        .iter()
        .any(|s| s.status.as_deref() == Some("unknown"));

    assert!(has_running, "Should have servers with running status");
    assert!(has_unknown, "Should have servers with unknown status");
}

#[test]
fn test_server_headnode_flag() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    // Find the actual headnode (has headnode = true)
    let headnode = servers
        .iter()
        .find(|s| s.headnode == Some(true))
        .expect("Should have a headnode");

    assert_eq!(headnode.hostname.as_deref(), Some("headnode"));
    assert_eq!(headnode.setup, Some(true));
}

#[test]
fn test_server_reservation_flags() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    let has_reserved = servers.iter().any(|s| s.reserved == Some(true));
    let has_unreserved = servers.iter().any(|s| s.reserved == Some(false));

    assert!(has_reserved, "Should have reserved servers");
    assert!(has_unreserved, "Should have unreserved servers");
}

#[test]
fn test_server_setup_states() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    // Find a server that is currently being set up
    let has_setting_up = servers.iter().any(|s| s.setting_up == Some(true));

    assert!(
        has_setting_up,
        "Should have servers currently being setup"
    );

    // All servers in this dataset should have setup = true
    assert!(
        servers.iter().all(|s| s.setup == Some(true)),
        "All servers should have setup = true"
    );
}

#[test]
fn test_server_rack_identifier() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    // Find servers with rack identifiers
    let has_rack_id = servers
        .iter()
        .any(|s| s.rack_identifier.as_ref().map(|r| !r.is_empty()).unwrap_or(false));

    // At least one server has a rack identifier in the test data
    assert!(has_rack_id, "Should have servers with rack identifiers");
}

#[test]
fn test_server_comments_field() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    // Comments field should be present (even if empty)
    for server in &servers {
        assert!(
            server.comments.is_some(),
            "Server {} should have comments field",
            server.uuid
        );
    }
}

#[test]
fn test_server_roundtrip_serialization() {
    let json_data = load_server_detail_fixture();
    let original_server: Server = serde_json::from_str(&json_data).unwrap();

    // Serialize and deserialize to ensure roundtrip works
    let serialized = serde_json::to_string(&original_server)
        .expect("Should be able to serialize server");

    let deserialized: Server = serde_json::from_str(&serialized)
        .expect("Should be able to deserialize serialized server");

    // Key fields should match
    assert_eq!(original_server.uuid, deserialized.uuid);
    assert_eq!(original_server.hostname, deserialized.hostname);
    assert_eq!(original_server.datacenter, deserialized.datacenter);
    assert_eq!(original_server.status, deserialized.status);
    assert_eq!(original_server.ram, deserialized.ram);
    assert_eq!(original_server.setup, deserialized.setup);
    assert_eq!(original_server.headnode, deserialized.headnode);
    assert_eq!(original_server.reserved, deserialized.reserved);
}

#[test]
fn test_numeric_fields_preserved() {
    let json_data = load_server_detail_fixture();
    let server: Server = serde_json::from_str(&json_data).unwrap();

    // Test that numeric values are preserved correctly
    assert_eq!(server.ram, Some(391857));
    assert_eq!(server.reservation_ratio, Some(0.15));
    assert_eq!(server.memory_total_bytes, Some(410883235840));
    assert_eq!(server.memory_available_bytes, Some(16426561536));
    assert_eq!(server.memory_arc_bytes, Some(54474435176));
    assert_eq!(server.memory_provisionable_bytes, Some(55723897651));
    assert_eq!(server.disk_pool_size_bytes, Some(5565035053056));
    assert_eq!(server.unreserved_cpu, Some(2250));
    assert_eq!(server.unreserved_ram, Some(53135));
    assert_eq!(server.unreserved_disk, Some(1876292));
}

#[test]
fn test_timestamp_parsing() {
    let json_data = load_server_detail_fixture();
    let server: Server = serde_json::from_str(&json_data).unwrap();

    // Validate timestamps are parsed correctly
    assert!(server.created.is_some(), "Should have created timestamp");
    assert!(server.last_boot.is_some(), "Should have last_boot timestamp");
    assert!(
        server.last_heartbeat.is_some(),
        "Should have last_heartbeat timestamp"
    );

    // Timestamps should be valid and parseable
    if let Some(created) = &server.created {
        assert!(created.timestamp() > 0, "Created timestamp should be valid");
    }
}

#[test]
fn test_server_with_different_ram_sizes() {
    let json_data = load_server_list_fixture();
    let servers: Vec<Server> = serde_json::from_str(&json_data).unwrap();

    // Collect different RAM sizes
    let mut ram_sizes: Vec<u64> = servers
        .iter()
        .filter_map(|s| s.ram)
        .collect();
    ram_sizes.sort();
    ram_sizes.dedup();

    // Should have servers with different RAM configurations
    assert!(
        ram_sizes.len() > 1,
        "Should have servers with different RAM sizes"
    );

    // Test data includes servers with ~98GB, ~130GB, and ~391GB
    assert!(
        ram_sizes.iter().any(|&r| r > 300000),
        "Should have high-memory servers"
    );
    assert!(
        ram_sizes.iter().any(|&r| r < 150000),
        "Should have lower-memory servers"
    );
}
