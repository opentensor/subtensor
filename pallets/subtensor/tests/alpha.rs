mod mock;
use mock::*;
use pallet_subtensor::*;
use sp_core::U256;
use subnets::Mechanism;
use substrate_fixed::types::I96F32;

// Test titles and descriptions for exhaustive testing of stake_into_subnet function:

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_dynamic_mechanism --exact --nocapture
#[test]
fn test_stake_into_subnet_dynamic_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let tao_to_stake = 1_000_000_000; // 1 TAO

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Add balance to coldkey account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, tao_to_stake);

        // Perform staking
        let alpha_staked =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, tao_to_stake);

        // Verify correct alpha calculation
        let expected_k =
            I96F32::from_num(initial_subnet_alpha) * I96F32::from_num(initial_subnet_tao);
        let expected_alpha_staked = I96F32::from_num(initial_subnet_alpha)
            - (expected_k / I96F32::from_num(initial_subnet_tao + tao_to_stake));
        let expected_alpha_staked_u64 = expected_alpha_staked.to_num::<u64>();
        assert_eq!(
            alpha_staked, expected_alpha_staked_u64,
            "Alpha staked calculation is incorrect"
        );

        // Check subnet alpha and TAO updates
        let new_subnet_alpha = SubnetAlphaIn::<Test>::get(netuid);
        let new_subnet_tao = SubnetTAO::<Test>::get(netuid);
        assert_eq!(
            new_subnet_alpha,
            initial_subnet_alpha - expected_alpha_staked_u64 - 1,
            "Subnet alpha not updated correctly"
        );
        assert_eq!(
            new_subnet_tao,
            initial_subnet_tao + tao_to_stake,
            "Subnet TAO not updated correctly"
        );

        // Ensure global and per-account storage updates
        assert_eq!(
            TotalStake::<Test>::get(),
            tao_to_stake,
            "Total stake not updated correctly"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            tao_to_stake,
            "Stake for hotkey-coldkey pair not updated correctly"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            expected_alpha_staked_u64,
            "Total hotkey alpha not updated correctly"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            expected_alpha_staked_u64,
            "Total coldkey alpha not updated correctly"
        );
        assert_eq!(
            Alpha::<Test>::get((&hotkey, netuid, &coldkey)),
            expected_alpha_staked_u64,
            "Alpha for hotkey-coldkey pair not updated correctly"
        );

        // Check StakingHotkeys update
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            staking_hotkeys.contains(&hotkey),
            "StakingHotkeys not updated correctly"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_stable_mechanism --exact --nocapture
