//! This file contains all tooling to work with sub-subnets
//!

use super::*;
use crate::epoch::run_epoch::EpochTerms;
use alloc::collections::BTreeMap;
use safe_math::*;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, NetUid, NetUidStorageIndex, SubId};

pub type LeaseId = u32;

pub type CurrencyOf<T> = <T as Config>::Currency;

pub type BalanceOf<T> =
    <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Theoretical maximum of subnets on bittensor. This value is used in indexed
/// storage of epoch values for sub-subnets as
///
/// `storage_index = netuid + sub_id * GLOBAL_MAX_SUBNET_COUNT`
///
/// For sub_id = 0 this index results in netuid and provides backward compatibility
/// for subnets with default sub-subnet count of 1.
///
/// Changing this value will require a migration of all epoch maps.
///
pub const GLOBAL_MAX_SUBNET_COUNT: u16 = 4096;

// Theoretical maximum number of subsubnets per subnet
// GLOBAL_MAX_SUBNET_COUNT * MAX_SUBSUBNET_COUNT_PER_SUBNET should be 0x10000
pub const MAX_SUBSUBNET_COUNT_PER_SUBNET: u8 = 16;

impl<T: Config> Pallet<T> {
    pub fn get_subsubnet_storage_index(netuid: NetUid, sub_id: SubId) -> NetUidStorageIndex {
        u16::from(sub_id)
            .saturating_mul(GLOBAL_MAX_SUBNET_COUNT)
            .saturating_add(u16::from(netuid))
            .into()
    }

    pub fn get_netuid_and_subid(
        netuid_index: NetUidStorageIndex,
    ) -> Result<(NetUid, SubId), Error<T>> {
        let maybe_netuid = u16::from(netuid_index).checked_rem(GLOBAL_MAX_SUBNET_COUNT);
        if let Some(netuid_u16) = maybe_netuid {
            let netuid = NetUid::from(netuid_u16);

            // Make sure the base subnet exists
            ensure!(
                Self::if_subnet_exist(netuid),
                Error::<T>::SubNetworkDoesNotExist
            );

            // Extract sub_id
            let sub_id_u8 = u8::try_from(u16::from(netuid_index).safe_div(GLOBAL_MAX_SUBNET_COUNT))
                .map_err(|_| Error::<T>::SubNetworkDoesNotExist)?;
            let sub_id = SubId::from(sub_id_u8);

            if SubsubnetCountCurrent::<T>::get(netuid) > sub_id {
                Ok((netuid, sub_id))
            } else {
                Err(Error::<T>::SubNetworkDoesNotExist.into())
            }
        } else {
            Err(Error::<T>::SubNetworkDoesNotExist.into())
        }
    }

    pub fn ensure_subsubnet_exists(netuid: NetUid, sub_id: SubId) -> DispatchResult {
        // Make sure the base subnet exists
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // Make sure the subsub limit is not exceeded
        ensure!(
            SubsubnetCountCurrent::<T>::get(netuid) > sub_id,
            Error::<T>::SubNetworkDoesNotExist
        );
        Ok(())
    }

    /// Set the desired valus of sub-subnet count for a subnet identified
    /// by netuid
    pub fn do_set_desired_subsubnet_count(
        netuid: NetUid,
        subsubnet_count: SubId,
    ) -> DispatchResult {
        // Make sure the subnet exists
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // Count cannot be zero
        ensure!(subsubnet_count > 0.into(), Error::<T>::InvalidValue);

        // Make sure we are not exceeding the max sub-subnet count
        ensure!(
            subsubnet_count <= MaxSubsubnetCount::<T>::get(),
            Error::<T>::InvalidValue
        );

        // Make sure we are not allowing numbers that will break the math
        ensure!(
            subsubnet_count <= SubId::from(MAX_SUBSUBNET_COUNT_PER_SUBNET),
            Error::<T>::InvalidValue
        );

        SubsubnetCountDesired::<T>::insert(netuid, subsubnet_count);
        Ok(())
    }

