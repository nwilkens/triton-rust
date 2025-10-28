//! Integration tests for parsing VMAPI data.
//!
//! These tests validate that the triton-vmapi models can correctly deserialize
//! actual VMAPI response data.

use std::fs;
use std::path::PathBuf;
use triton_vmapi::models::Vm;

/// Get the path to the test fixtures directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Load the VM list fixture from disk.
fn load_vm_list_fixture() -> String {
    let fixture_path = fixtures_dir().join("production_vm_list.json");
    fs::read_to_string(&fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read VM list fixture at {}: {}",
            fixture_path.display(),
            e
        )
    })
}

#[test]
fn test_deserialize_vm_list() {
    let json_data = load_vm_list_fixture();

    let vms: Vec<Vm> = serde_json::from_str(&json_data).unwrap_or_else(|e| {
        panic!(
            "Failed to deserialize VM list data: {}\nJSON: {}",
            e, json_data
        )
    });

    // Validate we got the expected number of VMs
    assert_eq!(vms.len(), 2, "Expected 2 VMs in test data");
}

#[test]
fn test_vm_bhyve_brand() {
    let json_data = load_vm_list_fixture();
    let vms: Vec<Vm> = serde_json::from_str(&json_data).unwrap();

    // Find the bhyve VM (first one in the fixture)
    let bhyve_vm = vms
        .iter()
        .find(|vm| vm.brand.as_deref() == Some("bhyve"))
        .expect("Should have a bhyve brand VM");

    // Validate basic fields
    assert_eq!(bhyve_vm.alias.as_deref(), Some("sample-dev-qa-pool"));
    assert_eq!(bhyve_vm.state.as_deref(), Some("running"));
    assert_eq!(bhyve_vm.zone_state.as_deref(), Some("running"));
    assert_eq!(bhyve_vm.autoboot, Some(false));
    assert_eq!(bhyve_vm.firewall_enabled, Some(false));

    // Validate resource allocation
    assert!(bhyve_vm.ram.is_some());
    assert!(bhyve_vm.max_physical_memory.is_some());
    assert!(bhyve_vm.quota.is_some());
    assert!(bhyve_vm.cpu_shares.is_some());
    assert!(bhyve_vm.cpu_cap.is_some());

    // Validate UUIDs
    assert!(bhyve_vm.owner_uuid.is_some());
    assert!(bhyve_vm.server_uuid.is_some());
    assert!(bhyve_vm.image_uuid.is_some());

    // Validate customer metadata exists and contains cloud-init data
    assert!(bhyve_vm.customer_metadata.is_some());
    if let Some(metadata) = &bhyve_vm.customer_metadata {
        assert!(metadata.get("cloud-init:user-data").is_some());
    }

    // Validate network configuration
    assert!(bhyve_vm.nics.is_some());
    if let Some(nics) = &bhyve_vm.nics {
        assert_eq!(nics.len(), 1, "Should have 1 NIC");
        let nic = &nics[0];
        assert_eq!(nic.mac, "02:00:00:1a:2b:3c");
        assert_eq!(nic.interface.as_deref(), Some("net0"));
        assert_eq!(nic.primary, Some(true));
        assert_eq!(nic.nic_tag.as_deref(), Some("internal"));
        assert_eq!(nic.ip.as_deref(), Some("10.0.28.108"));
        assert_eq!(nic.gateway.as_deref(), Some("10.0.28.1"));
        assert_eq!(nic.netmask.as_deref(), Some("255.255.254.0"));
        assert_eq!(nic.model.as_deref(), Some("virtio"));
        assert!(nic.vlan_id.is_some());
        assert!(nic.mtu.is_some());
        assert!(nic.network_uuid.is_some());
    }

    // Validate DNS resolvers
    assert!(bhyve_vm.resolvers.is_some());
    if let Some(resolvers) = &bhyve_vm.resolvers {
        assert_eq!(resolvers.len(), 4);
        assert!(resolvers.contains(&"10.0.28.2".to_string()));
        assert!(resolvers.contains(&"1.1.1.1".to_string()));
    }

    // Validate tags
    assert!(bhyve_vm.tags.is_some());
    if let Some(tags) = &bhyve_vm.tags {
        assert_eq!(tags.get("app").map(String::as_str), Some("sample"));
    }

    // Validate ZFS properties
    assert_eq!(
        bhyve_vm.zfs_filesystem.as_deref(),
        Some("zones/fff1c862-8ce7-4077-8a1e-1492ce9a9eff")
    );
    assert_eq!(bhyve_vm.zpool.as_deref(), Some("zones"));
    assert_eq!(
        bhyve_vm.zonepath.as_deref(),
        Some("/zones/fff1c862-8ce7-4077-8a1e-1492ce9a9eff")
    );

    // Validate timestamp parsing
    assert!(bhyve_vm.create_timestamp.is_some());

    // Validate empty arrays
    assert!(bhyve_vm.datasets.is_some());
    assert!(bhyve_vm.disks.is_some());
    assert!(bhyve_vm.snapshots.is_some());
}