#[test]
fn test_stake_into_subnet_stable_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let tao_to_stake = 1_000_000_000; // 1 TAO

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Add balance to coldkey account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, tao_to_stake);

        // Perform staking
        let alpha_staked =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, tao_to_stake);

        // Verify alpha staked equals TAO staked
        assert_eq!(
            alpha_staked, tao_to_stake,
            "Alpha staked should equal TAO staked in stable mechanism"
        );

        // Check subnet alpha is set to initial
        let new_subnet_alpha = SubnetAlphaIn::<Test>::get(netuid);
        assert_eq!(
            new_subnet_alpha, initial_subnet_alpha,
            "Subnet alpha should be zero in stable mechanism"
        );

        // Check subnet TAO update
        let new_subnet_tao = SubnetTAO::<Test>::get(netuid);
        assert_eq!(
            new_subnet_tao,
            initial_subnet_tao + tao_to_stake,
            "Subnet TAO not updated correctly"
        );

        // Ensure global and per-account storage updates
        assert_eq!(
            TotalStake::<Test>::get(),
            tao_to_stake,
            "Total stake not updated correctly"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            tao_to_stake,
            "Stake for hotkey-coldkey pair not updated correctly"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            alpha_staked,
            "Total hotkey alpha not updated correctly"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            alpha_staked,
            "Total coldkey alpha not updated correctly"
        );
        assert_eq!(
            Alpha::<Test>::get((&hotkey, netuid, &coldkey)),
            alpha_staked,
            "Alpha for hotkey-coldkey pair not updated correctly"
        );

        // Check StakingHotkeys update
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            staking_hotkeys.contains(&hotkey),
            "StakingHotkeys not updated correctly"
        );

        // Check SubnetAlphaOut update
        let subnet_alpha_out = SubnetAlphaOut::<Test>::get(netuid);
        assert_eq!(
            subnet_alpha_out,
            initial_subnet_alpha + alpha_staked,
            "SubnetAlphaOut not updated correctly"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_zero_amount --exact --nocapture
#[test]
fn test_stake_into_subnet_zero_amount() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let tao_to_stake = 0; // Staking zero amount

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial values
        let initial_total_stake = TotalStake::<Test>::get();
        let initial_stake = Stake::<Test>::get(hotkey, coldkey);
        let initial_total_hotkey_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_total_coldkey_alpha = TotalColdkeyAlpha::<Test>::get(coldkey, netuid);
        let initial_alpha = Alpha::<Test>::get((&hotkey, netuid, &coldkey));
        let _initial_staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);

        // Perform staking
        let alpha_staked =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, tao_to_stake);

        // Verify alpha staked is zero
        assert_eq!(
            alpha_staked, 0,
            "Alpha staked should be zero when staking zero amount"
        );

        // Check that all storage items remain unchanged
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao,
            "Subnet TAO should not change"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha,
            "Subnet Alpha In should not change"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha,
            "Subnet Alpha Out should not change"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake,
            "Total stake should not change"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            initial_stake,
            "Stake for hotkey-coldkey pair should not change"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            initial_total_hotkey_alpha,
            "Total hotkey alpha should not change"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            initial_total_coldkey_alpha,
            "Total coldkey alpha should not change"
        );
        assert_eq!(
            Alpha::<Test>::get((&hotkey, netuid, &coldkey)),
            initial_alpha,
            "Alpha for hotkey-coldkey pair should not change"
        );
        // This changes because we created a new connection.
        assert_eq!(
            StakingHotkeys::<Test>::get(coldkey),
            [hotkey],
            "StakingHotkeys should include the hotkey"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_max_amount --exact --nocapture
#[test]
fn test_stake_into_subnet_max_amount() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let max_tao = u64::MAX;

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 1_000_000_000; // 1 TAO
        let initial_subnet_alpha = 500_000; // 0.5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial values
        let initial_total_stake = TotalStake::<Test>::get();
        let initial_stake = Stake::<Test>::get(hotkey, coldkey);
        let initial_total_hotkey_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_total_coldkey_alpha = TotalColdkeyAlpha::<Test>::get(coldkey, netuid);
        let initial_alpha = Alpha::<Test>::get((&hotkey, netuid, &coldkey));

        // Perform staking with maximum amount
        let alpha_staked = SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, max_tao);

        // Verify alpha staked is equal to max_tao (for stable mechanism)
        assert_eq!(
            alpha_staked, max_tao,
            "Alpha staked should be equal to max TAO for stable mechanism"
        );

        // Check storage updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao.saturating_add(max_tao),
            "Subnet TAO should increase by max_tao"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha,
            "Subnet Alpha In should not change for stable mechanism"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha.saturating_add(max_tao),
            "Subnet Alpha Out should increase by max_tao"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake.saturating_add(max_tao),
            "Total stake should increase by max_tao"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            initial_stake.saturating_add(max_tao),
            "Stake for hotkey-coldkey pair should increase by max_tao"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            initial_total_hotkey_alpha.saturating_add(max_tao),
            "Total hotkey alpha should increase by max_tao"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            initial_total_coldkey_alpha.saturating_add(max_tao),
            "Total coldkey alpha should increase by max_tao"
        );
        assert_eq!(
            Alpha::<Test>::get((&hotkey, netuid, &coldkey)),
            initial_alpha.saturating_add(max_tao),
            "Alpha for hotkey-coldkey pair should increase by max_tao"
        );

        // Verify StakingHotkeys is updated
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            staking_hotkeys.contains(&hotkey),
            "StakingHotkeys should contain the hotkey"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_multiple_stakes --exact --nocapture