    /// Update current count for a subnet identified by netuid
    ///
    /// - This function should be called in every block in run_counbase
    /// - Cleans up all sub-subnet maps if count is reduced
    /// - Decreases or increases current subsubnet count by no more than
    ///   `GlobalSubsubnetDecreasePerSuperblock`
    ///
    pub fn update_subsubnet_counts_if_needed(current_block: u64) {
        // Run once per super-block
        let super_block_tempos = u64::from(SuperBlockTempos::<T>::get());
        Self::get_all_subnet_netuids().iter().for_each(|netuid| {
            let epoch_index = Self::get_epoch_index(*netuid, current_block);
            if let Some(rem) = epoch_index.checked_rem(super_block_tempos) {
                if rem == 0 {
                    let old_count = u8::from(SubsubnetCountCurrent::<T>::get(netuid));
                    let desired_count = u8::from(SubsubnetCountDesired::<T>::get(netuid));
                    let min_capped_count = old_count
                        .saturating_sub(u8::from(GlobalSubsubnetDecreasePerSuperblock::<T>::get()))
                        .max(1);
                    let max_capped_count = old_count
                        .saturating_add(u8::from(GlobalSubsubnetDecreasePerSuperblock::<T>::get()));
                    let new_count = desired_count.max(min_capped_count).min(max_capped_count);

                    if old_count != new_count {
                        if old_count > new_count {
                            for subid in new_count..old_count {
                                let netuid_index =
                                    Self::get_subsubnet_storage_index(*netuid, SubId::from(subid));

                                // Cleanup Weights
                                let _ = Weights::<T>::clear_prefix(netuid_index, u32::MAX, None);

                                // Cleanup Incentive
                                Incentive::<T>::remove(netuid_index);

                                // Cleanup LastUpdate
                                LastUpdate::<T>::remove(netuid_index);

                                // Cleanup Bonds
                                let _ = Bonds::<T>::clear_prefix(netuid_index, u32::MAX, None);

                                // Cleanup WeightCommits
                                let _ =
                                    WeightCommits::<T>::clear_prefix(netuid_index, u32::MAX, None);

                                // Cleanup TimelockedWeightCommits
                                let _ = TimelockedWeightCommits::<T>::clear_prefix(
                                    netuid_index,
                                    u32::MAX,
                                    None,
                                );
                            }
                        }

                        SubsubnetCountCurrent::<T>::insert(netuid, SubId::from(new_count));

                        // Reset split back to even
                        SubsubnetEmissionSplit::<T>::remove(netuid);
                    }
                }
            }
        });
    }

    pub fn do_set_emission_split(netuid: NetUid, maybe_split: Option<Vec<u16>>) -> DispatchResult {
        // Make sure the subnet exists
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        if let Some(split) = maybe_split {
            // Check the length
            ensure!(!split.is_empty(), Error::<T>::InvalidValue);
            ensure!(
                split.len() <= u8::from(SubsubnetCountCurrent::<T>::get(netuid)) as usize,
                Error::<T>::InvalidValue
            );

            // Check that values add up to 65535
            let total: u64 = split.iter().map(|s| *s as u64).sum();
            ensure!(total <= u16::MAX as u64, Error::<T>::InvalidValue);

            SubsubnetEmissionSplit::<T>::insert(netuid, split);
        } else {
            SubsubnetEmissionSplit::<T>::remove(netuid);
        }

        Ok(())
    }

    /// Split alpha emission in sub-subnet proportions
    /// stored in SubsubnetEmissionSplit
    ///
    pub fn split_emissions(netuid: NetUid, alpha: AlphaCurrency) -> Vec<AlphaCurrency> {
        let subsubnet_count = u64::from(SubsubnetCountCurrent::<T>::get(netuid));
        let maybe_split = SubsubnetEmissionSplit::<T>::get(netuid);

        // Unset split means even distribution
        let mut result: Vec<AlphaCurrency> = if let Some(split) = maybe_split {
            split
                .iter()
                .map(|s| {
                    AlphaCurrency::from(
                        (u64::from(alpha) as u128)
                            .saturating_mul(*s as u128)
                            .safe_div(u16::MAX as u128) as u64,
                    )
                })
                .collect()
        } else {
            let per_subsubnet = u64::from(alpha).safe_div(subsubnet_count);
            vec![AlphaCurrency::from(per_subsubnet); subsubnet_count as usize]
        };

        // Trim / extend and pad with zeroes if result is shorter than subsubnet_count
        if result.len() != subsubnet_count as usize {
            result.resize(subsubnet_count as usize, 0u64.into()); // pad with AlphaCurrency::from(0)
        }

        // If there's any rounding error or lost due to truncation emission, credit it to subsubnet 0
        let rounding_err =
            u64::from(alpha).saturating_sub(result.iter().map(|s| u64::from(*s)).sum());
        if let Some(cell) = result.first_mut() {
            *cell = cell.saturating_add(AlphaCurrency::from(rounding_err));
        }
        result
    }

    fn weighted_acc_u16(existing: u16, added: u16, weight: U64F64) -> u16 {
        U64F64::saturating_from_num(existing)
            .saturating_add(U64F64::saturating_from_num(added).saturating_mul(weight))
            .saturating_to_num::<u16>()
    }

