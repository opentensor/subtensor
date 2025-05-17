use crate::MaxChildkeyTake;

use super::mock::*;
use frame_support::{StorageHasher, Twox128, assert_ok, traits::Currency};
use frame_system::Config;
use sp_core::U256;
use sp_io::hashing::twox_128;
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::batch_tx::test_batch_txs --exact --show-output --nocapture
#[test]
fn test_batch_txs() {
    let alice = U256::from(0);
    let bob = U256::from(1);
    let charlie = U256::from(2);
    let initial_balances = vec![
        (alice, 8_000_000_000),
        (bob, 1_000_000_000),
        (charlie, 1_000_000_000),
    ];
    test_ext_with_balances(initial_balances).execute_with(|| {
        assert_ok!(Utility::batch(
            <<Test as Config>::RuntimeOrigin>::signed(alice),
            vec![
                RuntimeCall::Balances(BalanceCall::transfer_allow_death {
                    dest: bob,
                    value: 1_000_000_000
                }),
                RuntimeCall::Balances(BalanceCall::transfer_allow_death {
                    dest: charlie,
                    value: 1_000_000_000
                })
            ]
        ));
        assert_eq!(Balances::total_balance(&alice), 6_000_000_000);
        assert_eq!(Balances::total_balance(&bob), 2_000_000_000);
        assert_eq!(Balances::total_balance(&charlie), 2_000_000_000);
    });
}

// [101, 143, 170, 56, 80, 112, 224, 116, 200, 91, 246, 181, 104, 207, 5, 85, 219, 160, 24, 133, 156, 171, 126, 152, 159, 119, 102, 148, 87, 179, 148, 190]
#[test]
fn test_batch_txs2() {
    let alice = U256::from(0);
    let bob = U256::from(1);
    let charlie = U256::from(2);
    let initial_balances = vec![
        (alice, 8_000_000_000),
        (bob, 1_000_000_000),
        (charlie, 1_000_000_000),
    ];
    test_ext_with_balances(initial_balances).execute_with(|| {
        let init_value = SubtensorModule::get_max_childkey_take();
        log::error!("Storage value: {:?}", init_value);

        SubtensorModule::set_max_childkey_take(1000);

        // let mut final_key = [0u8; 32];
        // final_key[16..].copy_from_slice(twox_128(b"Subtensor"));
        // final_key[..16].copy_from_slice(twox_128(b"MaxChildkeyTake"));
        let final_key = [twox_128(b"SubtensorModule"), twox_128(b"MaxChildkeyTake")].concat();
        log::error!("Storage value: {:?}", final_key);

        let hash_key = MaxChildkeyTake::<Test>::hashed_key();
        log::error!("Storage value: {:?}", hash_key);
        // let key = [
        //     101, 143, 170, 56, 80, 112, 224, 116, 200, 91, 246, 181, 104, 207, 5, 85, 219, 160, 24,
        //     133, 156, 171, 126, 152, 159, 119, 102, 148, 87, 179, 148, 190,
        // ];
        let value = sp_io::storage::get(&final_key[..]);
        log::error!("Storage value: {:?}", value);
        assert_eq!(value, None);

        sp_io::storage::set(&final_key[..], &[0u8; 32][..]);
        let init_value = SubtensorModule::get_max_childkey_take();
        log::error!("Storage value: {:?}", init_value);
    });
}
