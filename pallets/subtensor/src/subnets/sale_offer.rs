//! Subnet sale offers and sale-time freezes.
//!
//! This module intentionally only owns the seller-side primitive: listing a subnet
//! for sale freezes the seller coldkey, subnet, and owner hotkey until the offer is
//! cancelled or later consumed by a sale finalization path.

use super::*;
use frame_support::traits::fungible;
use subtensor_runtime_common::{NetUid, TaoBalance};

pub type CurrencyOf<T> = <T as Config>::Currency;

pub type BalanceOf<T> =
    <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[freeze_struct("7e8727dabcea5026")]
#[derive(Encode, Decode, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SubnetSaleOffer<AccountId, Balance> {
    /// The subnet being sold.
    pub netuid: NetUid,
    /// The subnet owner coldkey that created the offer.
    pub seller: AccountId,
    /// Optional coldkey that is allowed to consume this offer.
    pub authorized_buyer: Option<AccountId>,
    /// Sale price expected by the seller.
    pub price: Balance,
}

pub type SubnetSaleOfferOf<T> = SubnetSaleOffer<AccountIdOf<T>, BalanceOf<T>>;

impl<T: Config> Pallet<T> {
    pub fn do_create_sale_offer(
        seller: T::AccountId,
        netuid: NetUid,
        price: TaoBalance,
        authorized_buyer: Option<T::AccountId>,
    ) -> DispatchResult {
        ensure!(price > TaoBalance::from(0_u64), Error::<T>::AmountTooLow);
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        ensure!(
            SubnetOwner::<T>::get(netuid) == seller,
            Error::<T>::NotSubnetOwner
        );
        ensure!(
            !SubnetSaleOffers::<T>::contains_key(netuid),
            Error::<T>::SaleOfferAlreadyExists
        );
        ensure!(
            !SubnetSaleFrozenColdkeys::<T>::contains_key(&seller),
            Error::<T>::ColdkeyLockedDuringSale
        );
        let owner_hotkey = SubnetOwnerHotkey::<T>::try_get(netuid)
            .map_err(|_| Error::<T>::HotKeyAccountNotExists)?;
        ensure!(
            !SubnetSaleFrozenHotkeys::<T>::contains_key(&owner_hotkey),
            Error::<T>::HotkeyLockedDuringSale
        );

        SubnetSaleOffers::<T>::insert(
            netuid,
            SubnetSaleOffer {
                netuid,
                seller: seller.clone(),
                authorized_buyer: authorized_buyer.clone(),
                price: price.into(),
            },
        );
        SubnetSaleFrozenColdkeys::<T>::insert(&seller, ());
        SubnetSaleFrozenHotkeys::<T>::insert(&owner_hotkey, ());

        Self::deposit_event(Event::SubnetSaleOfferCreated {
            seller,
            netuid,
            price,
            authorized_buyer,
        });

        Ok(())
    }

    pub fn do_cancel_sale_offer(
        maybe_seller: Option<T::AccountId>,
        netuid: NetUid,
    ) -> DispatchResult {
        let offer = SubnetSaleOffers::<T>::get(netuid).ok_or(Error::<T>::SaleOfferNotFound)?;

        // If the caller is not the seller, they are root.
        if let Some(seller) = maybe_seller {
            ensure!(seller == offer.seller, Error::<T>::NotSubnetOwner);
        }

        let seller = offer.seller.clone();
        SubnetSaleOffers::<T>::remove(offer.netuid);
        SubnetSaleFrozenColdkeys::<T>::remove(&offer.seller);
        if let Ok(owner_hotkey) = SubnetOwnerHotkey::<T>::try_get(offer.netuid) {
            SubnetSaleFrozenHotkeys::<T>::remove(owner_hotkey);
        }

        Self::deposit_event(Event::SubnetSaleOfferCancelled { seller, netuid });

        Ok(())
    }
}