#[test]
fn test_stake_into_subnet_multiple_stakes() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_amount = 1_000_000; // 1 TAO

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 1_000_000_000; // 1000 TAO
        let initial_subnet_alpha = 500_000_000; // 500 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Record initial values
        let initial_total_stake = TotalStake::<Test>::get();
        let initial_stake = Stake::<Test>::get(hotkey, coldkey);
        let initial_total_hotkey_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_total_coldkey_alpha = TotalColdkeyAlpha::<Test>::get(coldkey, netuid);
        let initial_alpha = Alpha::<Test>::get((&hotkey, netuid, &coldkey));

        // Perform multiple stakes
        let num_stakes = 5;
        let mut total_alpha_staked = 0;

        for _ in 0..num_stakes {
            let alpha_staked =
                SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, stake_amount);
            total_alpha_staked += alpha_staked;
        }

        // Verify cumulative effects
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao + (stake_amount * num_stakes as u64),
            "Subnet TAO should increase by total staked amount"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha,
            "Subnet Alpha In should not change for stable mechanism"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha + total_alpha_staked,
            "Subnet Alpha Out should increase by total alpha staked"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake + (stake_amount * num_stakes as u64),
            "Total stake should increase by total staked amount"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            initial_stake + (stake_amount * num_stakes as u64),
            "Stake for hotkey-coldkey pair should increase by total staked amount"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            initial_total_hotkey_alpha + total_alpha_staked,
            "Total hotkey alpha should increase by total alpha staked"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            initial_total_coldkey_alpha + total_alpha_staked,
            "Total coldkey alpha should increase by total alpha staked"
        );
        assert_eq!(
            Alpha::<Test>::get((&hotkey, netuid, &coldkey)),
            initial_alpha + total_alpha_staked,
            "Alpha for hotkey-coldkey pair should increase by total alpha staked"
        );

        // Verify StakingHotkeys is updated
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            staking_hotkeys.contains(&hotkey),
            "StakingHotkeys should contain the hotkey"
        );
        assert_eq!(
            staking_hotkeys.len(),
            1,
            "StakingHotkeys should only contain one entry"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_different_subnets --exact --nocapture
#[test]
fn test_stake_into_subnet_different_subnets() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_amount = 1_000_000; // 1 TAO

        // Set up two subnets with different mechanisms
        let netuid1 = 1;
        let netuid2 = 2;
        SubnetMechanism::<Test>::insert(netuid1, Mechanism::Stable);
        SubnetMechanism::<Test>::insert(netuid2, Mechanism::Dynamic);

        // Initialize subnets with some existing TAO and Alpha
        let initial_subnet_tao = 1_000_000_000; // 1000 TAO
        let initial_subnet_alpha = 500_000_000; // 500 Alpha
        for netuid in [netuid1, netuid2].iter() {
            SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
            SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
            SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);
        }

        // Stake into subnet 1 (Stable mechanism)
        let alpha_staked1 =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid1, stake_amount);

        // Verify subnet 1 effects
        assert_eq!(
            SubnetTAO::<Test>::get(netuid1),
            initial_subnet_tao + stake_amount,
            "Subnet 1 TAO should increase by staked amount"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid1),
            initial_subnet_alpha,
            "Subnet 1 Alpha In should not change for stable mechanism"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid1),
            initial_subnet_alpha + alpha_staked1,
            "Subnet 1 Alpha Out should increase by alpha staked"
        );
        assert_eq!(
            alpha_staked1, stake_amount,
            "For stable mechanism, alpha staked should equal TAO staked"
        );

        // Stake into subnet 2 (Dynamic mechanism)
        let alpha_staked2 =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid2, stake_amount);

        // Verify subnet 2 effects
        assert_eq!(
            SubnetTAO::<Test>::get(netuid2),
            initial_subnet_tao + stake_amount,
            "Subnet 2 TAO should increase by staked amount"
        );
        assert!(
            SubnetAlphaIn::<Test>::get(netuid2) < initial_subnet_alpha,
            "Subnet 2 Alpha In should decrease for dynamic mechanism"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid2),
            initial_subnet_alpha + alpha_staked2,
            "Subnet 2 Alpha Out should increase by alpha staked"
        );
        assert!(
            alpha_staked2 < stake_amount,
            "For dynamic mechanism, alpha staked should be less than TAO staked"
        );

        // Verify isolated effects
        assert_eq!(
            SubnetTAO::<Test>::get(netuid1),
            initial_subnet_tao + stake_amount,
            "Subnet 1 TAO should remain unchanged after staking in subnet 2"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid1),
            initial_subnet_alpha + alpha_staked1,
            "Subnet 1 Alpha Out should remain unchanged after staking in subnet 2"
        );

        // Verify global effects
        assert_eq!(
            TotalStake::<Test>::get(),
            stake_amount * 2,
            "Total stake should increase by total staked amount across both subnets"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            stake_amount * 2,
            "Stake for hotkey-coldkey pair should increase by total staked amount"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid1),
            alpha_staked1,
            "Total hotkey alpha for subnet 1 should match alpha staked in subnet 1"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid2),
            alpha_staked2,
            "Total hotkey alpha for subnet 2 should match alpha staked in subnet 2"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid1),
            alpha_staked1,
            "Total coldkey alpha for subnet 1 should match alpha staked in subnet 1"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid2),
            alpha_staked2,
            "Total coldkey alpha for subnet 2 should match alpha staked in subnet 2"
        );

        // Verify StakingHotkeys is updated
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            staking_hotkeys.contains(&hotkey),
            "StakingHotkeys should contain the hotkey"
        );
        assert_eq!(
            staking_hotkeys.len(),
            1,
            "StakingHotkeys should only contain one entry"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_hotkey_coldkey_combination --exact --nocapture
#[test]
fn test_stake_into_subnet_hotkey_coldkey_combination() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let stake_amount = 1_000_000; // 1 TAO
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);

        // Stake with hotkey1-coldkey1 combination
        let alpha_staked1 =
            SubtensorModule::stake_into_subnet(&hotkey1, &coldkey1, netuid, stake_amount);
        assert_eq!(
            alpha_staked1, stake_amount,
            "Alpha staked should equal TAO staked for stable mechanism"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey1, netuid, coldkey1)),
            stake_amount,
            "Alpha storage should be updated correctly"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey1, coldkey1),
            stake_amount,
            "Stake storage should be updated correctly"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey1, netuid),
            stake_amount,
            "TotalHotkeyAlpha should be updated"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey1, netuid),
            stake_amount,
            "TotalColdkeyAlpha should be updated"
        );

        // Verify StakingHotkeys for coldkey1
        let staking_hotkeys1 = StakingHotkeys::<Test>::get(coldkey1);
        assert!(
            staking_hotkeys1.contains(&hotkey1),
            "StakingHotkeys should contain hotkey1 for coldkey1"
        );
        assert_eq!(
            staking_hotkeys1.len(),
            1,
            "StakingHotkeys should only contain one entry for coldkey1"
        );

        // Stake with hotkey2-coldkey1 combination
        let alpha_staked2 =
            SubtensorModule::stake_into_subnet(&hotkey2, &coldkey1, netuid, stake_amount);
        assert_eq!(
            alpha_staked2, stake_amount,
            "Alpha staked should equal TAO staked for stable mechanism"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey2, netuid, coldkey1)),
            stake_amount,
            "Alpha storage should be updated correctly"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey2, coldkey1),
            stake_amount,
            "Stake storage should be updated correctly"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey2, netuid),
            stake_amount,
            "TotalHotkeyAlpha should be updated"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey1, netuid),
            stake_amount * 2,
            "TotalColdkeyAlpha should be updated"
        );

        // Verify updated StakingHotkeys for coldkey1
        let updated_staking_hotkeys1 = StakingHotkeys::<Test>::get(coldkey1);
        assert!(
            updated_staking_hotkeys1.contains(&hotkey1),
            "StakingHotkeys should still contain hotkey1 for coldkey1"
        );
        assert!(
            updated_staking_hotkeys1.contains(&hotkey2),
            "StakingHotkeys should now contain hotkey2 for coldkey1"
        );
        assert_eq!(
            updated_staking_hotkeys1.len(),
            2,
            "StakingHotkeys should contain two entries for coldkey1"
        );

        // Stake with hotkey1-coldkey2 combination
        let alpha_staked3 =
            SubtensorModule::stake_into_subnet(&hotkey1, &coldkey2, netuid, stake_amount);
        assert_eq!(
            alpha_staked3, stake_amount,
            "Alpha staked should equal TAO staked for stable mechanism"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey1, netuid, coldkey2)),
            stake_amount,
            "Alpha storage should be updated correctly"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey1, coldkey2),
            stake_amount,
            "Stake storage should be updated correctly"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey1, netuid),
            stake_amount * 2,
            "TotalHotkeyAlpha should be updated"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey2, netuid),
            stake_amount,
            "TotalColdkeyAlpha should be updated"
        );

        // Verify StakingHotkeys for coldkey2
        let staking_hotkeys2 = StakingHotkeys::<Test>::get(coldkey2);
        assert!(
            staking_hotkeys2.contains(&hotkey1),
            "StakingHotkeys should contain hotkey1 for coldkey2"
        );
        assert_eq!(
            staking_hotkeys2.len(),
            1,
            "StakingHotkeys should only contain one entry for coldkey2"
        );

        // Verify total stakes
        assert_eq!(
            TotalStake::<Test>::get(),
            stake_amount * 3,
            "Total stake should be the sum of all stakes"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_edge_cases --exact --nocapture
#[test]
fn test_stake_into_subnet_edge_cases() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Test with very large existing alpha and TAO in subnet
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);
        SubnetTAO::<Test>::insert(netuid, u64::MAX / 2);
        let large_stake = u64::MAX / 1000; // 1 billion
        SubnetAlphaIn::<Test>::insert(netuid, large_stake );
        let alpha_staked_large = SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, large_stake);
        log::debug!(target: "subtensor", "Alpha staked large: {:?}", alpha_staked_large);
        assert!(alpha_staked_large > 0, "Alpha staked should be non-zero for large stake");
        assert!(alpha_staked_large < large_stake, "Alpha staked should be less than TAO staked for dynamic mechanism");

        // Reset subnet values
        SubnetTAO::<Test>::insert(netuid, 0);
        SubnetAlphaIn::<Test>::insert(netuid, 0);

        // Test potential precision loss
        SubnetTAO::<Test>::insert(netuid, u64::MAX);
        SubnetAlphaIn::<Test>::insert(netuid, u64::MAX / 2);
        let precision_stake = u64::MAX / 2;
        let alpha_staked_precision = SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, precision_stake);
        assert!(alpha_staked_precision > 0, "Alpha staked should be non-zero for large stake with potential precision loss");
        assert!(alpha_staked_precision < precision_stake, "Alpha staked should be less than TAO staked for dynamic mechanism with potential precision loss");
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_storage_consistency --exact --nocapture
#[test]
fn test_stake_into_subnet_storage_consistency() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_amount = 1_000_000; // 1 million

        // Initial storage values
        let initial_subnet_tao = SubnetTAO::<Test>::get(netuid);
        let initial_subnet_alpha_in = SubnetAlphaIn::<Test>::get(netuid);
        let initial_subnet_alpha_out = SubnetAlphaOut::<Test>::get(netuid);
        let initial_total_stake = TotalStake::<Test>::get();
        let initial_alpha = Alpha::<Test>::get((hotkey, netuid, coldkey));
        let initial_stake = Stake::<Test>::get(hotkey, coldkey);
        let initial_total_coldkey_alpha = TotalColdkeyAlpha::<Test>::get(coldkey, netuid);
        let initial_total_hotkey_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);

        // Perform staking
        let alpha_staked =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, stake_amount);

        // Verify storage updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao + stake_amount,
            "SubnetTAO should be increased by stake amount"
        );
        assert!(
            SubnetAlphaIn::<Test>::get(netuid) <= initial_subnet_alpha_in,
            "SubnetAlphaIn should not increase"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha_out + alpha_staked,
            "SubnetAlphaOut should be increased by alpha staked"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_total_stake + stake_amount,
            "TotalStake should be increased by stake amount"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            initial_alpha + alpha_staked,
            "Alpha for hotkey-coldkey pair should be increased by alpha staked"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            initial_stake + stake_amount,
            "Stake for hotkey-coldkey pair should be increased by stake amount"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            initial_total_coldkey_alpha + alpha_staked,
            "TotalColdkeyAlpha should be increased by alpha staked"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            initial_total_hotkey_alpha + alpha_staked,
            "TotalHotkeyAlpha should be increased by alpha staked"
        );

        // Verify StakingHotkeys
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            staking_hotkeys.contains(&hotkey),
            "StakingHotkeys should contain the hotkey"
        );

        // Check no unintended side effects
        assert_eq!(
            SubnetTAO::<Test>::get(netuid + 1),
            0,
            "SubnetTAO for other subnets should not be affected"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid + 1),
            0,
            "SubnetAlphaIn for other subnets should not be affected"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid + 1),
            0,
            "SubnetAlphaOut for other subnets should not be affected"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid + 1, coldkey)),
            0,
            "Alpha for other subnets should not be affected"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid + 1),
            0,
            "TotalColdkeyAlpha for other subnets should not be affected"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid + 1),
            0,
            "TotalHotkeyAlpha for other subnets should not be affected"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_stake_into_subnet_return_value --exact --nocapture
