//! This file contains all tooling to work with sub-subnets
//!

use super::*;
use alloc::collections::BTreeMap;
use safe_math::*;
use sp_runtime::SaturatedConversion;
use subtensor_runtime_common::{AlphaCurrency, NetUid, SubId};

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
pub const GLOBAL_MAX_SUBNET_COUNT: u16 = 1024;

impl<T: Config> Pallet<T> {
    pub fn get_subsubnet_storage_index(netuid: NetUid, sub_id: SubId) -> NetUid {
        u16::from(sub_id)
            .saturating_mul(GLOBAL_MAX_SUBNET_COUNT)
            .saturating_add(u16::from(netuid))
            .into()
    }

    /// Set the desired valus of sub-subnet count for a subnet identified
    /// by netuid
    pub fn do_set_desired_subsubnet_count(netuid: NetUid, subsubnet_count: u8) -> DispatchResult {
        // Make sure the subnet exists
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // Count cannot be zero
        ensure!(subsubnet_count > 0, Error::<T>::InvalidValue);

        // Make sure we are not exceeding the max sub-subnet count
        ensure!(
            subsubnet_count <= MaxSubsubnetCount::<T>::get(),
            Error::<T>::InvalidValue
        );

        SubsubnetCountDesired::<T>::insert(netuid, subsubnet_count);
        Ok(())
    }

    /// Update current count for a subnet identified by netuid
    ///
    /// - This function should be called in every block in run_counbase
    /// - Cleans up all sub-subnet maps if count is reduced
    /// - Decreases current subsubnet count by no more than `GlobalSubsubnetDecreasePerSuperblock`
    ///
    pub fn update_subsubnet_counts_if_needed(current_block: u64) {
        // Run once per super-block
        let super_block_tempos = u64::from(SuperBlockTempos::<T>::get());
        Self::get_all_subnet_netuids().iter().for_each(|netuid| {
            let super_block = super_block_tempos.saturating_mul(u64::from(Tempo::<T>::get(netuid)));
            if let Some(rem) = current_block.checked_rem(super_block) {
                if rem == 0 {
                    let old_count = SubsubnetCountCurrent::<T>::get(netuid);
                    let desired_count = SubsubnetCountDesired::<T>::get(netuid);
                    let min_possible_count = old_count
                        .saturating_sub(GlobalSubsubnetDecreasePerSuperblock::<T>::get())
                        .max(1);
                    let new_count = desired_count.max(min_possible_count);

                    if old_count > new_count {

                        todo!();
                        // Cleanup weights
                        // Cleanup StakeWeight
                        // Cleanup Active
                        // Cleanup Emission
                        // Cleanup Rank
                        // Cleanup Trust
                        // Cleanup Consensus
                        // Cleanup Incentive
                        // Cleanup Dividends
                        // Cleanup PruningScores
                        // Cleanup ValidatorTrust
                        // Cleanup ValidatorPermit
                    }

                    SubsubnetCountCurrent::<T>::insert(netuid, new_count);
                }
            }
        });
    }

    /// Split alpha emission in sub-subnet proportions
    /// Currently splits evenly between sub-subnets, but the implementation
    /// may change in the future
    ///
    pub fn split_emissions(netuid: NetUid, alpha: AlphaCurrency) -> Vec<AlphaCurrency> {
        let subsubnet_count = u64::from(SubsubnetCountCurrent::<T>::get(netuid));

        // If there's any rounding error, credit it to subsubnet 0
        let per_subsubnet = u64::from(alpha).safe_div(subsubnet_count);
        let rounding_err =
            u64::from(alpha).saturating_sub(per_subsubnet.saturating_mul(subsubnet_count));

        let mut result = vec![AlphaCurrency::from(per_subsubnet); subsubnet_count as usize];
        result[0] = result[0].saturating_add(AlphaCurrency::from(rounding_err));
        result
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
        let aggregated: BTreeMap<T::AccountId, (AlphaCurrency, AlphaCurrency)> =
            Self::split_emissions(netuid, rao_emission)
                .into_iter()
                .enumerate()
                // Run epoch function for each subsubnet to distribute its portion of emissions
                .flat_map(|(sub_id, emission)| {
                    // This is subsubnet ID, e.g. a 0-7 number
                    let sub_id_u8: u8 = sub_id.saturated_into();
                    // This is netuid index for storing subsubnet data in storage maps and for using in
                    // epoch function
                    let subsub_netuid =
                        Self::get_subsubnet_storage_index(netuid, SubId::from(sub_id_u8));
                    // epoch returns: Vec<(T::AccountId, AlphaCurrency, AlphaCurrency)>
                    Self::epoch(subsub_netuid, emission).into_iter()
                })
                // Consolidate the hotkey emissions into a single BTreeMap
                .fold(BTreeMap::new(), |mut acc, (hotkey, divs, incs)| {
                    acc.entry(hotkey)
                        .and_modify(|tot| {
                            tot.0 = tot.0.saturating_add(divs);
                            tot.1 = tot.1.saturating_add(incs);
                        })
                        .or_insert((divs, incs));
                    acc
                });

        // Remap BTreeMap back to Vec<(T::AccountId, AlphaCurrency, AlphaCurrency)> format 
        // for processing in run_coinbase
        aggregated
            .into_iter()
            .map(|(hotkey, (divs, incs))| (hotkey, divs, incs))
            .collect()
    }
}
