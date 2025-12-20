use crate::tests::mock::*;
use crate::*;
use frame_support::traits::Currency;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, TaoCurrency};

#[test]
fn test_imbalance_conservation_burn_mint() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        // Setup initial flows to ensure non-zero emission logic runs
        SubnetTaoFlow::<Test>::insert(netuid, 100_000_000_i64);
        SubnetMechanism::<Test>::insert(netuid, 1);
        
        // Ensure subnets to emit to calculates correctly
        let subnets = SubtensorModule::get_all_subnet_netuids();
        let to_emit = SubtensorModule::get_subnets_to_emit_to(&subnets);
        assert!(to_emit.contains(&netuid));

        let emission_u64: u64 = 1_000_000;
        let emission = U96F32::from_num(emission_u64);

        // Capture initial state
        let initial_balances_issuance = Balances::total_issuance();
        let initial_subtensor_issuance = TotalIssuance::<Test>::get();
        let initial_stake = TotalStake::<Test>::get();

        log::info!("Initial Balances Issuance: {:?}", initial_balances_issuance);
        log::info!("Initial Subtensor Issuance: {:?}", initial_subtensor_issuance);
        log::info!("Initial Stake: {:?}", initial_stake);

        // Run Coinbase
        // This should:
        // 1. Issue Imbalance (Balances::TotalIssuance + 1M)
        // 2. Split Imbalance
        // 3. Drop/Burn Imbalance (Balances::TotalIssuance - 1M)
        // 4. Update Subtensor::TotalIssuance (+1M)
        // 5. Update TotalStake (+1M)
        
        SubtensorModule::run_coinbase(emission);

        let final_balances_issuance = Balances::total_issuance();
        let final_subtensor_issuance = TotalIssuance::<Test>::get();
        let final_stake = TotalStake::<Test>::get();

        log::info!("Final Balances Issuance: {:?}", final_balances_issuance);
        log::info!("Final Subtensor Issuance: {:?}", final_subtensor_issuance);
        log::info!("Final Stake: {:?}", final_stake);

        // CHECK 1: Real balances logic (Imbalance usage)
        // Since we drop/burn the imbalance, the real issuance should return to original.
        assert_eq!(
            initial_balances_issuance, final_balances_issuance,
            "Real Balance Issuance should be unchanged (Imbalance Mint -> Drop/Burn pattern)"
        );

        // CHECK 2: Virtual accounting logic
        assert_eq!(
            final_subtensor_issuance,
            initial_subtensor_issuance + TaoCurrency::from(emission_u64),
            "Virtual Subtensor Issuance should increase by emission"
        );

        assert_eq!(
            final_stake,
            initial_stake + TaoCurrency::from(emission_u64),
            "Virtual Total Stake should increase by emission"
        );
    });
}
