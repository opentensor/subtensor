mod mock;
use mock::*;
use sp_core::U256;
// use pallet_subtensor::*;

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_create_network --exact --nocapture
#[test]
fn test_create_network() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000);
        let netuid = create_network(coldkey, hotkey, 1);
        assert_eq!(
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, 100_000_000_000),
            500_000_000
        ); // With huge slippage because of the initial price.
    });
}