use crate::tests::mock::{RuntimeOrigin, SubtensorModule, Test, add_dynamic_network, new_test_ext};
use crate::{RootClaimType, RootClaimTypeEnum, SubnetAlphaIn, SubnetTAO, pallet};
use crate::{RootClaimable, TotalHotkeyAlpha};
use approx::assert_abs_diff_eq;
use frame_support::assert_ok;
use sp_core::U256;
use substrate_fixed::types::{I96F32, U96F32};
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
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
fn test_claim_root_with_drain_emissions() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        let initial_balance = 10_000_000u64;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_balance.into());

        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        assert_eq!(current_price, U96F32::from_num(0.5));

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let stake = 2_000_000u64;
        let initial_total_hotkey_alpha = 1_000_000u64;

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        let old_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        assert_eq!(old_validator_stake, initial_total_hotkey_alpha.into());

        // Distribute pending root alpha

        let pending_root_alpha = AlphaCurrency::from(10_000_000);
        SubtensorModule::drain_pending_emission(
            netuid,
            AlphaCurrency::ZERO,
            pending_root_alpha,
            AlphaCurrency::ZERO,
        );

        // Check new validator stake
        let validator_take_percent = I96F32::from(18u64) / I96F32::from(100u64);

        let new_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        let calculated_validator_stake = I96F32::from(u64::from(pending_root_alpha))
            * I96F32::from(initial_total_hotkey_alpha)
            * validator_take_percent
            / I96F32::from(u64::from(initial_total_hotkey_alpha))
            + I96F32::from(initial_total_hotkey_alpha);

        assert_abs_diff_eq!(
            u64::from(new_validator_stake),
            calculated_validator_stake.saturating_to_num::<u64>(),
            epsilon = 100u64,
        );

        let claimable = RootClaimable::<Test>::get(hotkey, netuid);
        let calculated_rate = (I96F32::from(u64::from(pending_root_alpha))
            * (I96F32::from(1u64) - validator_take_percent))
            / I96F32::from(u64::from(TotalHotkeyAlpha::<Test>::get(hotkey, netuid)));

        assert_abs_diff_eq!(
            (claimable * I96F32::from(1000u64)).saturating_to_num::<u64>(),
            (calculated_rate * I96F32::from(1000u64)).saturating_to_num::<u64>(),
            epsilon = 10u64,
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);

        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(coldkey),));

        log::debug!("RootClaimable = {}", claimable);
        let new_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        assert_abs_diff_eq!(
            u64::from(new_stake),
            (I96F32::from(stake) * claimable).saturating_to_num::<u64>(),
            epsilon = 10u64,
        );
    });
}

/*
Test 1 - Adding Stake Disproportionally

1. Beginning of the epoch 1: Alice stakes 1 TAO to root, Bob stakes 1 TAO to root, and both say "I want airdrop"
2. For simplicity let's say `root_alpha = 0.1` in every block. So at the end of the epoch 1 we have 36 root alpha to share between Alice and Bob. Each is entitled to 18 Alpha.
3. Beginning of the epoch 2: Alice stakes 1 TAO more, Bob stakes 2 TAO more. State is now:

```
alpha_to_airdrop: 36
rewards_per_tao: 18

Alice's stake: 2
Bob's stake: 3
debt_alice: 18
debt_bob: 36
```

4. As epoch 2 goes, we keep adding 0.1 root Alpha in every block, so it will be still +36 Alpha, but new Alpha is shared differently between Alice and Bob: Alice gets 2/5 of the new Alpha and Bob gets 3/5 of the new Alpha.

5. End of epoch 2: Alice is entitled to 18 + 14.4 = 32.4 Alpha, Bob is entitled to 18 + 21.6 = 39.6 Alpha. State update is following:

```
alpha_to_airdrop: 72
rewards_per_tao: 18 + 36/5 = 25.2
```

6. Alice and Bob want to claim their Alpha:

```
alpha_alice = 2 * 25.2 - 18 = 32.4
alpha_bob = 3 * 25.2 - 36 = 39.6
```

*/

#[test]
fn test_adding_stake_disproportionally() {
    new_test_ext(1).execute_with(|| {});
}