#[test]
fn test_stake_into_subnet_return_value() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Scenario 1: Stable mechanism (mechanism_id = 0)
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);
        let stake_amount_1 = 1_000_000; // 1 million
        let alpha_staked_1 =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, stake_amount_1);
        assert_eq!(
            alpha_staked_1, stake_amount_1,
            "For stable mechanism, alpha_staked should equal stake_amount"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            alpha_staked_1,
            "Alpha in storage should match returned value"
        );

        // Reset storage
        Alpha::<Test>::remove((hotkey, netuid, coldkey));
        SubnetAlphaIn::<Test>::remove(netuid);
        SubnetAlphaOut::<Test>::remove(netuid);
        SubnetTAO::<Test>::remove(netuid);

        // Scenario 2: Dynamic mechanism (mechanism_id = 1)
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);
        let initial_subnet_tao = 10_000_000; // 10 million
        let initial_subnet_alpha = 5_000_000; // 5 million
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);

        let stake_amount_2 = 2_000_000; // 2 million
        let alpha_staked_2 =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, stake_amount_2);

        // Calculate expected alpha staked
        let k: I96F32 =
            I96F32::from_num(initial_subnet_alpha) * I96F32::from_num(initial_subnet_tao);
        let expected_alpha_staked: u64 = (I96F32::from_num(initial_subnet_alpha)
            - (k / (I96F32::from_num(initial_subnet_tao) + I96F32::from_num(stake_amount_2))))
        .to_num::<u64>();

        assert_eq!(
            alpha_staked_2, expected_alpha_staked,
            "For dynamic mechanism, alpha_staked should match the calculated value"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            alpha_staked_2,
            "Alpha in storage should match returned value"
        );

        // Verify consistency with other storage updates
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            alpha_staked_2,
            "SubnetAlphaOut should match the returned alpha_staked"
        );
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao + stake_amount_2,
            "SubnetTAO should be increased by stake amount"
        );
    });
}

