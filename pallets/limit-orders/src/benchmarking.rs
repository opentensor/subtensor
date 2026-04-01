//! Benchmarks for Limit Orders Pallet
#![cfg(feature = "runtime-benchmarks")]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]
use crate::{NetUid, OrderType, Orders};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::{H256, Pair};
use sp_keyring::Sr25519Keyring as AccountKeyring;
use sp_runtime::{MultiSignature, Perbill};
extern crate alloc;
use crate::{Call, Config, Pallet};
use codec::Encode;

pub fn make_signed_order<T: crate::Config>(
    keyring: AccountKeyring,
    hotkey: T::AccountId,
    netuid: NetUid,
    order_type: crate::OrderType,
    amount: u64,
    limit_price: u64,
    expiry: u64,
    fee_rate: sp_runtime::Perbill,
    fee_recipient: T::AccountId,
) -> crate::SignedOrder<T::AccountId> {
    let signer = keyring.to_account_id();
    let order = crate::Order {
        signer,
        hotkey: hotkey.into(),
        netuid,
        order_type,
        amount,
        limit_price,
        expiry,
        fee_rate,
        fee_recipient: fee_recipient.into(),
    };
    let sig = keyring.pair().sign(&order.encode());
    crate::SignedOrder {
        order,
        signature: MultiSignature::Sr25519(sig),
    }
}

pub fn order_id<T: crate::Config>(order: &crate::Order<T::AccountId>) -> H256 {
    crate::pallet::Pallet::<T>::derive_order_id(order)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn cancel_order() {
        let signed = make_signed_order::<T>(
            AccountKeyring::Alice,
            AccountKeyring::Alice.to_account_id().into(),
            NetUid::from(1u16),
            OrderType::LimitBuy,
            1_000,
            2_000_000_000,
            1_000_000_000,
            Perbill::zero(),
            AccountKeyring::Alice.to_account_id().into(),
        );

        #[extrinsic_call]
        _(
            RawOrigin::Signed(AccountKeyring::Alice.to_account_id()),
            signed.order.clone(),
        );
        let id = order_id::<T>(&signed.order);

        assert_eq!(Orders::<T>::get(id), Some(crate::OrderStatus::Cancelled));
    }

    #[benchmark]
    fn set_pallet_status() {
        #[extrinsic_call]
        _(RawOrigin::Root, false);

        assert_eq!(crate::LimitOrdersEnabled::<T>::get(), false);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::tests::mock::new_test_ext(),
        crate::tests::mock::Test
    );
}
