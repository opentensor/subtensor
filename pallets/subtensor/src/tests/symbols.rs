#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::mock::*;

use crate::*;
use subtensor_runtime_common::NetUid;

// `get_name_for_subnet` was refactored from a 438-arm `match` that allocated a
// fresh `Vec<u8>` on every call into a static lookup over the `NAMES` table.
// These tests pin the observable contract so the refactor is provably
// behaviour-preserving:
//   - representative entries (first, last, one with a space, a deliberate
//     `"unknown"` sentinel slot) return the same bytes the old match did,
//   - out-of-range netuids fall back to `b"unknown"`,
//   - a non-empty `SubnetIdentitiesV3::subnet_name` still wins over the table,
//   - an empty `SubnetIdentitiesV3::subnet_name` still falls through to it.

#[test]
fn get_name_for_subnet_returns_table_entry_for_known_netuids() {
    new_test_ext(1).execute_with(|| {
        // First entry — the root subnet.
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(0u16)),
            b"root".to_vec()
        );

        // Second entry — apex / netuid 1.
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(1u16)),
            b"apex".to_vec()
        );

        // The one entry whose name contains a space — used to live on a
        // match arm that returned `b"red team".to_vec()`.
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(61u16)),
            b"red team".to_vec()
        );

        // Last entry in the table.
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(438u16)),
            b"ra".to_vec()
        );
    });
}

#[test]
fn get_name_for_subnet_preserves_unknown_sentinel_slots() {
    // Indices 55 and 60 were `b"unknown".to_vec()` in the original match
    // (they correspond to the ث / ذ code points that have no assigned
    // subnet name). The refactor must keep them as `b"unknown"` rather
    // than silently shifting other names into those slots.
    new_test_ext(1).execute_with(|| {
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(55u16)),
            b"unknown".to_vec()
        );
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(60u16)),
            b"unknown".to_vec()
        );
    });
}

#[test]
fn get_name_for_subnet_falls_back_to_unknown_for_out_of_range_netuids() {
    // The old code had `_ => b"unknown".to_vec()` at the bottom of the
    // match. The refactor uses `NAMES.get(idx).copied().unwrap_or(b"unknown")`.
    // Both must surface `b"unknown"` for any index past the table length.
    new_test_ext(1).execute_with(|| {
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(439u16)),
            b"unknown".to_vec()
        );
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(1_000u16)),
            b"unknown".to_vec()
        );
        assert_eq!(
            SubtensorModule::get_name_for_subnet(NetUid::from(u16::MAX)),
            b"unknown".to_vec()
        );
    });
}

#[test]
fn get_name_for_subnet_prefers_stored_identity_over_table() {
    // A subnet owner who sets a non-empty `SubnetIdentitiesV3::subnet_name`
    // must keep winning over the table default. Previously this was the
    // `.and_then(|identity| if !identity.subnet_name.is_empty() ...)` branch
    // at the top of the function; the refactor preserves it unchanged.
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        add_network(netuid, 1, 0);

        let identity = SubnetIdentityOfV3 {
            subnet_name: b"my-custom-subnet".to_vec(),
            github_repo: Vec::new(),
            subnet_contact: Vec::new(),
            subnet_url: Vec::new(),
            discord: Vec::new(),
            description: Vec::new(),
            logo_url: Vec::new(),
            additional: Vec::new(),
        };
        SubnetIdentitiesV3::<Test>::insert(netuid, identity);

        assert_eq!(
            SubtensorModule::get_name_for_subnet(netuid),
            b"my-custom-subnet".to_vec()
        );
    });
}

#[test]
fn get_name_for_subnet_falls_through_to_table_for_empty_identity_name() {
    // If the owner has written an identity but left `subnet_name` empty,
    // we must still fall through to the built-in table entry — not return
    // an empty string.
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        add_network(netuid, 1, 0);

        let identity = SubnetIdentityOfV3 {
            subnet_name: Vec::new(),
            github_repo: Vec::new(),
            subnet_contact: Vec::new(),
            subnet_url: Vec::new(),
            discord: Vec::new(),
            description: Vec::new(),
            logo_url: Vec::new(),
            additional: Vec::new(),
        };
        SubnetIdentitiesV3::<Test>::insert(netuid, identity);

        assert_eq!(
            SubtensorModule::get_name_for_subnet(netuid),
            b"apex".to_vec()
        );
    });
}

#[test]
fn names_and_symbols_tables_have_matching_length() {
    // The refactor leans on NAMES and SYMBOLS being the same length so a
    // given `netuid` indexes into both consistently. Pin that here so a
    // future edit to one table without the other fails fast.
    assert_eq!(crate::subnets::symbols::NAMES.len(), 439);
    assert_eq!(crate::subnets::symbols::SYMBOLS.len(), 439);
    assert_eq!(
        crate::subnets::symbols::NAMES.len(),
        crate::subnets::symbols::SYMBOLS.len()
    );
}
