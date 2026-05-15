//! Benchmarking setup
#![cfg(feature = "runtime-benchmarks")]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::unwrap_used
)]
use super::*;

#[allow(unused)]
use crate::Pallet as Registry;
use frame_benchmarking::v2::*;
use frame_support::traits::{Get, tokens::fungible::Mutate};
use frame_system::RawOrigin;
use sp_std::vec;

fn assert_last_event<T: frame_system::pallet::Config>(
    generic_event: <T as frame_system::pallet::Config>::RuntimeEvent,
) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

// This creates an `IdentityInfo` object with `num_fields` extra fields.
// All data is pre-populated with some arbitrary bytes.
fn create_identity_info<T: Config>(_num_fields: u32) -> IdentityInfo<T::MaxAdditionalFields> {
    let data = Data::Raw(
        vec![0; 32]
            .try_into()
            .expect("size does not exceed 64; qed"),
    );

    IdentityInfo {
        additional: Default::default(),
        display: data.clone(),
        legal: data.clone(),
        web: data.clone(),
        riot: data.clone(),
        email: data.clone(),
        pgp_fingerprint: Some([0; 20]),
        image: data.clone(),
        twitter: data,
    }
}

#[benchmarks(where BalanceOf<T>: From<u64>)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_identity() {
        // The target user
        let caller: T::AccountId = whitelisted_caller();
        let deposit = T::InitialDeposit::get() * 10u64.into();
        let _ = T::Currency::set_balance(&caller, deposit);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone()),
            caller.clone(),
            Box::new(create_identity_info::<T>(0)),
        );

        assert_last_event::<T>(Event::<T>::IdentitySet { who: caller }.into());
    }

    #[benchmark]
    fn clear_identity() {
        // The target user
        let caller: T::AccountId = whitelisted_caller();
        let _ = T::Currency::set_balance(&caller, T::InitialDeposit::get() * 10u64.into());

        Registry::<T>::set_identity(
            RawOrigin::Signed(caller.clone()).into(),
            caller.clone(),
            Box::new(create_identity_info::<T>(0)),
        )
        .unwrap();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), caller.clone());

        assert_last_event::<T>(Event::<T>::IdentityDissolved { who: caller }.into());
    }

    impl_benchmark_test_suite!(Registry, crate::mock::new_test_ext(), crate::mock::Test);
}
