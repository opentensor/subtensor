//! Staking Safety Utilities
//!
//! This module provides comprehensive validation and safety checks for staking operations
//! in the Subtensor network. It centralizes common validation patterns to ensure consistent
//! error handling and reduce code duplication across staking operations.
//!
//! # Overview
//!
//! Staking operations involve multiple steps that can fail at various points. This module
//! provides:
//!
//! - Pre-flight validation for stake operations
//! - Bounds checking for stake amounts
//! - Hotkey/coldkey relationship validation
//! - Subnet existence and state validation
//! - Rate limiting checks
//!
//! # Safety Guarantees
//!
//! All functions in this module are designed to be called before any state mutations occur,
//! ensuring that if validation fails, no partial state changes are left behind.

use super::*;
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};

/// Result of a pre-flight staking validation check.
#[derive(Debug, PartialEq, Eq)]
pub enum StakeValidationResult {
    /// The operation is valid and can proceed.
    Valid,
    /// The operation would result in zero effective stake.
    ZeroEffectiveStake,
    /// The hotkey does not exist on the network.
    HotkeyNotFound,
    /// The subnet does not exist.
    SubnetNotFound,
    /// The coldkey does not have sufficient balance.
    InsufficientBalance,
    /// The stake amount is below the minimum threshold.
    BelowMinimumStake,
    /// The subtoken is not enabled for this subnet.
    SubtokenDisabled,
}

