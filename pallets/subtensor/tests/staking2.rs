mod mock;
use mock::*;
use pallet_subtensor::*;
use sp_core::U256;
use substrate_fixed::types::I96F32;

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking2 -- test_swap_tao_for_alpha_dynamic_mechanism --exact --nocapture
#[test]
fn test_swap_tao_for_alpha_dynamic_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let tao_to_swap = 1_000_000_000; // 1 TAO

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, 1);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial total stake
        let initial_total_stake = TotalStake::<Test>::get();

        // Perform swap
        let alpha_received = SubtensorModule::swap_tao_for_alpha(netuid, tao_to_swap);

        // Verify correct alpha calculation using constant product formula
        let k = I96F32::from_num(initial_subnet_alpha) * I96F32::from_num(initial_subnet_tao);
        let expected_alpha = I96F32::from_num(initial_subnet_alpha) - 
            (k / (I96F32::from_num(initial_subnet_tao + tao_to_swap)));
        let expected_alpha_u64 = expected_alpha.to_num::<u64>();
        
        assert_eq!(
            alpha_received, expected_alpha_u64,
            "Alpha received calculation is incorrect"
        );

        // Check subnet updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao + tao_to_swap,
            "Subnet TAO not updated correctly"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha - alpha_received,
            "Subnet Alpha In not updated correctly"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha + alpha_received,
            "Subnet Alpha Out not updated correctly"
        );

        // Check total stake update
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake + tao_to_swap,
            "Total stake not updated correctly"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking2 -- test_swap_tao_for_alpha_stable_mechanism --exact --nocapture
#[test]
fn test_swap_tao_for_alpha_stable_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let tao_to_swap = 1_000_000_000; // 1 TAO

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, 0);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial total stake
        let initial_total_stake = TotalStake::<Test>::get();

        // Perform swap
        let alpha_received = SubtensorModule::swap_tao_for_alpha(netuid, tao_to_swap);

        // Verify alpha received equals TAO swapped in stable mechanism
        assert_eq!(
            alpha_received, tao_to_swap,
            "Alpha received should equal TAO swapped in stable mechanism"
        );

        // Check subnet updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao + tao_to_swap,
            "Subnet TAO not updated correctly"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha - alpha_received,
            "Subnet Alpha In not updated correctly"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha + alpha_received,
            "Subnet Alpha Out not updated correctly"
        );

        // Check total stake update
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake + tao_to_swap,
            "Total stake not updated correctly"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking2 -- test_swap_alpha_for_tao_dynamic_mechanism --exact --nocapture
#[test]
fn test_swap_alpha_for_tao_dynamic_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let alpha_to_swap = 1_000_000; // 1 Alpha

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, 1);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial total stake
        let initial_total_stake = TotalStake::<Test>::get();

        // Perform swap
        let tao_received = SubtensorModule::swap_alpha_for_tao(netuid, alpha_to_swap);

        // Verify correct TAO calculation using constant product formula
        let k = I96F32::from_num(initial_subnet_alpha) * I96F32::from_num(initial_subnet_tao);
        let expected_tao = I96F32::from_num(initial_subnet_tao) - 
            (k / (I96F32::from_num(initial_subnet_alpha + alpha_to_swap)));
        let expected_tao_u64 = expected_tao.to_num::<u64>();

        assert_eq!(
            tao_received, expected_tao_u64,
            "TAO received calculation is incorrect"
        );

        // Check subnet updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao - tao_received,
            "Subnet TAO not updated correctly"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha + alpha_to_swap,
            "Subnet Alpha In not updated correctly"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha - alpha_to_swap,
            "Subnet Alpha Out not updated correctly"
        );

        // Check total stake update
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake - tao_received,
            "Total stake not updated correctly"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking2 -- test_swap_alpha_for_tao_stable_mechanism --exact --nocapture
#[test]
fn test_swap_alpha_for_tao_stable_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let alpha_to_swap = 1_000_000; // 1 Alpha

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, 0);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial total stake
        let initial_total_stake = TotalStake::<Test>::get();

        // Perform swap
        let tao_received = SubtensorModule::swap_alpha_for_tao(netuid, alpha_to_swap);

        // Verify TAO received equals alpha swapped in stable mechanism
        assert_eq!(
            tao_received, alpha_to_swap,
            "TAO received should equal alpha swapped in stable mechanism"
        );

        // Check subnet updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao - tao_received,
            "Subnet TAO not updated correctly"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha + alpha_to_swap,
            "Subnet Alpha In not updated correctly"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha - alpha_to_swap,
            "Subnet Alpha Out not updated correctly"
        );

        // Check total stake update
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake - tao_received,
            "Total stake not updated correctly"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking2 -- test_swap_edge_cases --exact --nocapture
