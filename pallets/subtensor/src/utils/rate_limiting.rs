use subtensor_runtime_common::NetUid;

use super::*;

/// Enum representing different types of transactions
#[derive(Copy, Clone)]
pub enum TransactionType {
    SetChildren,
    SetChildkeyTake,
    Unknown,
    RegisterNetwork,
    SetWeightsVersionKey,
    SetSNOwnerHotkey,
    OwnerHyperparamUpdate, // Deprecated aggregate; keep for compatibility if referenced in tests
    SubsubnetParameterUpdate,
    // Per-hyperparameter owner updates (rate-limited independently)
    OwnerSetServingRateLimit,
    OwnerSetMaxDifficulty,
    OwnerSetAdjustmentAlpha,
    OwnerSetMaxWeightLimit,
    OwnerSetImmunityPeriod,
    OwnerSetMinAllowedWeights,
    OwnerSetKappa,
    OwnerSetRho,
    OwnerSetActivityCutoff,
    OwnerSetPowRegistrationAllowed,
    OwnerSetMinBurn,
    OwnerSetMaxBurn,
    OwnerSetBondsMovingAverage,
    OwnerSetBondsPenalty,
    OwnerToggleCommitReveal,
    OwnerToggleLiquidAlphaEnabled,
    OwnerSetAlphaValues,
    OwnerSetWeightCommitInterval,
    OwnerToggleTransfer,
    OwnerSetAlphaSigmoidSteepness,
    OwnerToggleYuma3Enabled,
    OwnerToggleBondsReset,
    OwnerSetOwnerImmuneNeuronLimit,
}

/// Implement conversion from TransactionType to u16
impl From<TransactionType> for u16 {
    fn from(tx_type: TransactionType) -> Self {
        match tx_type {
            TransactionType::SetChildren => 0,
            TransactionType::SetChildkeyTake => 1,
            TransactionType::Unknown => 2,
            TransactionType::RegisterNetwork => 3,
            TransactionType::SetWeightsVersionKey => 4,
            TransactionType::SetSNOwnerHotkey => 5,
            TransactionType::OwnerHyperparamUpdate => 6,
            TransactionType::SubsubnetParameterUpdate => 7,
            TransactionType::OwnerSetServingRateLimit => 10,
            TransactionType::OwnerSetMaxDifficulty => 11,
            TransactionType::OwnerSetAdjustmentAlpha => 12,
            TransactionType::OwnerSetMaxWeightLimit => 13,
            TransactionType::OwnerSetImmunityPeriod => 14,
            TransactionType::OwnerSetMinAllowedWeights => 15,
            TransactionType::OwnerSetKappa => 16,
            TransactionType::OwnerSetRho => 17,
            TransactionType::OwnerSetActivityCutoff => 18,
            TransactionType::OwnerSetPowRegistrationAllowed => 19,
            TransactionType::OwnerSetMinBurn => 20,
            TransactionType::OwnerSetMaxBurn => 21,
            TransactionType::OwnerSetBondsMovingAverage => 22,
            TransactionType::OwnerSetBondsPenalty => 23,
            TransactionType::OwnerToggleCommitReveal => 24,
            TransactionType::OwnerToggleLiquidAlphaEnabled => 25,
            TransactionType::OwnerSetAlphaValues => 26,
            TransactionType::OwnerSetWeightCommitInterval => 27,
            TransactionType::OwnerToggleTransfer => 28,
            TransactionType::OwnerSetAlphaSigmoidSteepness => 29,
            TransactionType::OwnerToggleYuma3Enabled => 30,
            TransactionType::OwnerToggleBondsReset => 31,
            TransactionType::OwnerSetOwnerImmuneNeuronLimit => 32,
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
            4 => TransactionType::SetWeightsVersionKey,
            5 => TransactionType::SetSNOwnerHotkey,
            6 => TransactionType::OwnerHyperparamUpdate,
            7 => TransactionType::SubsubnetParameterUpdate,
            10 => TransactionType::OwnerSetServingRateLimit,
            11 => TransactionType::OwnerSetMaxDifficulty,
            12 => TransactionType::OwnerSetAdjustmentAlpha,
            13 => TransactionType::OwnerSetMaxWeightLimit,
            14 => TransactionType::OwnerSetImmunityPeriod,
            15 => TransactionType::OwnerSetMinAllowedWeights,
            16 => TransactionType::OwnerSetKappa,
            17 => TransactionType::OwnerSetRho,
            18 => TransactionType::OwnerSetActivityCutoff,
            19 => TransactionType::OwnerSetPowRegistrationAllowed,
            20 => TransactionType::OwnerSetMinBurn,
            21 => TransactionType::OwnerSetMaxBurn,
            22 => TransactionType::OwnerSetBondsMovingAverage,
            23 => TransactionType::OwnerSetBondsPenalty,
            24 => TransactionType::OwnerToggleCommitReveal,
            25 => TransactionType::OwnerToggleLiquidAlphaEnabled,
            26 => TransactionType::OwnerSetAlphaValues,
            27 => TransactionType::OwnerSetWeightCommitInterval,
            28 => TransactionType::OwnerToggleTransfer,
            29 => TransactionType::OwnerSetAlphaSigmoidSteepness,
            30 => TransactionType::OwnerToggleYuma3Enabled,
            31 => TransactionType::OwnerToggleBondsReset,
            32 => TransactionType::OwnerSetOwnerImmuneNeuronLimit,
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
            TransactionType::SetChildren => 150, // 30 minutes
            TransactionType::SetChildkeyTake => TxChildkeyTakeRateLimit::<T>::get(),
            TransactionType::RegisterNetwork => NetworkRateLimit::<T>::get(),
            TransactionType::SubsubnetParameterUpdate => SubsubnetCountSetRateLimit::<T>::get(),

            TransactionType::Unknown => 0, // Default to no limit for unknown types (no limit)
            _ => 0,
        }
    }

