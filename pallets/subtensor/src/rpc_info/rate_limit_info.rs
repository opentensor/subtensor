//! API for getting transaction rate limits associated with coldkeys and hotkeys.

use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};

use crate::{
    utils::rate_limiting::TransactionType, Config, Pallet, TargetStakesPerInterval, TxRateLimit,
};

/// Transaction rate limits.
// #[freeze_struct("fe79d58173da662a")]
#[derive(Decode, Encode, Clone, Debug)]
pub struct RateLimits {
    transaction: Compact<u64>,
    set_children: Compact<u64>,
    set_childkey_take: Compact<u64>,
    stakes: Compact<u64>,
}

/// Contains last blocks, when rate-limited transactions was evaluated.
// #[freeze_struct("fe79d58173da662a")]
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
            hotkey: hotkey.to_owned(),
            last_block_set_children,
            last_block_set_childkey_take,
        }
    }
}
