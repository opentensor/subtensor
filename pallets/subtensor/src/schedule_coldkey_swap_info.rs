use super::*;
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};

use sp_core::hexdisplay::AsBytesRef;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct ScheduledColdkeySwapInfo<T: Config> {
    old_coldkey: T::AccountId,
    new_coldkey: T::AccountId,
    arbitration_block: Compact<u64>,
}

impl<T: Config> Pallet<T> {
    /// Retrieves the scheduled coldkey swap information for an existing account.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - The account ID of the coldkey to check.
    ///
    /// # Returns
    ///
    /// * `Option<ScheduledColdkeySwapInfo<T>>` - The scheduled coldkey swap information if it exists, otherwise `None`.
    ///
    /// # Notes
    ///
    /// This function checks if there are any destination coldkeys associated with the given coldkey.
    /// If there are, it retrieves the arbitration block and constructs the `ScheduledColdkeySwapInfo` struct.
    fn get_scheduled_coldkey_swap_by_existing_account(
        coldkey: AccountIdOf<T>,
    ) -> Option<ScheduledColdkeySwapInfo<T>> {
        let destinations: Vec<T::AccountId> = ColdkeySwapDestinations::<T>::get(&coldkey);
        if destinations.is_empty() {
            return None;
        }

        let arbitration_block: u64 = ColdkeyArbitrationBlock::<T>::get(&coldkey);

        Some(ScheduledColdkeySwapInfo {
            old_coldkey: coldkey,
            new_coldkey: destinations.first().cloned().unwrap_or_else(|| {
                T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
                    .expect("Infinite length input; no invalid inputs for type; qed")
            }),
            arbitration_block: arbitration_block.into(),
        })
    }

    /// Retrieves the scheduled coldkey swap information for a given coldkey account vector.
    ///
    /// # Arguments
    ///
    /// * `coldkey_account_vec` - The vector of bytes representing the coldkey account.
    ///
    /// # Returns
    ///
    /// * `Option<ScheduledColdkeySwapInfo<T>>` - The scheduled coldkey swap information if it exists, otherwise `None`.
    ///
    /// # Notes
    ///
    /// This function decodes the coldkey account vector into an account ID and then calls
    /// `get_scheduled_coldkey_swap_by_existing_account` to retrieve the swap information.
    pub fn get_scheduled_coldkey_swap(
        coldkey_account_vec: Vec<u8>,
    ) -> Option<ScheduledColdkeySwapInfo<T>> {
        if coldkey_account_vec.len() != 32 {
            return None;
        }

        let coldkey: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).ok()?;
        Self::get_scheduled_coldkey_swap_by_existing_account(coldkey)
    }

    /// Retrieves all scheduled coldkey swaps from storage.
    ///
    /// # Returns
    ///
    /// * `Vec<ScheduledColdkeySwapInfo<T>>` - A vector containing all scheduled coldkey swap information.
    ///
    /// # Notes
    ///
    /// This function iterates over all coldkeys in `ColdkeySwapDestinations` and retrieves their swap information
    /// using `get_scheduled_coldkey_swap_by_existing_account`.
    pub fn get_all_scheduled_coldkey_swaps() -> Vec<ScheduledColdkeySwapInfo<T>> {
        let mut scheduled_swaps: Vec<ScheduledColdkeySwapInfo<T>> = Vec::new();
        for coldkey in ColdkeySwapDestinations::<T>::iter_keys() {
            if let Some(swap_info) = Self::get_scheduled_coldkey_swap_by_existing_account(coldkey) {
                scheduled_swaps.push(swap_info);
            }
        }
        scheduled_swaps
    }

    /// Retrieves the scheduled coldkey swaps for a given block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block number to check for scheduled coldkey swaps.
    ///
    /// # Returns
    ///
    /// * `Vec<ScheduledColdkeySwapInfo<T>>` - A vector containing the scheduled coldkey swap information for the given block.
    ///
    /// # Notes
    ///
    /// This function retrieves the coldkeys to swap at the given block and then retrieves their swap information
    /// using `get_scheduled_coldkey_swap_by_existing_account`.
    pub fn get_scheduled_coldkey_swaps_at_block(block: u64) -> Vec<ScheduledColdkeySwapInfo<T>> {
        let coldkeys_to_swap: Vec<T::AccountId> = ColdkeysToSwapAtBlock::<T>::get(block);
        let mut scheduled_swaps: Vec<ScheduledColdkeySwapInfo<T>> = Vec::new();
        for coldkey in coldkeys_to_swap {
            if let Some(swap_info) = Self::get_scheduled_coldkey_swap_by_existing_account(coldkey) {
                scheduled_swaps.push(swap_info);
            }
        }
        scheduled_swaps
    }

    /// Retrieves the remaining arbitration period for a given coldkey account vector.
    ///
    /// # Arguments
    ///
    /// * `coldkey_account_vec` - The vector of bytes representing the coldkey account.
    ///
    /// # Returns
    ///
    /// * `Option<u64>` - The remaining arbitration period in blocks if it exists, otherwise `None`.
    ///
    /// # Notes
    ///
    /// This function decodes the coldkey account vector into an account ID and calculates the remaining arbitration period
    /// by subtracting the current block number from the arbitration block number.
    pub fn get_remaining_arbitration_period(coldkey_account_vec: Vec<u8>) -> Option<u64> {
        if coldkey_account_vec.len() != 32 {
            return None;
        }

        let coldkey: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).ok()?;
        let current_block: u64 = Self::get_current_block_as_u64();
        let arbitration_block: u64 = ColdkeyArbitrationBlock::<T>::get(&coldkey);

        if arbitration_block > current_block {
            Some(arbitration_block.saturating_sub(current_block))
        } else {
            Some(0)
        }
    }

    /// Retrieves the destination coldkeys for a given coldkey account vector.
    ///
    /// # Arguments
    ///
    /// * `coldkey_account_vec` - The vector of bytes representing the coldkey account.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<T::AccountId>>` - A vector containing the destination coldkeys if they exist, otherwise `None`.
    ///
    /// # Notes
    ///
    /// This function decodes the coldkey account vector into an account ID and retrieves the destination coldkeys
    /// from `ColdkeySwapDestinations`.
    pub fn get_coldkey_swap_destinations(
        coldkey_account_vec: Vec<u8>,
    ) -> Option<Vec<T::AccountId>> {
        if coldkey_account_vec.len() != 32 {
            return None;
        }

        let coldkey: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).ok()?;
        Some(ColdkeySwapDestinations::<T>::get(&coldkey))
    }
}
