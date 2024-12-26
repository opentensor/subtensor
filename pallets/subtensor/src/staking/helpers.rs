use super::*;
use frame_support::traits::{
    tokens::{
        fungible::{Balanced as _, Inspect as _},
        Fortitude, Precision, Preservation,
    },
    Imbalance,
};

impl<T: Config> Pallet<T> {
    // Returns true if the passed hotkey allow delegative staking.
    //
    pub fn hotkey_is_delegate(hotkey: &T::AccountId) -> bool {
        Delegates::<T>::contains_key(hotkey)
    }

    // Sets the hotkey as a delegate with take.
    //
    pub fn delegate_hotkey(hotkey: &T::AccountId, take: u16) {
        Delegates::<T>::insert(hotkey, take);
    }

    // Returns the total amount of stake in the staking table.
    //
    pub fn get_total_stake() -> u64 {
        TotalStake::<T>::get()
    }

    // Increases the total amount of stake by the passed amount.
    //
    pub fn increase_total_stake(increment: u64) {
        TotalStake::<T>::put(Self::get_total_stake().saturating_add(increment));
    }

    // Decreases the total amount of stake by the passed amount.
    //
    pub fn decrease_total_stake(decrement: u64) {
        TotalStake::<T>::put(Self::get_total_stake().saturating_sub(decrement));
    }

    // Retrieves the total stakes for a given hotkey (account ID) for the current staking interval.
    // pub fn get_stakes_this_interval_for_coldkey_hotkey(
    //     coldkey: &T::AccountId,
    //     hotkey: &T::AccountId,
    // ) -> u64 {
    //     // Retrieve the configured stake interval duration from storage.
    //     let stake_interval = StakeInterval::<T>::get();

    //     // Obtain the current block number as an unsigned 64-bit integer.
    //     let current_block = Self::get_current_block_as_u64();

    //     // Fetch the total stakes and the last block number when stakes were made for the hotkey.
    //     let (stakes, block_last_staked_at) =
    //         TotalHotkeyColdkeyStakesThisInterval::<T>::get(coldkey, hotkey);

    //     // Calculate the block number after which the stakes for the hotkey should be reset.
    //     let block_to_reset_after = block_last_staked_at.saturating_add(stake_interval);

    //     // If the current block number is beyond the reset point,
    //     // it indicates the end of the staking interval for the hotkey.
    //     // Also reset if the current block is less than the last staked block, which can happen
    //     // on chain clones when block number is rolled back.
    //     if block_to_reset_after <= current_block || current_block <= block_last_staked_at {
    //         // Reset the stakes for this hotkey for the current interval.
    //         Self::set_stakes_this_interval_for_coldkey_hotkey(
    //             coldkey,
    //             hotkey,
    //             0,
    //             block_last_staked_at,
    //         );
    //         // Return 0 as the stake amount since we've just reset the stakes.
    //         return 0;
    //     }

    //     // If the staking interval has not yet ended, return the current stake amount.
    //     stakes
    // }   (DEPRECATED)

    // pub fn get_target_stakes_per_interval() -> u64 {
    //     TargetStakesPerInterval::<T>::get()
    // }  (DEPRECATED)

    // Creates a cold - hot pairing account if the hotkey is not already an active account.
    //
    pub fn create_account_if_non_existent(coldkey: &T::AccountId, hotkey: &T::AccountId) {
        if !Self::hotkey_account_exists(hotkey) {
            Stake::<T>::insert(hotkey, coldkey, 0);
            Owner::<T>::insert(hotkey, coldkey);

            // Update OwnedHotkeys map
            let mut hotkeys = OwnedHotkeys::<T>::get(coldkey);
            if !hotkeys.contains(hotkey) {
                hotkeys.push(hotkey.clone());
                OwnedHotkeys::<T>::insert(coldkey, hotkeys);
            }

            // Update StakingHotkeys map
            let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
            if !staking_hotkeys.contains(hotkey) {
                staking_hotkeys.push(hotkey.clone());
                StakingHotkeys::<T>::insert(coldkey, staking_hotkeys);
            }
        }
    }

    /// Returns the coldkey owning this hotkey. This function should only be called for active accounts.
    ///
    /// # Arguments
    /// * `hotkey` - The hotkey account ID.
    ///
    /// # Returns
    /// The coldkey account ID that owns the hotkey.
    pub fn get_owning_coldkey_for_hotkey(hotkey: &T::AccountId) -> T::AccountId {
        Owner::<T>::get(hotkey)
    }

    /// Returns the hotkey take.
    ///
    /// # Arguments
    /// * `hotkey` - The hotkey account ID.
    ///
    /// # Returns
    /// The take value of the hotkey.
    pub fn get_hotkey_take(hotkey: &T::AccountId) -> u16 {
        Delegates::<T>::get(hotkey)
    }

