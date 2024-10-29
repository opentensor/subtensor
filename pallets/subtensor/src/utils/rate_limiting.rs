use super::*;
use sp_core::Get;

/// Enum representing different types of transactions
#[derive(Copy, Clone)]
pub enum TransactionType {
    SetChildren,
    SetChildkeyTake,
    Unknown,
}

/// Implement conversion from TransactionType to u16
impl From<TransactionType> for u16 {
    fn from(tx_type: TransactionType) -> Self {
        match tx_type {
            TransactionType::SetChildren => 0,
            TransactionType::SetChildkeyTake => 1,
            TransactionType::Unknown => 2,
        }
    }
}

/// Implement conversion from u16 to TransactionType
impl From<u16> for TransactionType {
    fn from(value: u16) -> Self {
        match value {
            0 => TransactionType::SetChildren,
            1 => TransactionType::SetChildkeyTake,
            _ => TransactionType::Unknown,
        }
    }
}
impl<T: Config> Pallet<T> {
    // ========================
    // ==== Rate Limiting =====
    // ========================
    /// Get the rate limit for a specific transaction type
    pub fn get_rate_limit(tx_type: &TransactionType) -> u64 {
        match tx_type {
            TransactionType::SetChildren => (DefaultTempo::<T>::get().saturating_mul(2)).into(), // Cannot set children twice within the default tempo period.
            TransactionType::SetChildkeyTake => TxChildkeyTakeRateLimit::<T>::get(),
            TransactionType::Unknown => 0, // Default to no limit for unknown types (no limit)
        }
    }

    /// Check if a transaction should be rate limited on a specific subnet
    pub fn passes_rate_limit_on_subnet(
        tx_type: &TransactionType,
        hotkey: &T::AccountId,
        netuid: u16,
    ) -> bool {
        let block: u64 = Self::get_current_block_as_u64();
        let limit: u64 = Self::get_rate_limit(tx_type);
        let last_block: u64 = Self::get_last_transaction_block(hotkey, netuid, tx_type);

        // Check for future block numbers.
        // This can happen on chain clones when the block number is rolled back.
        if last_block > block {
            return true; // Allow the transaction if the last block is in the future.
        }

        // Allow the first transaction (when last_block is 0) or if the rate limit has passed
        last_block == 0 || block.saturating_sub(last_block) >= limit
    }

    /// Check if a transaction should be rate limited globally
    pub fn passes_rate_limit_globally(tx_type: &TransactionType, hotkey: &T::AccountId) -> bool {
        let netuid: u16 = u16::MAX;
        let block: u64 = Self::get_current_block_as_u64();
        let limit: u64 = Self::get_rate_limit(tx_type);
        let last_block: u64 = Self::get_last_transaction_block(hotkey, netuid, tx_type);

        // Check for future block numbers.
        // This can happen on chain clones when the block number is rolled back.
        if last_block > block {
            return true; // Allow the transaction if the last block is in the future.
        }

        block.saturating_sub(last_block) >= limit
    }

    /// Get the block number of the last transaction for a specific hotkey, network, and transaction type
    pub fn get_last_transaction_block(
        hotkey: &T::AccountId,
        netuid: u16,
        tx_type: &TransactionType,
    ) -> u64 {
        let tx_as_u16: u16 = (*tx_type).into();
        TransactionKeyLastBlock::<T>::get((hotkey, netuid, tx_as_u16))
    }

    /// Set the block number of the last transaction for a specific hotkey, network, and transaction type
    pub fn set_last_transaction_block(
        hotkey: &T::AccountId,
        netuid: u16,
        tx_type: &TransactionType,
        block: u64,
    ) {
        let tx_as_u16: u16 = (*tx_type).into();
        TransactionKeyLastBlock::<T>::insert((hotkey, netuid, tx_as_u16), block);
    }

    pub fn set_last_tx_block(key: &T::AccountId, block: u64) {
        LastTxBlock::<T>::insert(key, block)
    }
    pub fn get_last_tx_block(key: &T::AccountId) -> u64 {
        LastTxBlock::<T>::get(key)
    }
    pub fn set_last_tx_block_delegate_take(key: &T::AccountId, block: u64) {
        LastTxBlockDelegateTake::<T>::insert(key, block)
    }
    pub fn get_last_tx_block_delegate_take(key: &T::AccountId) -> u64 {
        LastTxBlockDelegateTake::<T>::get(key)
    }

    pub fn set_last_tx_block_childkey_take(key: &T::AccountId, block: u64) {
        LastTxBlockChildKeyTake::<T>::insert(key, block)
    }
    pub fn get_last_tx_block_childkey_take(key: &T::AccountId) -> u64 {
        LastTxBlockChildKeyTake::<T>::get(key)
    }
    pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        let rate_limit: u64 = Self::get_tx_rate_limit();
        if rate_limit == 0 || prev_tx_block == 0 || prev_tx_block > current_block {
            return false;
        }

        // Check for future block numbers.
        // This can happen on chain clones when the block number is rolled back.
        if prev_tx_block > current_block {
            return false;
        }

        current_block.saturating_sub(prev_tx_block) <= rate_limit
    }
    pub fn exceeds_tx_delegate_take_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        let rate_limit: u64 = Self::get_tx_delegate_take_rate_limit();
        if rate_limit == 0 || prev_tx_block == 0 {
            return false;
        }

        // Check for future block numbers.
        // This can happen on chain clones when the block number is rolled back.
        if prev_tx_block > current_block {
            return false;
        }

        current_block.saturating_sub(prev_tx_block) <= rate_limit
    }
}
