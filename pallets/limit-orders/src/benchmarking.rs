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
use sp_runtime::{AccountId32, MultiSignature, Perbill, traits::AccountIdConversion};
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
    use frame_support::traits::Get;
    use subtensor_swap_interface::OrderSwapInterface;

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

    /// Worst case: `n` orders each with a distinct signer (coldkey/hotkey) and a
    /// distinct fee recipient, maximising per-order storage reads and fee transfers.
    #[benchmark]
    fn execute_orders(n: Linear<1, { T::MaxOrdersPerBatch::get() }>) {
        let netuid = NetUid::from(1u16);
        let mut orders = alloc::vec::Vec::new();

        for i in 0..n {
            // Derive a unique sr25519 keypair for each order so every order
            // hits a different storage slot (different signer balance reads).
            let pair =
                sp_core::sr25519::Pair::from_string(&alloc::format!("//Signer{}", i), None)
                    .unwrap();
            let account: T::AccountId = AccountId32::from(pair.public()).into();
            let fee_recipient: T::AccountId = frame_benchmarking::account("fee_recipient", i, 0);

            // Allow the swap implementation to fund/register this account.
            T::SwapInterface::set_up_acc_for_benchmark(&account, &account);

            let order = crate::Order {
                signer: account.clone(),
                hotkey: account.clone(),
                netuid,
                order_type: OrderType::LimitBuy,
                amount: 1_000_000_000u64,
                limit_price: u64::MAX, // always satisfied for a buy
                expiry: u64::MAX,
                fee_rate: Perbill::from_percent(1),
                fee_recipient,
            };
            let sig = pair.sign(&order.encode());
            orders.push(crate::SignedOrder {
                order,
                signature: MultiSignature::Sr25519(sig),
            });
        }

        let bounded_orders: frame_support::BoundedVec<_, T::MaxOrdersPerBatch> =
            frame_support::BoundedVec::try_from(orders).unwrap();
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), bounded_orders);
    }

    /// Worst case: `n` buy orders each with a distinct signer and fee recipient,
    /// maximising asset-collection reads, pro-rata distribution writes, and the
    /// number of unique fee-transfer recipients in `collect_fees`.
    #[benchmark]
    fn execute_batched_orders(n: Linear<1, { T::MaxOrdersPerBatch::get() }>) {
        let netuid = NetUid::from(1u16);

        // Set up the pallet intermediary so the net pool swap and alpha
        // distribution transfers succeed.
        let pallet_acct: T::AccountId = T::PalletId::get().into_account_truncating();
        let pallet_hotkey: T::AccountId = T::PalletHotkey::get();
        T::SwapInterface::set_up_acc_for_benchmark(&pallet_hotkey, &pallet_acct);

        let mut orders = alloc::vec::Vec::new();

        for i in 0..n {
            let pair =
                sp_core::sr25519::Pair::from_string(&alloc::format!("//Signer{}", i), None)
                    .unwrap();
            let account: T::AccountId = AccountId32::from(pair.public()).into();
            let fee_recipient: T::AccountId = frame_benchmarking::account("fee_recipient", i, 0);

            T::SwapInterface::set_up_acc_for_benchmark(&account, &account);

            let order = crate::Order {
                signer: account.clone(),
                hotkey: account.clone(),
                netuid,
                order_type: OrderType::LimitBuy,
                amount: 1_000_000_000u64,
                limit_price: u64::MAX,
                expiry: u64::MAX,
                fee_rate: Perbill::from_percent(1),
                fee_recipient,
            };
            let sig = pair.sign(&order.encode());
            orders.push(crate::SignedOrder {
                order,
                signature: MultiSignature::Sr25519(sig),
            });
        }

        let bounded_orders: frame_support::BoundedVec<_, T::MaxOrdersPerBatch> =
            frame_support::BoundedVec::try_from(orders).unwrap();
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), netuid, bounded_orders);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::tests::mock::new_test_ext(),
        crate::tests::mock::Test
    );
}