    pub fn get_rate_limit_on_subnet(tx_type: &TransactionType, netuid: NetUid) -> u64 {
        #[allow(clippy::match_single_binding)]
        match tx_type {
            TransactionType::SetWeightsVersionKey => (Tempo::<T>::get(netuid) as u64)
                .saturating_mul(WeightsVersionKeyRateLimit::<T>::get()),
            // Owner hyperparameter updates are rate-limited by N tempos on the subnet (sudo configurable)
            TransactionType::OwnerHyperparamUpdate
            | TransactionType::OwnerSetServingRateLimit
            | TransactionType::OwnerSetMaxDifficulty
            | TransactionType::OwnerSetAdjustmentAlpha
            | TransactionType::OwnerSetMaxWeightLimit
            | TransactionType::OwnerSetImmunityPeriod
            | TransactionType::OwnerSetMinAllowedWeights
            | TransactionType::OwnerSetKappa
            | TransactionType::OwnerSetRho
            | TransactionType::OwnerSetActivityCutoff
            | TransactionType::OwnerSetPowRegistrationAllowed
            | TransactionType::OwnerSetMinBurn
            | TransactionType::OwnerSetMaxBurn
            | TransactionType::OwnerSetBondsMovingAverage
            | TransactionType::OwnerSetBondsPenalty
            | TransactionType::OwnerToggleCommitReveal
            | TransactionType::OwnerToggleLiquidAlphaEnabled
            | TransactionType::OwnerSetAlphaValues
            | TransactionType::OwnerSetWeightCommitInterval
            | TransactionType::OwnerToggleTransfer
            | TransactionType::OwnerSetAlphaSigmoidSteepness
            | TransactionType::OwnerToggleYuma3Enabled
            | TransactionType::OwnerToggleBondsReset
            | TransactionType::OwnerSetOwnerImmuneNeuronLimit => {
                let epochs = OwnerHyperparamRateLimit::<T>::get() as u64;
                (Tempo::<T>::get(netuid) as u64).saturating_mul(epochs)
            }
            TransactionType::SetSNOwnerHotkey => DefaultSetSNOwnerHotkeyRateLimit::<T>::get(),

            _ => Self::get_rate_limit(tx_type),
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
        netuid: NetUid,
    ) -> bool {
        let block: u64 = Self::get_current_block_as_u64();
        let limit: u64 = Self::get_rate_limit_on_subnet(tx_type, netuid);
        let last_block: u64 = Self::get_last_transaction_block_on_subnet(hotkey, netuid, tx_type);

        Self::check_passes_rate_limit(limit, block, last_block)
    }

    /// Get the block number of the last transaction for a specific key, and transaction type
    pub fn get_last_transaction_block(key: &T::AccountId, tx_type: &TransactionType) -> u64 {
        match tx_type {
            TransactionType::RegisterNetwork => Self::get_network_last_lock_block(),
            _ => Self::get_last_transaction_block_on_subnet(key, NetUid::ROOT, tx_type),
        }
    }

    /// Get the block number of the last transaction for a specific hotkey, network, and transaction type
    pub fn get_last_transaction_block_on_subnet(
        hotkey: &T::AccountId,
        netuid: NetUid,
        tx_type: &TransactionType,
    ) -> u64 {
        match tx_type {
            TransactionType::RegisterNetwork => Self::get_network_last_lock_block(),
            TransactionType::SetSNOwnerHotkey => {
                Self::get_rate_limited_last_block(&RateLimitKey::SetSNOwnerHotkey(netuid))
            }
            TransactionType::OwnerHyperparamUpdate => {
                Self::get_rate_limited_last_block(&RateLimitKey::OwnerHyperparamUpdate(netuid))
            }
            _ => {
                let tx_as_u16: u16 = (*tx_type).into();
                TransactionKeyLastBlock::<T>::get((hotkey, netuid, tx_as_u16))
            }
        }
    }

    /// Set the block number of the last transaction for a specific hotkey, network, and transaction type
    pub fn set_last_transaction_block_on_subnet(
        key: &T::AccountId,
        netuid: NetUid,
        tx_type: &TransactionType,
        block: u64,
    ) {
        match tx_type {
            TransactionType::RegisterNetwork => Self::set_network_last_lock_block(block),
            TransactionType::SetSNOwnerHotkey => {
                Self::set_rate_limited_last_block(&RateLimitKey::SetSNOwnerHotkey(netuid), block)
            }
            TransactionType::OwnerHyperparamUpdate => Self::set_rate_limited_last_block(
                &RateLimitKey::OwnerHyperparamUpdate(netuid),
                block,
            ),
            _ => {
                let tx_as_u16: u16 = (*tx_type).into();
                TransactionKeyLastBlock::<T>::insert((key, netuid, tx_as_u16), block);
            }
        }
    }

    pub fn remove_last_tx_block(key: &T::AccountId) {
        Self::remove_rate_limited_last_block(&RateLimitKey::LastTxBlock(key.clone()))
    }
    pub fn set_last_tx_block(key: &T::AccountId, block: u64) {
        Self::set_rate_limited_last_block(&RateLimitKey::LastTxBlock(key.clone()), block);
    }
    pub fn get_last_tx_block(key: &T::AccountId) -> u64 {
        Self::get_rate_limited_last_block(&RateLimitKey::LastTxBlock(key.clone()))
    }

    pub fn remove_last_tx_block_delegate_take(key: &T::AccountId) {
        Self::remove_rate_limited_last_block(&RateLimitKey::LastTxBlockDelegateTake(key.clone()))
    }
    pub fn set_last_tx_block_delegate_take(key: &T::AccountId, block: u64) {
        Self::set_rate_limited_last_block(
            &RateLimitKey::LastTxBlockDelegateTake(key.clone()),
            block,
        );
    }
    pub fn get_last_tx_block_delegate_take(key: &T::AccountId) -> u64 {
        Self::get_rate_limited_last_block(&RateLimitKey::LastTxBlockDelegateTake(key.clone()))
    }
    pub fn get_last_tx_block_childkey_take(key: &T::AccountId) -> u64 {
        Self::get_rate_limited_last_block(&RateLimitKey::LastTxBlockChildKeyTake(key.clone()))
    }
    pub fn remove_last_tx_block_childkey(key: &T::AccountId) {
        Self::remove_rate_limited_last_block(&RateLimitKey::LastTxBlockChildKeyTake(key.clone()))
    }
    pub fn set_last_tx_block_childkey(key: &T::AccountId, block: u64) {
        Self::set_rate_limited_last_block(
            &RateLimitKey::LastTxBlockChildKeyTake(key.clone()),
            block,
        );
    }
    pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        let rate_limit: u64 = Self::get_tx_rate_limit();
        if rate_limit == 0 || prev_tx_block == 0 {
            return false;
        }

        current_block.saturating_sub(prev_tx_block) <= rate_limit
    }
    pub fn exceeds_tx_delegate_take_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
        let rate_limit: u64 = Self::get_tx_delegate_take_rate_limit();
        if rate_limit == 0 || prev_tx_block == 0 {
            return false;
        }

        current_block.saturating_sub(prev_tx_block) <= rate_limit
    }
}
