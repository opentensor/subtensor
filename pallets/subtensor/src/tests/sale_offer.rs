#![allow(clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use subtensor_runtime_common::{NetUid, TaoBalance};

const SELLER: u64 = 1;
const OWNER_HOTKEY: u64 = 2;
const BUYER: u64 = 3;

fn sale_fixture() -> (NetUid, U256, U256, U256) {
    let seller = U256::from(SELLER);
    let owner_hotkey = U256::from(OWNER_HOTKEY);
    let buyer = U256::from(BUYER);
    let netuid = add_dynamic_network(&owner_hotkey, &seller);

    (netuid, seller, owner_hotkey, buyer)
}

fn create_offer(netuid: NetUid, seller: U256, buyer: Option<U256>) {
    assert_ok!(SubtensorModule::create_sale_offer(
        RuntimeOrigin::signed(seller),
        netuid,
        TaoBalance::from(1_000_000_000_u64),
        buyer,
    ));
}

#[test]
fn create_sale_offer_stores_offer_and_freezes_keys() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, owner_hotkey, buyer) = sale_fixture();
        let price = TaoBalance::from(1_000_000_000_u64);

        assert_ok!(SubtensorModule::create_sale_offer(
            RuntimeOrigin::signed(seller),
            netuid,
            price,
            Some(buyer),
        ));

        let offer = SubnetSaleOffers::<Test>::get(netuid).unwrap();
        assert_eq!(offer.netuid, netuid);
        assert_eq!(offer.seller, seller);
        assert_eq!(offer.authorized_buyer, Some(buyer));
        assert_eq!(offer.price, price);
        assert_eq!(offer.created_at, System::block_number());
        assert!(SubnetSaleFrozenColdkeys::<Test>::contains_key(seller));
        assert!(SubnetSaleFrozenHotkeys::<Test>::contains_key(owner_hotkey));
        assert_eq!(
            last_event(),
            RuntimeEvent::SubtensorModule(Event::SubnetSaleOfferCreated {
                seller,
                netuid,
                price,
                authorized_buyer: Some(buyer),
            })
        );
    });
}

#[test]
fn create_sale_offer_allows_open_offer() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, _owner_hotkey, _buyer) = sale_fixture();

        assert_ok!(SubtensorModule::create_sale_offer(
            RuntimeOrigin::signed(seller),
            netuid,
            TaoBalance::from(1_000_000_000_u64),
            None,
        ));

        let offer = SubnetSaleOffers::<Test>::get(netuid).unwrap();
        assert_eq!(offer.authorized_buyer, None);
    });
}

#[test]
fn create_sale_offer_rejects_zero_price() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, _owner_hotkey, buyer) = sale_fixture();

        assert_noop!(
            SubtensorModule::create_sale_offer(
                RuntimeOrigin::signed(seller),
                netuid,
                TaoBalance::from(0_u64),
                Some(buyer),
            ),
            Error::<Test>::AmountTooLow,
        );
    });
}

#[test]
fn create_sale_offer_rejects_missing_subnet() {
    new_test_ext(1).execute_with(|| {
        let seller = U256::from(SELLER);
        let missing_netuid = NetUid::from(99);

        assert_noop!(
            SubtensorModule::create_sale_offer(
                RuntimeOrigin::signed(seller),
                missing_netuid,
                TaoBalance::from(1_000_000_000_u64),
                None,
            ),
            Error::<Test>::SubnetNotExists,
        );
    });
}

#[test]
fn create_sale_offer_rejects_non_owner() {
    new_test_ext(1).execute_with(|| {
        let (netuid, _seller, _owner_hotkey, buyer) = sale_fixture();
        let not_owner = U256::from(99);

        assert_noop!(
            SubtensorModule::create_sale_offer(
                RuntimeOrigin::signed(not_owner),
                netuid,
                TaoBalance::from(1_000_000_000_u64),
                Some(buyer),
            ),
            Error::<Test>::NotSubnetOwner,
        );
    });
}

#[test]
fn create_sale_offer_rejects_existing_offer() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, _owner_hotkey, buyer) = sale_fixture();
        create_offer(netuid, seller, Some(buyer));

        assert_noop!(
            SubtensorModule::create_sale_offer(
                RuntimeOrigin::signed(seller),
                netuid,
                TaoBalance::from(2_000_000_000_u64),
                None,
            ),
            Error::<Test>::SaleOfferAlreadyExists,
        );
    });
}

