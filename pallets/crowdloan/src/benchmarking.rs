//! Benchmarks for Crowdloan Pallet
#![cfg(feature = "runtime-benchmarks")]
use crate::{BalanceOf, CrowdloanId, CrowdloanInfo, CurrencyOf, pallet::*};
use frame_benchmarking::{account, v2::*};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;

extern crate alloc;

const SEED: u32 = 0;

use alloc::{boxed::Box, vec};

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = frame_system::Pallet::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    // compare to the last event record
    let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

#[benchmarks]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn create() {
        let creator: T::AccountId = whitelisted_caller();
        let deposit = T::MinimumDeposit::get();
        let cap = deposit + deposit;
        let now = frame_system::Pallet::<T>::block_number();
        let end = now + T::MaximumBlockDuration::get();
        let target_address = account::<T::AccountId>("target_address", 0, SEED);
        let call: Box<<T as Config>::RuntimeCall> =
            Box::new(frame_system::Call::<T>::remark { remark: vec![] }.into());
        let _ = CurrencyOf::<T>::make_free_balance_be(&creator, deposit);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(creator.clone()),
            deposit,
            cap,
            end,
            target_address.clone(),
            call.clone(),
        );

        // ensure the crowdloan is stored correctly
        let crowdloan_id = 0;
        assert_eq!(
            Crowdloans::<T>::get(crowdloan_id),
            Some(CrowdloanInfo {
                creator: creator.clone(),
                deposit,
                cap,
                end,
                raised: deposit,
                target_address,
                call,
                finalized: false,
            })
        );
        // ensure the creator has been deducted the deposit
        assert!(CurrencyOf::<T>::free_balance(&creator).is_zero());
        // ensure the initial deposit is stored correctly as contribution
        assert_eq!(
            Contributions::<T>::get(crowdloan_id, &creator),
            Some(deposit)
        );
        // ensure the raised amount is updated correctly
        assert!(Crowdloans::<T>::get(crowdloan_id).is_some_and(|c| c.raised == deposit));
        // ensure the crowdloan account has the deposit
        assert_eq!(
            CurrencyOf::<T>::free_balance(&Pallet::<T>::crowdloan_account_id(crowdloan_id)),
            deposit
        );
        // ensure the event is emitted
        assert_last_event::<T>(
            Event::<T>::Created {
                crowdloan_id,
                creator,
                end,
                cap,
            }
            .into(),
        );
        // ensure next crowdloan id is incremented
        assert_eq!(NextCrowdloanId::<T>::get(), crowdloan_id + 1);
    }

    #[benchmark]
    fn contribute() {
        // create a crowdloan
        let creator: T::AccountId = whitelisted_caller();
        let deposit = T::MinimumDeposit::get();
        let cap = deposit + deposit;
        let now = frame_system::Pallet::<T>::block_number();
        let end = now + T::MaximumBlockDuration::get();
        let target_address: T::AccountId = account::<T::AccountId>("target_address", 0, SEED);
        let call: Box<<T as Config>::RuntimeCall> =
            Box::new(frame_system::Call::<T>::remark { remark: vec![] }.into());
        let _ = CurrencyOf::<T>::make_free_balance_be(&creator, deposit);
        let _ = Pallet::<T>::create(
            RawOrigin::Signed(creator.clone()).into(),
            deposit,
            cap,
            end,
            target_address.clone(),
            call.clone(),
        );

        // setup contributor
        let contributor: T::AccountId = account::<T::AccountId>("contributor", 0, SEED);
        let amount: BalanceOf<T> = T::MinimumContribution::get();
        let crowdloan_id: CrowdloanId = 0;
        let _ = CurrencyOf::<T>::make_free_balance_be(&contributor, amount);

        #[extrinsic_call]
        _(RawOrigin::Signed(contributor.clone()), crowdloan_id, amount);

        // ensure the contribution is stored correctly
        assert_eq!(
            Contributions::<T>::get(crowdloan_id, &contributor),
            Some(amount)
        );
        // ensure the contributor has been deducted the amount
        assert!(CurrencyOf::<T>::free_balance(&contributor).is_zero());
        // ensure the crowdloan raised amount is updated correctly
        assert!(Crowdloans::<T>::get(crowdloan_id).is_some_and(|c| c.raised == deposit + amount));
        // ensure the event is emitted
        assert_last_event::<T>(
            Event::<T>::Contributed {
                contributor,
                crowdloan_id,
                amount,
            }
            .into(),
        );
        // ensure the contribution is present in the crowdloan account
        assert_eq!(
            CurrencyOf::<T>::free_balance(&Pallet::<T>::crowdloan_account_id(crowdloan_id)),
            deposit + amount
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