// Test titles and descriptions for exhaustive testing of unstake_from_subnet function:

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_dynamic_mechanism --exact --nocapture
#[test]
fn test_unstake_from_subnet_dynamic_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Stake some alpha first
        let alpha_to_stake = 1_000_000; // 1 Alpha
        Alpha::<Test>::insert((hotkey, netuid, coldkey), alpha_to_stake);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, alpha_to_stake);
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, alpha_to_stake);

        // Perform unstaking
        let tao_unstaked =
            SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_to_stake);

        // Verify correct TAO calculation
        let expected_k =
            I96F32::from_num(initial_subnet_alpha) * I96F32::from_num(initial_subnet_tao);
        let expected_tao_unstaked = I96F32::from_num(initial_subnet_tao)
            - (expected_k / I96F32::from_num(initial_subnet_alpha + alpha_to_stake));
        let expected_tao_unstaked_u64 = expected_tao_unstaked.to_num::<u64>();
        assert_eq!(
            tao_unstaked, expected_tao_unstaked_u64,
            "TAO unstaked calculation is incorrect"
        );

        // Check subnet alpha and TAO updates
        let new_subnet_alpha = SubnetAlphaIn::<Test>::get(netuid);
        let new_subnet_tao = SubnetTAO::<Test>::get(netuid);
        assert_eq!(
            new_subnet_alpha,
            initial_subnet_alpha + alpha_to_stake,
            "Subnet alpha not updated correctly"
        );
        assert_eq!(
            new_subnet_tao,
            initial_subnet_tao - tao_unstaked,
            "Subnet TAO not updated correctly"
        );

        // Ensure global and per-account storage updates
        assert_eq!(
            TotalStake::<Test>::get(),
            0,
            "Total stake not updated correctly"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            0,
            "Stake for hotkey-coldkey pair not updated correctly"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            0,
            "Total hotkey alpha not updated correctly"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            0,
            "Total coldkey alpha not updated correctly"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            0,
            "Alpha for hotkey-coldkey pair not updated correctly"
        );

        // Verify StakingHotkeys update
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            !staking_hotkeys.contains(&hotkey),
            "Hotkey should be removed from StakingHotkeys"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_stable_mechanism --exact --nocapture
