use crate::TotalHotkeyAlpha;
use crate::tests::mock::{RuntimeOrigin, SubtensorModule, Test, add_dynamic_network, new_test_ext};
use crate::{RootClaimType, RootClaimTypeEnum, SubnetAlphaIn, SubnetTAO, pallet};
use frame_support::assert_ok;
use sp_core::U256;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};
use subtensor_swap_interface::SwapHandler;

#[test]
fn test_set_root_claim_type() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);

        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);
    });
}

#[test]
fn test_claim_root() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        assert_eq!(current_price, U96F32::from_num(0.5));

        let stake = 1_000_000u64;
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, AlphaCurrency::from(stake));
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            stake.into(),
        );

        let claimable_amount = 1_000_000u64;
        SubtensorModule::increase_root_claimable_for_hotkey_and_subnet(
            &hotkey,
            netuid,
            claimable_amount.into(),
        );

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);

        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);

        let new_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(new_stake, 0u64.into());

        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(coldkey),));

        let new_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(new_stake, (stake * 2).into());
    });
}
