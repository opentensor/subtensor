//! API for getting rate-limited transactions info.
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};

use crate::{
    freeze_struct, utils::rate_limiting::TransactionType, Config, Pallet, TargetStakesPerInterval,
    TxRateLimit,
};

/// Transaction rate limits.
#[freeze_struct("e3734bd0690f2da8")]
#[derive(Decode, Encode, Clone, Debug)]
pub struct RateLimits {
    transaction: Compact<u64>,
    set_children: Compact<u64>,
    set_childkey_take: Compact<u64>,
    stakes: Compact<u64>,
}

/// Contains last blocks, when rate-limited transactions was evaluated.
#[freeze_struct("9154106cd720ce08")]
#[derive(Decode, Encode, Clone, Debug)]
pub struct HotkeyLimitedTxInfo<AccountId> {
    hotkey: AccountId,
    last_block_set_children: Compact<u64>,
    last_block_set_childkey_take: Compact<u64>,
}

impl<T: Config> Pallet<T> {
    /// Get transactions rate limits.
    pub fn get_rate_limits() -> RateLimits {
        RateLimits {
            transaction: TxRateLimit::<T>::get().into(),
            set_children: Self::get_rate_limit(&TransactionType::SetChildren).into(),
            set_childkey_take: Self::get_rate_limit(&TransactionType::SetChildkeyTake).into(),
            stakes: TargetStakesPerInterval::<T>::get().into(),
        }
    }

    /// Get transaction rate limits associated with the `hotkey`.
    pub fn get_limited_tx_info_for_hotkey(
        hotkey: &T::AccountId,
        netuid: u16,
    ) -> HotkeyLimitedTxInfo<T::AccountId> {
        let last_block_set_children =
            Self::get_last_transaction_block(hotkey, netuid, &TransactionType::SetChildren).into();
        let last_block_set_childkey_take =
            Self::get_last_transaction_block(hotkey, netuid, &TransactionType::SetChildkeyTake)
                .into();
        HotkeyLimitedTxInfo {
            hotkey: hotkey.clone(),
            last_block_set_children,
            last_block_set_childkey_take,
        }
    }
}