#[test]
fn test_unstake_from_subnet_stable_mechanism() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Stake some alpha first
        let alpha_to_stake = 1_000_000; // 1 Alpha
        Alpha::<Test>::insert((hotkey, netuid, coldkey), alpha_to_stake);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, alpha_to_stake);
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, alpha_to_stake);
        Stake::<Test>::insert(hotkey, coldkey, alpha_to_stake);
        TotalStake::<Test>::put(alpha_to_stake);

        // Perform unstaking
        let tao_unstaked =
            SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_to_stake);

        // Verify TAO unstaked equals alpha unstaked
        assert_eq!(
            tao_unstaked, alpha_to_stake,
            "TAO unstaked should equal alpha unstaked in stable mechanism"
        );

        // Check subnet alpha and TAO updates
        let new_subnet_alpha = SubnetAlphaIn::<Test>::get(netuid);
        let new_subnet_tao = SubnetTAO::<Test>::get(netuid);
        assert_eq!(
            new_subnet_alpha, 0,
            "Subnet alpha should be zero in stable mechanism"
        );
        assert_eq!(
            new_subnet_tao,
            initial_subnet_tao - tao_unstaked,
            "Subnet TAO not updated correctly"
        );

        // Ensure global and per-account storage updates
        assert_eq!(
            TotalStake::<Test>::get(),
            0,
            "Total stake not updated correctly"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            0,
            "Stake for hotkey-coldkey pair not updated correctly"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            0,
            "Total hotkey alpha not updated correctly"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            0,
            "Total coldkey alpha not updated correctly"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            0,
            "Alpha for hotkey-coldkey pair not updated correctly"
        );

        // Verify StakingHotkeys update
        let staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            !staking_hotkeys.contains(&hotkey),
            "Hotkey should be removed from StakingHotkeys"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_zero_alpha --exact --nocapture
#[test]
fn test_unstake_from_subnet_zero_alpha() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Stake some alpha first
        let alpha_to_stake = 1_000_000; // 1 Alpha
        Alpha::<Test>::insert((hotkey, netuid, coldkey), alpha_to_stake);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, alpha_to_stake);
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, alpha_to_stake);
        Stake::<Test>::insert(hotkey, coldkey, alpha_to_stake);
        TotalStake::<Test>::put(alpha_to_stake);

        // Perform unstaking of zero alpha
        let tao_unstaked = SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, 0);

        // Verify no changes occurred
        assert_eq!(tao_unstaked, 0, "No TAO should be unstaked");
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao,
            "Subnet TAO should not change"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid),
            initial_subnet_alpha,
            "Subnet alpha should not change"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha,
            "Subnet alpha out should not change"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            alpha_to_stake,
            "Alpha for hotkey-coldkey pair should not change"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            alpha_to_stake,
            "Total coldkey alpha should not change"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            alpha_to_stake,
            "Total hotkey alpha should not change"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            alpha_to_stake,
            "Stake for hotkey-coldkey pair should not change"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            alpha_to_stake,
            "Total stake should not change"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_all_alpha --exact --nocapture
#[test]
fn test_unstake_from_subnet_all_alpha() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Stake some alpha first
        let alpha_to_stake = 1_000_000; // 1 Alpha
        Alpha::<Test>::insert((hotkey, netuid, coldkey), alpha_to_stake);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, alpha_to_stake);
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, alpha_to_stake);
        Stake::<Test>::insert(hotkey, coldkey, alpha_to_stake);
        TotalStake::<Test>::put(alpha_to_stake);

        // Add hotkey to StakingHotkeys
        let mut staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        staking_hotkeys.push(hotkey);
        StakingHotkeys::<Test>::insert(coldkey, staking_hotkeys);

        // Perform unstaking of all alpha
        let tao_unstaked =
            SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_to_stake);

        // Verify proper removal of storage entries
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao - tao_unstaked,
            "Subnet TAO should be updated"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha - alpha_to_stake,
            "Subnet alpha out should be updated"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            0,
            "Alpha for hotkey-coldkey pair should be zero"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            0,
            "Total coldkey alpha should be zero"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            0,
            "Total hotkey alpha should be zero"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            0,
            "Stake for hotkey-coldkey pair should be zero"
        );
        assert_eq!(TotalStake::<Test>::get(), 0, "Total stake should be zero");

        // Verify StakingHotkeys update
        let updated_staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            !updated_staking_hotkeys.contains(&hotkey),
            "Hotkey should be removed from StakingHotkeys"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_partial_alpha --exact --nocapture
