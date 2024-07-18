mod mock;
use mock::*;
use sp_core::U256;
use pallet_subtensor::*;

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test alpha -- test_create_stao_subnet --exact --nocapture
#[test]
fn test_create_stao_subnet() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000_000);

        let netuid_a = register_network(coldkey, hotkey, 0); // Mechanism = 0 is STAO
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_a) ), 100_000_000_000); // with transfer fee.
        assert_eq!(SubtensorModule::alpha_to_dynamic(100_000_000_000, netuid_a), 100_000_000_000);

        let netuid_b = register_network(coldkey, hotkey, 1); // Mechanism = 1 is STAO
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_b) ), 200_000_000_000); // with transfer fee.
        assert_eq!(SubtensorModule::alpha_to_dynamic(200_000_000_000, netuid_b), 200_000_000_000);

        let netuid_c = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_c) ), 400_000_000_000); // with transfer fee.
        assert_eq!(SubtensorModule::alpha_to_dynamic(400_000_000_000, netuid_c), 400_000_000_000);

        let netuid_d = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_d) ), 1_200_000_000_000); // with alpha conversion.
        assert_eq!(SubtensorModule::alpha_to_dynamic(1_200_000_000_000, netuid_d), 800_000_000_000); // Converts to the entire lock cost value. 

        // let netuid_e = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        // assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_e) ), 2_799_999_999_972); // with alpha conversion.
        // assert_eq!(pallet_subtensor::SubnetTAO::<Test>::get( netuid_e ), 16_000_000_000_000); // with alpha conversion.
        // assert_eq!(pallet_subtensor::SubnetAlpha::<Test>::get( netuid_e ), 2_799_999_999_972); // with alpha conversion.

        // assert_eq!(SubtensorModule::alpha_to_dynamic(2_799_999_999_972, netuid_e), 1_599_999_999_984); // Major slippage.

        // let total_alpha: I96F32 = I96F32::from_num( SubnetAlpha::<T>::get( netuid ) );
        // let total_tao: I96F32 = I96F32::from_num( SubnetTAO::<T>::get( netuid ) );
        // // Calculate the tao equivalent for the given alpha value.
        // let tao_equivalent: I96F32 = (I96F32::from_num(alpha).checked_div(total_alpha).unwrap_or(I96F32::from_num(0.0))) * total_tao;


        // SubtensorModule::add_balance_to_coldkey_account(&coldkey, 32_000_000_000_000);
        // let netuid_f = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        // assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_f) ), 5_999_999_999_940); // with alpha conversion.
        // assert_eq!(SubtensorModule::alpha_to_dynamic(5_999_999_999_940, netuid_f), 3_199_999_999_968); // Major slippage.

    });
}