#[test]
fn test_vm_joyent_brand() {
    let json_data = load_vm_list_fixture();
    let vms: Vec<Vm> = serde_json::from_str(&json_data).unwrap();

    // Find the joyent VM (second one in the fixture)
    let joyent_vm = vms
        .iter()
        .find(|vm| vm.brand.as_deref() == Some("joyent"))
        .expect("Should have a joyent brand VM");

    // Validate basic fields
    assert_eq!(joyent_vm.alias.as_deref(), Some("k8s-worker-01"));
    assert_eq!(joyent_vm.state.as_deref(), Some("stopped"));
    assert_eq!(joyent_vm.zone_state.as_deref(), Some("stopped"));
    assert_eq!(joyent_vm.autoboot, Some(true));
    assert_eq!(joyent_vm.firewall_enabled, Some(true));

    // Validate resource allocation
    assert!(joyent_vm.ram.is_some());
    assert!(joyent_vm.max_physical_memory.is_some());
    assert!(joyent_vm.max_swap.is_some());
    assert!(joyent_vm.max_locked_memory.is_some());
    assert!(joyent_vm.max_lwps.is_some());
    assert!(joyent_vm.quota.is_some());
    assert!(joyent_vm.cpu_shares.is_some());
    assert!(joyent_vm.cpu_cap.is_some());

    // Validate datasets (should have data)
    assert!(joyent_vm.datasets.is_some());
    if let Some(datasets) = &joyent_vm.datasets {
        assert_eq!(datasets.len(), 1);
        assert_eq!(
            datasets[0],
            "zones/aaaabbbb-cccc-dddd-eeee-ffffffffffff/data"
        );
    }

    // Validate snapshots
    assert!(joyent_vm.snapshots.is_some());
    if let Some(snapshots) = &joyent_vm.snapshots {
        assert_eq!(snapshots.len(), 1);
        let snapshot = &snapshots[0];
        assert_eq!(snapshot.get("name").and_then(|v| v.as_str()), Some("baseline"));
        assert!(snapshot.get("created").is_some());
    }

    // Validate network configuration with different NIC model
    assert!(joyent_vm.nics.is_some());
    if let Some(nics) = &joyent_vm.nics {
        assert_eq!(nics.len(), 1);
        let nic = &nics[0];
        assert_eq!(nic.mac, "00:00:5e:00:53:af");
        assert_eq!(nic.interface.as_deref(), Some("net0"));
        assert_eq!(nic.primary, Some(true));
        assert_eq!(nic.nic_tag.as_deref(), Some("external"));
        assert_eq!(nic.ip.as_deref(), Some("203.0.113.42"));
        assert_eq!(nic.gateway.as_deref(), Some("203.0.113.1"));
        assert_eq!(nic.netmask.as_deref(), Some("255.255.255.0"));
        assert_eq!(nic.model.as_deref(), Some("e1000"));
    }

    // Validate tags
    assert!(joyent_vm.tags.is_some());
    if let Some(tags) = &joyent_vm.tags {
        assert_eq!(tags.get("role").map(String::as_str), Some("worker"));
    }
}

#[test]
fn test_all_vms_have_required_fields() {
    let json_data = load_vm_list_fixture();
    let vms: Vec<Vm> = serde_json::from_str(&json_data).unwrap();

    for vm in &vms {
        // Every VM should have a UUID (required field)
        assert!(
            !vm.uuid.to_string().is_empty(),
            "VM should have a valid UUID"
        );

        // Validate all VMs have essential operational fields
        assert!(vm.brand.is_some(), "VM should have a brand");
        assert!(vm.state.is_some(), "VM should have a state");
        assert!(vm.zone_state.is_some(), "VM should have a zone_state");
        assert!(vm.owner_uuid.is_some(), "VM should have an owner_uuid");
        assert!(vm.server_uuid.is_some(), "VM should have a server_uuid");
        assert!(vm.image_uuid.is_some(), "VM should have an image_uuid");

        // Validate resource fields exist
        assert!(vm.ram.is_some(), "VM should have RAM allocation");
        assert!(
            vm.max_physical_memory.is_some(),
            "VM should have max_physical_memory"
        );

        // Validate ZFS fields
        assert!(vm.zpool.is_some(), "VM should have a zpool");
        assert!(vm.zonepath.is_some(), "VM should have a zonepath");
        assert!(
            vm.zfs_filesystem.is_some(),
            "VM should have a zfs_filesystem"
        );

        // Validate creation timestamp
        assert!(
            vm.create_timestamp.is_some(),
            "VM should have a create_timestamp"
        );
    }
}

#[test]
fn test_vm_roundtrip_serialization() {
    let json_data = load_vm_list_fixture();
    let vms: Vec<Vm> = serde_json::from_str(&json_data).unwrap();

    // Serialize and deserialize each VM to ensure roundtrip works
    for original_vm in &vms {
        let serialized = serde_json::to_string(original_vm)
            .expect("Should be able to serialize VM");

        let deserialized: Vm = serde_json::from_str(&serialized)
            .expect("Should be able to deserialize serialized VM");

        // Key fields should match
        assert_eq!(original_vm.uuid, deserialized.uuid);
        assert_eq!(original_vm.alias, deserialized.alias);
        assert_eq!(original_vm.brand, deserialized.brand);
        assert_eq!(original_vm.state, deserialized.state);
        assert_eq!(original_vm.owner_uuid, deserialized.owner_uuid);
    }
}

#[test]
fn test_numeric_fields_preserved() {
    let json_data = load_vm_list_fixture();
    let vms: Vec<Vm> = serde_json::from_str(&json_data).unwrap();

    // Find the bhyve VM to test numeric field preservation
    let bhyve_vm = vms
        .iter()
        .find(|vm| vm.brand.as_deref() == Some("bhyve"))
        .expect("Should have a bhyve brand VM");

    // Test that numeric values are preserved (using serde_json::Value)
    // These should match the test data
    if let Some(ram) = &bhyve_vm.ram {
        assert_eq!(ram.as_u64(), Some(65280));
    }

    if let Some(cpu_cap) = &bhyve_vm.cpu_cap {
        assert_eq!(cpu_cap.as_u64(), Some(1600));
    }

    if let Some(cpu_shares) = &bhyve_vm.cpu_shares {
        assert_eq!(cpu_shares.as_u64(), Some(510));
    }

    if let Some(quota) = &bhyve_vm.quota {
        assert_eq!(quota.as_u64(), Some(102400));
    }
}