#[test]
fn test_swap_edge_cases() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;

        // Test case 1: Zero amount swaps
        SubnetMechanism::<Test>::insert(netuid, 1);
        let alpha_received = SubtensorModule::swap_tao_for_alpha(netuid, 0);
        assert_eq!(alpha_received, 0, "Zero TAO swap should return zero alpha");

        let tao_received = SubtensorModule::swap_alpha_for_tao(netuid, 0);
        assert_eq!(tao_received, 0, "Zero alpha swap should return zero TAO");

        // Test case 2: Maximum values
        SubnetTAO::<Test>::insert(netuid, u64::MAX);
        SubnetAlphaIn::<Test>::insert(netuid, u64::MAX);
        SubnetAlphaOut::<Test>::insert(netuid, 0);

        let large_amount = u64::MAX / 2;
        let alpha_received = SubtensorModule::swap_tao_for_alpha(netuid, large_amount);
        assert!(alpha_received > 0, "Large TAO swap should return non-zero alpha");
        assert!(alpha_received < large_amount, "Alpha received should be less than TAO swapped");

        // Test case 3: Empty subnet
        SubnetTAO::<Test>::insert(netuid, 0);
        SubnetAlphaIn::<Test>::insert(netuid, 0);
        SubnetAlphaOut::<Test>::insert(netuid, 0);

        let alpha_received = SubtensorModule::swap_tao_for_alpha(netuid, 1000);
        assert_eq!(alpha_received, 0, "Empty subnet should return zero alpha");

        let tao_received = SubtensorModule::swap_alpha_for_tao(netuid, 1000);
        assert_eq!(tao_received, 0, "Empty subnet should return zero TAO");
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking2 -- test_swap_multiple_operations --exact --nocapture
#[test]
fn test_swap_multiple_operations() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, 1);

        // Initialize subnet
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Perform multiple swaps
        let tao_amount = 1_000_000;
        let mut total_alpha_received = 0;
        let mut total_tao_spent = 0;

        // Multiple TAO to Alpha swaps
        for _ in 0..5 {
            let alpha_received = SubtensorModule::swap_tao_for_alpha(netuid, tao_amount);
            total_alpha_received += alpha_received;
            total_tao_spent += tao_amount;
        }

        // Verify cumulative effects
        assert!(total_alpha_received > 0, "Should receive non-zero alpha");
        assert!(
            total_alpha_received < total_tao_spent,
            "Total alpha received should be less than total TAO spent in dynamic mechanism"
        );

        // Check final state
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao + total_tao_spent,
            "Final subnet TAO incorrect"
        );
        assert!(
            SubnetAlphaIn::<Test>::get(netuid) < initial_subnet_alpha,
            "Final subnet alpha in should decrease"
        );
        assert!(
            SubnetAlphaOut::<Test>::get(netuid) > initial_subnet_alpha,
            "Final subnet alpha out should increase"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test staking2 -- test_swap_mechanism_transition --exact --nocapture
#[test]
fn test_swap_mechanism_transition() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let swap_amount = 1_000_000;

        // Initialize subnet
        let initial_subnet_tao = 10_000_000_000;
        let initial_subnet_alpha = 5_000_000;
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // First swap with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, 0);
        let stable_alpha = SubtensorModule::swap_tao_for_alpha(netuid, swap_amount);
        assert_eq!(
            stable_alpha, swap_amount,
            "Stable mechanism should swap 1:1"
        );

        // Switch to dynamic mechanism and swap
        SubnetMechanism::<Test>::insert(netuid, 1);
        let dynamic_alpha = SubtensorModule::swap_tao_for_alpha(netuid, swap_amount);
        assert!(
            dynamic_alpha < swap_amount,
            "Dynamic mechanism should return less alpha than TAO"
        );

        // Switch back to stable and verify
        SubnetMechanism::<Test>::insert(netuid, 0);
        let final_stable_alpha = SubtensorModule::swap_tao_for_alpha(netuid, swap_amount);
        assert_eq!(
            final_stable_alpha, swap_amount,
            "Should return to 1:1 swap with stable mechanism"
        );
    });
}