#[test]
fn test_unstake_from_subnet_partial_alpha() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 50_000_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Stake some alpha first
        let alpha_to_stake = 2_000_000; // 2 Alpha
        Alpha::<Test>::insert((hotkey, netuid, coldkey), alpha_to_stake);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, alpha_to_stake);
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, alpha_to_stake);
        Stake::<Test>::insert(hotkey, coldkey, alpha_to_stake);
        TotalStake::<Test>::put(alpha_to_stake);

        // Perform partial unstaking
        let alpha_to_unstake = 1_000_000; // 1 Alpha
        let tao_unstaked =
            SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_to_unstake);

        // Verify storage updates
        assert!(tao_unstaked > 0, "Partial TAO should be unstaked");
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao - tao_unstaked,
            "Subnet TAO should be updated"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha - alpha_to_unstake,
            "Subnet alpha out should be updated"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            alpha_to_stake - alpha_to_unstake,
            "Alpha for hotkey-coldkey pair should be updated"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            alpha_to_stake - alpha_to_unstake,
            "Total coldkey alpha should be updated"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            alpha_to_stake - alpha_to_unstake,
            "Total hotkey alpha should be updated"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            alpha_to_stake - tao_unstaked,
            "Stake for hotkey-coldkey pair should be updated"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            alpha_to_stake - tao_unstaked,
            "Total stake should be updated"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_nonexistent_stake --exact --nocapture
#[test]
fn test_unstake_from_subnet_nonexistent_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Attempt to unstake from a non-existent stake
        let alpha_to_unstake = 1_000_000; // 1 Alpha
        let tao_unstaked =
            SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_to_unstake);

        // Verify no changes
        assert_eq!(tao_unstaked, 0, "No TAO should be unstaked");
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao,
            "Subnet TAO should remain unchanged"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha,
            "Subnet alpha out should remain unchanged"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey, netuid, coldkey)),
            0,
            "Alpha for hotkey-coldkey pair should remain zero"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            0,
            "Total coldkey alpha should remain zero"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey, netuid),
            0,
            "Total hotkey alpha should remain zero"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey, coldkey),
            0,
            "Stake for hotkey-coldkey pair should remain zero"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            0,
            "Total stake should remain zero"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_multiple_hotkeys --exact --nocapture