    /// Returns true if the hotkey account has been created.
    ///
    /// # Arguments
    /// * `hotkey` - The hotkey account ID.
    ///
    /// # Returns
    /// True if the hotkey account exists, false otherwise.
    pub fn hotkey_account_exists(hotkey: &T::AccountId) -> bool {
        Owner::<T>::contains_key(hotkey)
    }

    /// Returns true if the passed coldkey owns the hotkey.
    ///
    /// # Arguments
    /// * `coldkey` - The coldkey account ID.
    /// * `hotkey` - The hotkey account ID.
    ///
    /// # Returns
    /// True if the coldkey owns the hotkey, false otherwise.
    pub fn coldkey_owns_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> bool {
        if Self::hotkey_account_exists(hotkey) {
            Owner::<T>::get(hotkey) == *coldkey
        } else {
            false
        }
    }

    /// Clears the nomination for an account, if it is a nominator account and the stake is below the minimum required threshold.
    pub fn clear_small_nomination_if_required(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        stake: u64,
    ) {
        // Verify if the account is a nominator account by checking ownership of the hotkey by the coldkey.
        if !Self::coldkey_owns_hotkey(coldkey, hotkey) {
            // If the stake is below the minimum required, it's considered a small nomination and needs to be cleared.
            // Log if the stake is below the minimum required
            if stake < Self::get_nominator_min_required_stake() {
                // Log the clearing of a small nomination
                // Remove the stake from the nominator account. (this is a more forceful unstake operation which )
                // Actually deletes the staking account.
                let cleared_stake = Self::unstake_from_subnet(hotkey, coldkey, netuid, stake);
                // Add the stake to the coldkey account.
                Self::add_balance_to_coldkey_account(coldkey, cleared_stake);
            }
        }
    }

    /// Clears small nominations for all accounts.
    ///
    /// WARN: This is an O(N) operation, where N is the number of staking accounts. It should be
    /// used with caution.
    pub fn clear_small_nominations() {
        // Loop through all staking accounts to identify and clear nominations below the minimum stake.
        for ((hotkey, coldkey, netuid), stake) in Alpha::<T>::iter() {
            Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid, stake);
        }
    }

    pub fn add_balance_to_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) {
        // infallible
        let _ = T::Currency::deposit(coldkey, amount, Precision::BestEffort);
    }

    pub fn can_remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> bool {
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }

        // This bit is currently untested. @todo

        T::Currency::can_withdraw(coldkey, amount)
            .into_result(false)
            .is_ok()
    }

    pub fn get_coldkey_balance(
        coldkey: &T::AccountId,
    ) -> <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance
    {
        T::Currency::reducible_balance(coldkey, Preservation::Expendable, Fortitude::Polite)
    }

    #[must_use = "Balance must be used to preserve total issuance of token"]
    pub fn remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> Result<u64, DispatchError> {
        if amount == 0 {
            return Ok(0);
        }

        let credit = T::Currency::withdraw(
            coldkey,
            amount,
            Precision::BestEffort,
            Preservation::Preserve,
            Fortitude::Polite,
        )
        .map_err(|_| Error::<T>::BalanceWithdrawalError)?
        .peek();

        if credit == 0 {
            return Err(Error::<T>::ZeroBalanceAfterWithdrawn.into());
        }

        Ok(credit)
    }

    pub fn kill_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> Result<u64, DispatchError> {
        if amount == 0 {
            return Ok(0);
        }

        let credit = T::Currency::withdraw(
            coldkey,
            amount,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Force,
        )
        .map_err(|_| Error::<T>::BalanceWithdrawalError)?
        .peek();

        if credit == 0 {
            return Err(Error::<T>::ZeroBalanceAfterWithdrawn.into());
        }

        Ok(credit)
    }

    // TODO deprecate this function.
    // DEPRECATED.
    // pub fn unstake_all_coldkeys_from_hotkey_account_on_network(hotkey: &T::AccountId, netuid: u16) {
    //     // Iterate through all coldkeys that have a stake on this hotkey account.
    //     for (nominator, _) in Stake::<T>::iter_prefix(hotkey) {
    //         for netuid_i in Self::get_all_subnet_netuids() {
    //             if netuid_i != netuid {
    //                 continue;
    //             }
    //             let alpha: u64 =
    //                 Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &nominator, netuid_i);
    //             let tao: u64 = Self::unstake_from_subnet(hotkey, &nominator, netuid_i, alpha);
    //             Self::add_balance_to_coldkey_account(&nominator, tao);
    //         }
    //     }
    // }
}
