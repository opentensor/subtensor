#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use crate::mock::*;
mod mock;
use coinbase::block_emission;
use frame_support::{assert_err, assert_ok};
use pallet_subtensor::*;
use sp_core::Get;
use sp_core::U256;
use substrate_fixed::types::I96F32;

// Test the ability to hash all sorts of hotkeys.
#[test]

fn test_hotkey_hashing() {
    new_test_ext(1).execute_with(|| {
        for i in 0..10000 {
            SubtensorModule::hash_hotkey_to_u64(&U256::from(i));
        }
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --workspace --test coinbase -- test_dynamic_flow --exact --nocapture
#[test]
fn test_dynamic_flow() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let block_emission: u64 = 1000000000;
        let initial_subnet_tao: u64 = 10_000_000_000;
        let initial_subnet_alpha: u64 = 100_000_000_000;
        add_network(netuid, 110, 100);
        // Tempo::<Test>::insert(netuid, 1);
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetTAO::<Test>::insert(netuid, initial_subnet_tao); // 10 TAO
        SubnetAlphaIn::<Test>::insert(netuid, initial_subnet_alpha); // 100 Alpha
        SubtensorModule::register_neuron( netuid, &hotkey );

        // Stake into the network adding 1 TAO through he pool.
        let stake: u64 = 1_000_000_000;
        SubtensorModule::stake_into_subnet( &hotkey, &coldkey, netuid, stake ); // Stake into subnet.

        // Stake into root adding 1 TAO through the pool.
        SubtensorModule::stake_into_subnet( &hotkey, &coldkey, 0, stake ); // Stake into root.

        // Check price function.
        let total_alpha_reserves: u64 = SubnetAlphaIn::<Test>::get(netuid);
        let total_tao_reserves: u64 = SubnetTAO::<Test>::get(netuid);
        let expected_alpha_price: I96F32 = I96F32::from_num(total_tao_reserves) / I96F32::from_num(total_alpha_reserves);
        assert_eq!( SubtensorModule::get_alpha_price( netuid ), expected_alpha_price );

        // Run a block forward.
        let total_issuance_before: u64 = TotalIssuance::<Test>::get();
        let total_stake_before: u64 = TotalStake::<Test>::get();
        let subnet_tao_before: u64 = SubnetTAO::<Test>::get(netuid);
        let alpha_out_before: u64 = SubnetAlphaOut::<Test>::get(netuid);
        let alpha_in_before: u64 = SubnetAlphaIn::<Test>::get(netuid);
        step_block(1);
        // Single subnet gets 100% of block emission.
        assert_eq!(EmissionValues::<Test>::get(netuid), block_emission);
        // Alpha_in emission = 1 because alpha_price < emission_value/block_emission
        assert_eq!(SubnetAlphaInEmission::<Test>::get(netuid), block_emission);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid), alpha_in_before + block_emission);
        // Tao_in emission = block_emission * alpha_price
        let expected_tao_emission: u64 = (I96F32::from_num(block_emission) * expected_alpha_price).to_num::<u64>();
        assert_eq!(SubnetTaoInEmission::<Test>::get(netuid), expected_tao_emission );
        assert_eq!(TotalIssuance::<Test>::get(), total_issuance_before + expected_tao_emission );
        assert_eq!(TotalStake::<Test>::get(), total_stake_before + expected_tao_emission );
        assert_eq!(SubnetTAO::<Test>::get(netuid), subnet_tao_before + expected_tao_emission );
        // Alpha_out = 2 - alpha_in thus == 2*block_emission - block_emission = block_emission
        assert_eq!(SubnetAlphaOutEmission::<Test>::get(netuid), block_emission);
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid), alpha_out_before); // Unchanged.
        assert_eq!(PendingEmission::<Test>::get(netuid), block_emission); // All alpha..

        // Reset pending.
        PendingRootDivs::<Test>::insert(netuid, 0);
        PendingEmission::<Test>::insert(netuid, 0);

        // Set tao weight.
        TaoWeight::<Test>::insert( netuid, u64::MAX );
        let total_issuance_before: u64 = TotalIssuance::<Test>::get();
        let total_stake_before: u64 = TotalStake::<Test>::get();
        let root_tao_before: u64 = SubnetTAO::<Test>::get(0);
        let subnet_tao_before: u64 = SubnetTAO::<Test>::get(netuid);
        let alpha_out_before: u64 = SubnetAlphaOut::<Test>::get(netuid);
        let alpha_in_before: u64 = SubnetAlphaIn::<Test>::get(netuid);
        let previous_pending: u64 = PendingEmission::<Test>::get(netuid);
        step_block(1);
        // Single subnet gets 100% of block emission.
        assert_eq!(EmissionValues::<Test>::get(netuid), block_emission);
        // Alpha_in emission = 1 because alpha_price < emission_value/block_emission
        assert_eq!(SubnetAlphaInEmission::<Test>::get(netuid), block_emission);
        // Get the pending root dividends.
        let pending_root_divs: u64 = PendingRootDivs::<Test>::get(netuid);
        // Simualte the swap back to alpha.
        let pending_as_alpha: u64 = SubtensorModule::sim_swap_tao_for_alpha( netuid, pending_root_divs );
        // Subnet ALpha out Emission .
        assert_eq!(SubnetAlphaOutEmission::<Test>::get(netuid), block_emission);
        // Assert difference is Pending emission.
        assert_eq!(PendingEmission::<Test>::get(netuid), previous_pending + (block_emission - pending_as_alpha) - 6 ); // All alpha with rounding.
        // Alpha in is increased by emission.
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid), alpha_in_before + block_emission + pending_as_alpha + 5 ); // Corret with rounding.

        // Lets train the pending.
        let total_issuance_before: u64 = TotalIssuance::<Test>::get();
        let total_stake_before: u64 = TotalStake::<Test>::get();
        let root_tao_before: u64 = SubnetTAO::<Test>::get(0);
        let subnet_tao_before: u64 = SubnetTAO::<Test>::get(netuid);
        let alpha_out_before: u64 = SubnetAlphaOut::<Test>::get(netuid);
        let alpha_in_before: u64 = SubnetAlphaIn::<Test>::get(netuid);
        let previous_pending: u64 = PendingEmission::<Test>::get(netuid);
        let root_pending: u64 = PendingRootDivs::<Test>::get(netuid);
        SubtensorModule::drain_pending_emission( netuid );
        // Drained
        assert_eq!(PendingEmission::<Test>::get(netuid), 0);
        // Unchanged.
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid), alpha_in_before);
        // Dividends are all pending to this hotkey.
        assert_eq!(HotkeyDividendsPerSubnet::<Test>::get(netuid, &hotkey), previous_pending);
        // Root Dividends are all pending to this hotkey.
        assert_eq!(RootDividendsPerSubnet::<Test>::get(netuid, &hotkey), root_pending);

        // TODO( const ): test multiple subnets.
        // TODO( const ): test multi root proportions
        // TODO( const ): test parents etc
        

    });
}