#[test]
fn test_unstake_from_subnet_multiple_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);

        // Set up the subnet with dynamic mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Dynamic);

        // Initialize subnet with some existing TAO and Alpha
        let initial_subnet_tao = 10_000_000_000; // 10 TAO
        let initial_subnet_alpha = 5_000_000; // 5 Alpha
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao);
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha);
        SubnetAlphaOut::<Test>::insert(netuid, initial_subnet_alpha);

        // Stake some alpha for both hotkeys
        let alpha_to_stake = 1_000_000; // 1 Alpha
        Alpha::<Test>::insert((hotkey1, netuid, coldkey), alpha_to_stake);
        Alpha::<Test>::insert((hotkey2, netuid, coldkey), alpha_to_stake);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, alpha_to_stake * 2);
        TotalHotkeyAlpha::<Test>::insert(hotkey1, netuid, alpha_to_stake);
        TotalHotkeyAlpha::<Test>::insert(hotkey2, netuid, alpha_to_stake);
        Stake::<Test>::insert(hotkey1, coldkey, initial_subnet_tao);
        Stake::<Test>::insert(hotkey2, coldkey, initial_subnet_tao);
        TotalStake::<Test>::put(initial_subnet_tao);

        // Add both hotkeys to StakingHotkeys
        let staking_hotkeys = vec![hotkey1, hotkey2];
        StakingHotkeys::<Test>::insert(coldkey, staking_hotkeys.clone());

        // Unstake all alpha from hotkey1
        let tao_unstaked =
            SubtensorModule::unstake_from_subnet(&hotkey1, &coldkey, netuid, alpha_to_stake);

        // Verify storage updates
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            initial_subnet_tao - tao_unstaked,
            "Subnet TAO should be updated"
        );
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            initial_subnet_alpha - alpha_to_stake,
            "Subnet alpha out should be updated"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey1, netuid, coldkey)),
            0,
            "Alpha for hotkey1-coldkey pair should be zero"
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey2, netuid, coldkey)),
            alpha_to_stake,
            "Alpha for hotkey2-coldkey pair should remain unchanged"
        );
        assert_eq!(
            TotalColdkeyAlpha::<Test>::get(coldkey, netuid),
            alpha_to_stake,
            "Total coldkey alpha should be updated"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey1, netuid),
            0,
            "Total hotkey1 alpha should be zero"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey2, netuid),
            alpha_to_stake,
            "Total hotkey2 alpha should remain unchanged"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey1, coldkey),
            initial_subnet_tao - tao_unstaked,
            "Stake for hotkey1-coldkey pair should be updated"
        );
        assert_eq!(
            Stake::<Test>::get(hotkey2, coldkey),
            initial_subnet_tao,
            "Stake for hotkey2-coldkey pair should remain unchanged"
        );
        assert_eq!(
            TotalStake::<Test>::get(),
            initial_subnet_tao - tao_unstaked,
            "Total stake should be updated"
        );

        // Verify StakingHotkeys update
        let updated_staking_hotkeys = StakingHotkeys::<Test>::get(coldkey);
        assert!(
            !updated_staking_hotkeys.contains(&hotkey1),
            "Hotkey1 should be removed from StakingHotkeys"
        );
        assert!(
            updated_staking_hotkeys.contains(&hotkey2),
            "Hotkey2 should remain in StakingHotkeys"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_edge_cases --exact --nocapture
#[test]
fn test_unstake_from_subnet_edge_cases() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Test case 1: Maximum u64 values
        let max_u64 = u64::MAX;
        SubnetTAO::<Test>::insert(netuid, max_u64);
        SubnetAlphaIn::<Test>::insert(netuid, max_u64);
        SubnetAlphaOut::<Test>::insert(netuid, max_u64);
        Alpha::<Test>::insert((hotkey, netuid, coldkey), max_u64);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, max_u64);
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, max_u64);
        Stake::<Test>::insert(hotkey, coldkey, max_u64);
        TotalStake::<Test>::put(max_u64);

        let unstaked = SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, max_u64);
        assert_eq!(
            unstaked, max_u64,
            "Should unstake maximum u64 value without overflow"
        );

        // Test case 2: Unstaking more than staked
        SubnetTAO::<Test>::insert(netuid, 1000);
        SubnetAlphaIn::<Test>::insert(netuid, 1000);
        SubnetAlphaOut::<Test>::insert(netuid, 1000);
        Alpha::<Test>::insert((hotkey, netuid, coldkey), 500);
        let unstaked = SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, 1000);
        assert_eq!(unstaked, 500, "Should only unstake available amount");

        // Test case 3: Unstaking from empty subnet
        SubnetTAO::<Test>::insert(netuid, 0);
        SubnetAlphaIn::<Test>::insert(netuid, 0);
        SubnetAlphaOut::<Test>::insert(netuid, 0);
        Alpha::<Test>::insert((hotkey, netuid, coldkey), 0);
        let unstaked = SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, 100);
        assert_eq!(unstaked, 0, "Should not unstake from empty subnet");
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_concurrent_stakes --exact --nocapture
#[test]
fn test_unstake_from_subnet_concurrent_stakes() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Initialize subnet with some existing TAO and Alpha
        SubnetTAO::<Test>::insert(netuid, 10_000);
        SubnetAlphaIn::<Test>::insert(netuid, 10_000);
        SubnetAlphaOut::<Test>::insert(netuid, 10_000);

        // Stake for both hotkeys
        Alpha::<Test>::insert((hotkey1, netuid, coldkey), 5000);
        Alpha::<Test>::insert((hotkey2, netuid, coldkey), 5000);
        TotalColdkeyAlpha::<Test>::insert(coldkey, netuid, 10_000);
        TotalHotkeyAlpha::<Test>::insert(hotkey1, netuid, 5000);
        TotalHotkeyAlpha::<Test>::insert(hotkey2, netuid, 5000);
        Stake::<Test>::insert(hotkey1, coldkey, 5000);
        Stake::<Test>::insert(hotkey2, coldkey, 5000);
        TotalStake::<Test>::put(10_000);

        // Unstake from hotkey1
        let unstaked = SubtensorModule::unstake_from_subnet(&hotkey1, &coldkey, netuid, 3000);
        assert_eq!(unstaked, 3000, "Should unstake 3000 from hotkey1");

        // Verify storage updates
        assert_eq!(Alpha::<Test>::get((hotkey1, netuid, coldkey)), 2000);
        assert_eq!(Alpha::<Test>::get((hotkey2, netuid, coldkey)), 5000);
        assert_eq!(TotalColdkeyAlpha::<Test>::get(coldkey, netuid), 7000);
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey1, netuid), 2000);
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey2, netuid), 5000);
        assert_eq!(Stake::<Test>::get(hotkey1, coldkey), 2000);
        assert_eq!(Stake::<Test>::get(hotkey2, coldkey), 5000);
        assert_eq!(TotalStake::<Test>::get(), 7000);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_unstake_from_subnet_return_value --exact --nocapture
#[test]
fn test_unstake_from_subnet_return_value() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        // Set up the subnet with stable mechanism
        SubnetMechanism::<Test>::insert(netuid, Mechanism::Stable);

        // Initialize subnet with some existing TAO and Alpha
        SubnetTAO::<Test>::insert(netuid, 10_000);
        SubnetAlphaIn::<Test>::insert(netuid, 10_000);
        SubnetAlphaOut::<Test>::insert(netuid, 10_000);
        Alpha::<Test>::insert((hotkey, netuid, coldkey), 5000);

        // Test case 1: Unstake exact amount
        let unstaked = SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, 5000);
        assert_eq!(unstaked, 5000, "Should return exact unstaked amount");

        // Test case 2: Unstake more than available
        Alpha::<Test>::insert((hotkey, netuid, coldkey), 3000);
        let unstaked = SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, 5000);
        assert_eq!(
            unstaked, 3000,
            "Should return available amount when unstaking more than available"
        );

        // Test case 3: Unstake from empty stake
        Alpha::<Test>::insert((hotkey, netuid, coldkey), 0);
        let unstaked = SubtensorModule::unstake_from_subnet(&hotkey, &coldkey, netuid, 1000);
        assert_eq!(
            unstaked, 0,
            "Should return 0 when unstaking from empty stake"
        );
    });
}
