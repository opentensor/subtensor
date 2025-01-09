use super::*;

/// Enum representing different types of transactions
#[derive(Copy, Clone)]
pub enum TransactionType {
    SetChildren,
    SetChildkeyTake,
    Unknown,
    RegisterNetwork,
}

/// Implement conversion from TransactionType to u16
impl From<TransactionType> for u16 {
    fn from(tx_type: TransactionType) -> Self {
        match tx_type {
            TransactionType::SetChildren => 0,
            TransactionType::SetChildkeyTake => 1,
            TransactionType::Unknown => 2,
            TransactionType::RegisterNetwork => 3,
        }
    }
}

/// Implement conversion from u16 to TransactionType
impl From<u16> for TransactionType {
    fn from(value: u16) -> Self {
        match value {
            0 => TransactionType::SetChildren,
            1 => TransactionType::SetChildkeyTake,
            3 => TransactionType::RegisterNetwork,
            _ => TransactionType::Unknown,
        }
    }
}
impl<T: Config> Pallet<T> {
    /// Get the rate limit for a specific transaction type
    pub fn get_rate_limit(tx_type: &TransactionType) -> u64 {
        match tx_type {
            TransactionType::SetChildren => 150, // 30 minutes
            TransactionType::SetChildkeyTake => TxChildkeyTakeRateLimit::<T>::get(),
            TransactionType::Unknown => 0, // Default to no limit for unknown types (no limit)
            TransactionType::RegisterNetwork => NetworkRateLimit::<T>::get(),
        }
    }

    pub fn check_passes_rate_limit(limit: u64, block: u64, last_block: u64) -> bool {
        // Allow the first transaction (when last_block is 0) or if the rate limit has passed
        last_block == 0 || block.saturating_sub(last_block) >= limit
    }

    pub fn passes_rate_limit(tx_type: &TransactionType, key: &T::AccountId) -> bool {
        let block: u64 = Self::get_current_block_as_u64();
        let limit: u64 = Self::get_rate_limit(tx_type);
        let last_block: u64 = Self::get_last_transaction_block(key, tx_type);

        Self::check_passes_rate_limit(limit, block, last_block)
    }

    /// Check if a transaction should be rate limited on a specific subnet
    pub fn passes_rate_limit_on_subnet(
        tx_type: &TransactionType,
        hotkey: &T::AccountId,
        netuid: u16,
    ) -> bool {
        let block: u64 = Self::get_current_block_as_u64();
        let limit: u64 = Self::get_rate_limit(tx_type);
        let last_block: u64 = Self::get_last_transaction_block_on_subnet(hotkey, netuid, tx_type);

        Self::check_passes_rate_limit(limit, block, last_block)
    }

    /// Get the block number of the last transaction for a specific key, and transaction type
    pub fn get_last_transaction_block(key: &T::AccountId, tx_type: &TransactionType) -> u64 {
        match tx_type {
            TransactionType::RegisterNetwork => NetworkLastRegistered::<T>::get(),
            _ => Self::get_last_transaction_block_on_subnet(key, 0, tx_type),
        }
    }

    /// Get the block number of the last transaction for a specific hotkey, network, and transaction type
    pub fn get_last_transaction_block_on_subnet(
        hotkey: &T::AccountId,
        netuid: u16,
        tx_type: &TransactionType,
    ) -> u64 {
        match tx_type {
            TransactionType::RegisterNetwork => NetworkLastRegistered::<T>::get(),
            _ => {
                let tx_as_u16: u16 = (*tx_type).into();
                TransactionKeyLastBlock::<T>::get((hotkey, netuid, tx_as_u16))
            }
        }
    }

    /// Set the block number of the last transaction for a specific key, and transaction type
    pub fn set_last_transaction_block(key: &T::AccountId, tx_type: &TransactionType, block: u64) {
        match tx_type {
            TransactionType::RegisterNetwork => Self::set_network_last_lock_block(block),
            _ => Self::set_last_transaction_block_on_subnet(key, 0, tx_type, block),
        }
    }

    /// Set the block number of the last transaction for a specific hotkey, network, and transaction type
    pub fn set_last_transaction_block_on_subnet(
        key: &T::AccountId,
        netuid: u16,
        tx_type: &TransactionType,
        block: u64,
    ) {
        match tx_type {
            TransactionType::RegisterNetwork => Self::set_network_last_lock_block(block),
            _ => {
                let tx_as_u16: u16 = (*tx_type).into();
                TransactionKeyLastBlock::<T>::insert((key, netuid, tx_as_u16), block);
            }
        }
    }

    pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        let rate_limit: u64 = TxRateLimit::<T>::get();
        if rate_limit == 0 || prev_tx_block == 0 {
            return false;
        }

        current_block.saturating_sub(prev_tx_block) <= rate_limit
    }
    pub fn exceeds_tx_delegate_take_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        let rate_limit: u64 = TxDelegateTakeRateLimit::<T>::get();
        if rate_limit == 0 || prev_tx_block == 0 {
            return false;
        }

        current_block.saturating_sub(prev_tx_block) <= rate_limit
    }
}