#[test]
fn create_sale_offer_rejects_frozen_seller() {
    new_test_ext(1).execute_with(|| {
        let (first_netuid, seller, _first_owner_hotkey, buyer) = sale_fixture();
        let second_owner_hotkey = U256::from(20);
        let second_netuid = add_dynamic_network(&second_owner_hotkey, &seller);
        create_offer(first_netuid, seller, Some(buyer));

        assert_noop!(
            SubtensorModule::create_sale_offer(
                RuntimeOrigin::signed(seller),
                second_netuid,
                TaoBalance::from(2_000_000_000_u64),
                None,
            ),
            Error::<Test>::ColdkeyLockedDuringSale,
        );
    });
}

#[test]
fn create_sale_offer_rejects_missing_owner_hotkey() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, _owner_hotkey, buyer) = sale_fixture();
        SubnetOwnerHotkey::<Test>::remove(netuid);

        assert_noop!(
            SubtensorModule::create_sale_offer(
                RuntimeOrigin::signed(seller),
                netuid,
                TaoBalance::from(1_000_000_000_u64),
                Some(buyer),
            ),
            Error::<Test>::HotKeyAccountNotExists,
        );
    });
}

#[test]
fn create_sale_offer_rejects_frozen_owner_hotkey() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, owner_hotkey, buyer) = sale_fixture();
        SubnetSaleFrozenHotkeys::<Test>::insert(owner_hotkey, ());

        assert_noop!(
            SubtensorModule::create_sale_offer(
                RuntimeOrigin::signed(seller),
                netuid,
                TaoBalance::from(1_000_000_000_u64),
                Some(buyer),
            ),
            Error::<Test>::HotkeyLockedDuringSale,
        );
    });
}

#[test]
fn cancel_sale_offer_unfreezes_keys() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, owner_hotkey, buyer) = sale_fixture();
        create_offer(netuid, seller, Some(buyer));

        assert_ok!(SubtensorModule::cancel_sale_offer(
            RuntimeOrigin::signed(seller),
            netuid,
        ));

        assert!(!SubnetSaleOffers::<Test>::contains_key(netuid));
        assert!(!SubnetSaleFrozenColdkeys::<Test>::contains_key(seller));
        assert!(!SubnetSaleFrozenHotkeys::<Test>::contains_key(owner_hotkey));
        assert_eq!(
            last_event(),
            RuntimeEvent::SubtensorModule(Event::SubnetSaleOfferCancelled { seller, netuid })
        );
    });
}

#[test]
fn cancel_sale_offer_root_unfreezes_keys() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, owner_hotkey, buyer) = sale_fixture();
        create_offer(netuid, seller, Some(buyer));

        assert_ok!(SubtensorModule::cancel_sale_offer(
            RuntimeOrigin::root(),
            netuid,
        ));

        assert!(!SubnetSaleOffers::<Test>::contains_key(netuid));
        assert!(!SubnetSaleFrozenColdkeys::<Test>::contains_key(seller));
        assert!(!SubnetSaleFrozenHotkeys::<Test>::contains_key(owner_hotkey));
        assert_eq!(
            last_event(),
            RuntimeEvent::SubtensorModule(Event::SubnetSaleOfferCancelled { seller, netuid })
        );
    });
}

#[test]
fn cancel_sale_offer_rejects_missing_offer() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, _owner_hotkey, _buyer) = sale_fixture();

        assert_noop!(
            SubtensorModule::cancel_sale_offer(RuntimeOrigin::signed(seller), netuid),
            Error::<Test>::SaleOfferNotFound,
        );
    });
}

#[test]
fn cancel_sale_offer_rejects_non_seller() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, _owner_hotkey, buyer) = sale_fixture();
        let not_seller = U256::from(99);
        create_offer(netuid, seller, Some(buyer));

        assert_noop!(
            SubtensorModule::cancel_sale_offer(RuntimeOrigin::signed(not_seller), netuid),
            Error::<Test>::NotSubnetOwner,
        );
    });
}

#[test]
fn remove_network_cleans_sale_offer() {
    new_test_ext(1).execute_with(|| {
        let (netuid, seller, owner_hotkey, buyer) = sale_fixture();
        create_offer(netuid, seller, Some(buyer));

        assert_ok!(SubtensorModule::root_dissolve_network(
            RuntimeOrigin::root(),
            netuid,
        ));

        assert!(!SubnetSaleOffers::<Test>::contains_key(netuid));
        assert!(!SubnetSaleFrozenColdkeys::<Test>::contains_key(seller));
        assert!(!SubnetSaleFrozenHotkeys::<Test>::contains_key(owner_hotkey));
    });
}