    fn weighted_acc_alpha(
        existing: AlphaCurrency,
        added: AlphaCurrency,
        weight: U64F64,
    ) -> AlphaCurrency {
        U64F64::saturating_from_num(existing)
            .saturating_add(U64F64::saturating_from_num(added).saturating_mul(weight))
            .saturating_to_num::<u64>()
            .into()
    }

    /// Splits rao_emission between different sub-subnets using `split_emissions` function.
    ///
    /// Runs the epoch function for each sub-subnet and consolidates hotkey_emission
    /// into a single vector.
    ///
    pub fn epoch_with_subsubnets(
        netuid: NetUid,
        rao_emission: AlphaCurrency,
    ) -> Vec<(T::AccountId, AlphaCurrency, AlphaCurrency)> {
        let aggregated: BTreeMap<T::AccountId, EpochTerms> =
            Self::split_emissions(netuid, rao_emission)
                .into_iter()
                .enumerate()
                // Run epoch function for each subsubnet to distribute its portion of emissions
                .flat_map(|(sub_id_usize, sub_emission)| {
                    let sub_id_u8: u8 = sub_id_usize.try_into().unwrap_or_default();
                    let sub_id = SubId::from(sub_id_u8);

                    // Run epoch function on the subsubnet emission
                    let epoch_output = Self::epoch_subsubnet(netuid, sub_id, sub_emission);
                    Self::persist_subsub_epoch_terms(netuid, sub_id, epoch_output.as_map());

                    // Calculate subsubnet weight from the split emission (not the other way because preserving
                    // emission accuracy is the priority)
                    let sub_weight = U64F64::saturating_from_num(sub_emission)
                        .safe_div(U64F64::saturating_from_num(rao_emission));

                    // Produce an iterator of (hotkey, (terms, sub_weight)) tuples
                    epoch_output
                        .0
                        .into_iter()
                        .map(move |(hotkey, terms)| (hotkey, (terms, sub_weight)))
                })
                // Consolidate the hotkey emissions into a single BTreeMap
                .fold(BTreeMap::new(), |mut acc, (hotkey, (terms, sub_weight))| {
                    acc.entry(hotkey)
                        .and_modify(|acc_terms| {
                            acc_terms.dividend = Self::weighted_acc_u16(
                                acc_terms.dividend,
                                terms.dividend,
                                sub_weight,
                            );
                            acc_terms.validator_emission = Self::weighted_acc_alpha(
                                acc_terms.validator_emission,
                                terms.validator_emission,
                                sub_weight,
                            );
                            acc_terms.server_emission = Self::weighted_acc_alpha(
                                acc_terms.server_emission,
                                terms.server_emission,
                                sub_weight,
                            );
                            acc_terms.stake_weight = Self::weighted_acc_u16(
                                acc_terms.stake_weight,
                                terms.stake_weight,
                                sub_weight,
                            );
                            acc_terms.active |= terms.active;
                            acc_terms.emission = Self::weighted_acc_alpha(
                                acc_terms.emission,
                                terms.emission,
                                sub_weight,
                            );
                            acc_terms.rank =
                                Self::weighted_acc_u16(acc_terms.rank, terms.rank, sub_weight);
                            acc_terms.trust =
                                Self::weighted_acc_u16(acc_terms.trust, terms.trust, sub_weight);
                            acc_terms.consensus = Self::weighted_acc_u16(
                                acc_terms.consensus,
                                terms.consensus,
                                sub_weight,
                            );
                            acc_terms.pruning_score = Self::weighted_acc_u16(
                                acc_terms.pruning_score,
                                terms.pruning_score,
                                sub_weight,
                            );
                            acc_terms.validator_trust = Self::weighted_acc_u16(
                                acc_terms.validator_trust,
                                terms.validator_trust,
                                sub_weight,
                            );
                            acc_terms.new_validator_permit |= terms.new_validator_permit;
                        })
                        .or_insert(terms);
                    acc
                });

        // State updates from epoch function
        Self::persist_netuid_epoch_terms(netuid, &aggregated);

        // Remap BTreeMap back to Vec<(T::AccountId, AlphaCurrency, AlphaCurrency)> format
        // for processing emissions in run_coinbase
        // Emission tuples ( hotkeys, server_emission, validator_emission )
        aggregated
            .into_iter()
            .map(|(hotkey, terms)| (hotkey, terms.server_emission, terms.validator_emission))
            .collect()
    }
}