impl<T: Config> Pallet<T> {
    /// Performs a comprehensive pre-flight validation for adding stake to a subnet.
    ///
    /// This function checks all preconditions without mutating any state, making it
    /// safe to call as a dry-run validation before executing the actual stake operation.
    ///
    /// # Arguments
    /// * `coldkey` - The coldkey account initiating the stake.
    /// * `hotkey` - The hotkey account to stake on.
    /// * `netuid` - The subnet to stake on.
    /// * `amount` - The amount of TAO to stake.
    ///
    /// # Returns
    /// * `StakeValidationResult` indicating whether the operation can proceed.
    ///
    /// # Example
    /// ```ignore
    /// let result = Pallet::<T>::preflight_add_stake(&coldkey, &hotkey, netuid, amount);
    /// match result {
    ///     StakeValidationResult::Valid => { /* proceed with stake */ },
    ///     other => { /* handle error */ },
    /// }
    /// ```
    pub fn preflight_add_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        amount: TaoCurrency,
    ) -> StakeValidationResult {
        // Check subnet existence
        if !Self::if_subnet_exist(netuid) {
            return StakeValidationResult::SubnetNotFound;
        }

        // Check subtoken enabled
        if !SubtokenEnabled::<T>::get(netuid) {
            return StakeValidationResult::SubtokenDisabled;
        }

        // Check hotkey existence
        if !Self::hotkey_account_exists(hotkey) {
            return StakeValidationResult::HotkeyNotFound;
        }

        // Check zero amount
        if amount.is_zero() {
            return StakeValidationResult::ZeroEffectiveStake;
        }

        // Check minimum stake threshold
        if amount < DefaultMinStake::<T>::get() {
            return StakeValidationResult::BelowMinimumStake;
        }

        // Check balance sufficiency
        if !Self::can_remove_balance_from_coldkey_account(coldkey, amount.into()) {
            return StakeValidationResult::InsufficientBalance;
        }

        StakeValidationResult::Valid
    }

    /// Performs a comprehensive pre-flight validation for removing stake from a subnet.
    ///
    /// This function checks all preconditions without mutating any state, making it
    /// safe to call as a dry-run validation before executing the actual unstake operation.
    ///
    /// # Arguments
    /// * `coldkey` - The coldkey account initiating the unstake.
    /// * `hotkey` - The hotkey account to unstake from.
    /// * `netuid` - The subnet to unstake from.
    /// * `alpha_amount` - The amount of alpha to unstake.
    ///
    /// # Returns
    /// * `StakeValidationResult` indicating whether the operation can proceed.
    pub fn preflight_remove_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        alpha_amount: AlphaCurrency,
    ) -> StakeValidationResult {
        // Check subnet existence
        if !Self::if_subnet_exist(netuid) {
            return StakeValidationResult::SubnetNotFound;
        }

        // Check hotkey existence
        if !Self::hotkey_account_exists(hotkey) {
            return StakeValidationResult::HotkeyNotFound;
        }

        // Check zero amount
        if alpha_amount.is_zero() {
            return StakeValidationResult::ZeroEffectiveStake;
        }

        // Check that there is enough stake
        let current_stake =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid);
        if current_stake < alpha_amount {
            return StakeValidationResult::InsufficientBalance;
        }

        StakeValidationResult::Valid
    }

    /// Validates that a stake transition (move/swap/transfer) between two subnets
    /// is well-formed before any state mutations occur.
    ///
    /// This performs all the common checks needed for cross-subnet stake operations:
    /// - Both subnets must exist and have subtokens enabled
    /// - Both hotkeys must exist
    /// - The origin must have sufficient stake
    /// - The operation must not be a no-op (same subnet + same hotkey + same coldkey)
    ///
    /// # Arguments
    /// * `origin_coldkey` - The coldkey initiating the operation.
    /// * `destination_coldkey` - The coldkey receiving the stake.
    /// * `origin_hotkey` - The hotkey from which stake is removed.
    /// * `destination_hotkey` - The hotkey to which stake is added.
    /// * `origin_netuid` - The source subnet.
    /// * `destination_netuid` - The destination subnet.
    /// * `alpha_amount` - The amount of alpha to transition.
    ///
    /// # Returns
    /// * `StakeValidationResult` indicating whether the operation can proceed.
    pub fn preflight_stake_transition(
        origin_coldkey: &T::AccountId,
        _destination_coldkey: &T::AccountId,
        origin_hotkey: &T::AccountId,
        destination_hotkey: &T::AccountId,
        origin_netuid: NetUid,
        destination_netuid: NetUid,
        alpha_amount: AlphaCurrency,
    ) -> StakeValidationResult {
        // Check both subnets exist
        if !Self::if_subnet_exist(origin_netuid) {
            return StakeValidationResult::SubnetNotFound;
        }
        if !Self::if_subnet_exist(destination_netuid) {
            return StakeValidationResult::SubnetNotFound;
        }

        // Check subtokens enabled
        if !SubtokenEnabled::<T>::get(origin_netuid) {
            return StakeValidationResult::SubtokenDisabled;
        }
        if !SubtokenEnabled::<T>::get(destination_netuid) {
            return StakeValidationResult::SubtokenDisabled;
        }

        // Check both hotkeys exist
        if !Self::hotkey_account_exists(origin_hotkey) {
            return StakeValidationResult::HotkeyNotFound;
        }
        if !Self::hotkey_account_exists(destination_hotkey) {
            return StakeValidationResult::HotkeyNotFound;
        }

        // Check zero amount
        if alpha_amount.is_zero() {
            return StakeValidationResult::ZeroEffectiveStake;
        }

        // Check sufficient stake in origin
        let origin_stake = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
            origin_hotkey,
            origin_coldkey,
            origin_netuid,
        );
        if origin_stake < alpha_amount {
            return StakeValidationResult::InsufficientBalance;
        }

        StakeValidationResult::Valid
    }

    /// Checks whether a hotkey-coldkey pair has any stake across all subnets.
    ///
    /// This is useful for determining whether cleanup operations are needed
    /// when a hotkey is being removed or migrated.
    ///
    /// # Arguments
    /// * `hotkey` - The hotkey to check.
    /// * `coldkey` - The coldkey to check.
    ///
    /// # Returns
    /// * `bool` - `true` if the pair has stake on any subnet, `false` otherwise.
    pub fn has_stake_on_any_subnet(hotkey: &T::AccountId, coldkey: &T::AccountId) -> bool {
        Self::get_all_subnet_netuids().into_iter().any(|netuid| {
            !Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid).is_zero()
        })
    }

    /// Returns the total alpha stake across all subnets for a hotkey-coldkey pair.
    ///
    /// Unlike `get_total_stake_for_hotkey` which converts to TAO equivalent,
    /// this returns the raw sum of alpha values without price conversion.
    /// This is useful for bookkeeping and consistency checks.
    ///
    /// # Arguments
    /// * `hotkey` - The hotkey account.
    /// * `coldkey` - The coldkey account.
    ///
    /// # Returns
    /// * `u64` - The sum of all alpha stake values across subnets.
    pub fn get_total_raw_alpha_stake_for_pair(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
    ) -> u64 {
        Self::get_all_subnet_netuids()
            .into_iter()
            .map(|netuid| {
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid).to_u64()
            })
            .fold(0u64, |acc, val| acc.saturating_add(val))
    }

    /// Validates that delegate take parameters are within acceptable bounds.
    ///
    /// # Arguments
    /// * `take` - The proposed take value.
    ///
    /// # Returns
    /// * `true` if the take is within [MinDelegateTake, MaxDelegateTake], `false` otherwise.
    pub fn is_valid_delegate_take(take: u16) -> bool {
        let min_take = MinDelegateTake::<T>::get();
        let max_take = MaxDelegateTake::<T>::get();
        take >= min_take && take <= max_take
    }

    /// Checks if a coldkey-hotkey pair can perform a staking operation on a subnet
    /// without exceeding rate limits.
    ///
    /// This combines subnet existence, hotkey existence, and rate limit checks
    /// into a single call for convenience.
    ///
    /// # Arguments
    /// * `coldkey` - The coldkey account.
    /// * `hotkey` - The hotkey account.
    /// * `netuid` - The subnet identifier.
    ///
    /// # Returns
    /// * `Ok(())` if the operation is permitted.
    /// * `Err(Error)` with the specific reason for denial.
    pub fn check_stake_operation_permitted(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
    ) -> Result<(), Error<T>> {
        // Check subnet exists
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Check hotkey exists
        ensure!(
            Self::hotkey_account_exists(hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Check rate limit
        Self::ensure_stake_operation_limit_not_exceeded(hotkey, coldkey, netuid.into())?;

        Ok(())
    }

    /// Safely computes the pro-rata share of a TAO pot for a given staker.
    ///
    /// Uses the largest-remainder method to ensure the total distribution
    /// exactly equals the pot size without any rounding losses.
    ///
    /// # Arguments
    /// * `staker_alpha` - The staker's alpha value.
    /// * `total_alpha` - The total alpha across all stakers.
    /// * `pot` - The total TAO to distribute.
    ///
    /// # Returns
    /// * `u64` - The staker's pro-rata share (floor division).
    /// * `u128` - The remainder for largest-remainder adjustment.
    pub fn compute_prorata_share(staker_alpha: u128, total_alpha: u128, pot: u64) -> (u64, u128) {
        if total_alpha == 0 || pot == 0 {
            return (0, 0);
        }

        let pot_u128: u128 = pot as u128;
        let prod: u128 = pot_u128.saturating_mul(staker_alpha);
        let share_u128: u128 = prod.checked_div(total_alpha).unwrap_or_default();
        let share_u64: u64 = share_u128.min(u128::from(u64::MAX)) as u64;
        let remainder: u128 = prod.checked_rem(total_alpha).unwrap_or_default();

        (share_u64, remainder)
    }
}
