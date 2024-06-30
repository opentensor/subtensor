use super::*;
use crate::epoch::math::*;

impl<T: Config> Pallet<T> {
    // ==========================
    // ==== Helper functions ====
    // ==========================

    /// Checks if the provided version key is up-to-date for the given network.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID to check.
    /// * `version_key` - The version key to validate.
    ///
    /// # Returns
    ///
    /// Returns `true` if the version key is up-to-date, `false` otherwise.
    pub fn check_version_key(netuid: u16, version_key: u64) -> bool {
        let network_version_key: u64 = WeightsVersionKey::<T>::get(netuid);
        log::info!(
            "check_version_key( network_version_key:{:?}, version_key:{:?} )",
            network_version_key,
            version_key
        );
        network_version_key == 0 || version_key >= network_version_key
    }

    /// Checks if the neuron has set weights within the weights_set_rate_limit.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `neuron_uid` - The neuron's unique identifier.
    /// * `current_block` - The current block number.
    ///
    /// # Returns
    ///
    /// Returns `true` if the neuron can set weights, `false` otherwise.
    pub fn check_rate_limit(netuid: u16, neuron_uid: u16, current_block: u64) -> bool {
        if Self::is_uid_exist_on_network(netuid, neuron_uid) {
            let last_set_weights: u64 = Self::get_last_update_for_uid(netuid, neuron_uid);
            if last_set_weights == 0 {
                return true; // Never set weights before (Storage default)
            }
            return (current_block - last_set_weights) >= Self::get_weights_set_rate_limit(netuid);
        }
        false // Non-registered peers can't pass
    }

    /// Checks for any invalid UIDs on the given network.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID to check.
    /// * `uids` - A slice of UIDs to validate.
    ///
    /// # Returns
    ///
    /// Returns `true` if any UID is invalid, `false` if all are valid.
    pub fn contains_invalid_uids(netuid: u16, uids: &[u16]) -> bool {
        for uid in uids {
            if !Self::is_uid_exist_on_network(netuid, *uid) {
                log::debug!(
                    "contains_invalid_uids( netuid:{:?}, uid:{:?} does not exist on network. )",
                    netuid,
                    uids
                );
                return true;
            }
        }
        false
    }

    /// Checks if the number of UIDs matches the number of values.
    ///
    /// # Arguments
    ///
    /// * `uids` - A slice of UIDs.
    /// * `values` - A slice of values.
    ///
    /// # Returns
    ///
    /// Returns `true` if the lengths match, `false` otherwise.
    pub fn uids_match_values(uids: &[u16], values: &[u16]) -> bool {
        uids.len() == values.len()
    }

    /// Checks if the given slice of UIDs contains any duplicates.
    ///
    /// # Arguments
    ///
    /// * `items` - A slice of UIDs to check for duplicates.
    ///
    /// # Returns
    ///
    /// Returns `true` if duplicates are found, `false` otherwise.
    pub fn has_duplicate_uids(items: &[u16]) -> bool {
        let mut parsed: Vec<u16> = Vec::new();
        for item in items {
            if parsed.contains(item) {
                return true;
            }
            parsed.push(*item);
        }
        false
    }

    /// Checks if the neuron has validator permit or is setting self-weight.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `uid` - The neuron's UID.
    /// * `uids` - A slice of UIDs for the weights.
    /// * `weights` - A slice of weight values.
    ///
    /// # Returns
    ///
    /// Returns `true` if the neuron has permission, `false` otherwise.
    pub fn check_validator_permit(netuid: u16, uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }
        Self::get_validator_permit_for_uid(netuid, uid)
    }

    /// Checks if the number of weights is valid for the given network and neuron.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `uid` - The neuron's UID.
    /// * `uids` - A slice of UIDs for the weights.
    /// * `weights` - A slice of weight values.
    ///
    /// # Returns
    ///
    /// Returns `true` if the length is valid, `false` otherwise.
    pub fn check_length(netuid: u16, uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        let subnet_n: usize = Self::get_subnetwork_n(netuid) as usize;
        let min_allowed_length: usize = Self::get_min_allowed_weights(netuid) as usize;
        let min_allowed: usize = subnet_n.min(min_allowed_length);

        if netuid != Self::get_root_netuid() && Self::is_self_weight(uid, uids, weights) {
            return true;
        }
        weights.len() >= min_allowed
    }

    /// Normalizes the given weights so that they sum to u16::MAX.
    ///
    /// # Arguments
    ///
    /// * `weights` - A vector of weights to normalize.
    ///
    /// # Returns
    ///
    /// Returns a new vector of normalized weights.
    pub fn normalize_weights(mut weights: Vec<u16>) -> Vec<u16> {
        let sum: u64 = weights.iter().map(|x| *x as u64).sum();
        if sum == 0 {
            return weights;
        }
        weights.iter_mut().for_each(|x| {
            *x = (*x as u64 * u16::MAX as u64 / sum) as u16;
        });
        weights
    }

    /// Checks if the weights are within the maximum weight limit for the network.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `uid` - The neuron's UID.
    /// * `uids` - A slice of UIDs for the weights.
    /// * `weights` - A slice of weight values.
    ///
    /// # Returns
    ///
    /// Returns `true` if weights are within limits, `false` otherwise.
    pub fn max_weight_limited(netuid: u16, uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }

        let max_weight_limit: u16 = Self::get_max_weight_limit(netuid);
        if max_weight_limit == u16::MAX {
            return true;
        }

        check_vec_max_limited(weights, max_weight_limit)
    }

    /// Checks if the given UIDs and weights correspond to a self-weight.
    ///
    /// # Arguments
    ///
    /// * `uid` - The neuron's UID.
    /// * `uids` - A slice of UIDs for the weights.
    /// * `weights` - A slice of weight values.
    ///
    /// # Returns
    ///
    /// Returns `true` if it's a self-weight, `false` otherwise.
    pub fn is_self_weight(uid: u16, uids: &[u16], weights: &[u16]) -> bool {
        if weights.len() != 1 {
            return false;
        }
        let Some(first_uid) = uids.first() else {
            return false;
        };
        uid == *first_uid
    }

    /// Checks if the number of UIDs is within the allowed limit for the network.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `uids` - A slice of UIDs to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the number of UIDs is within limits, `false` otherwise.
    pub fn check_len_uids_within_allowed(netuid: u16, uids: &[u16]) -> bool {
        let subnetwork_n: u16 = Self::get_subnetwork_n(netuid);
        uids.len() <= subnetwork_n as usize
    }

    /// Checks if an account can commit weights for a given network.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `who` - The account ID trying to commit.
    ///
    /// # Returns
    ///
    /// Returns `true` if the account can commit, `false` otherwise.
    pub fn can_commit(netuid: u16, who: &T::AccountId) -> bool {
        if let Some((_hash, commit_block)) = WeightCommits::<T>::get(netuid, who) {
            let interval: u64 = Self::get_commit_reveal_weights_interval(netuid);
            if interval == 0 {
                return true; // Prevent division by 0
            }

            let current_block: u64 = Self::get_current_block_as_u64();
            let interval_start: u64 = current_block - (current_block % interval);
            let last_commit_interval_start: u64 = commit_block - (commit_block % interval);

            // Allow commit if we're within the interval bounds
            if current_block <= interval_start + interval
                && interval_start > last_commit_interval_start
            {
                return true;
            }

            false
        } else {
            true
        }
    }

    /// Checks if the current block is within the reveal range for a given commit.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `commit_block` - The block number when the commit was made.
    ///
    /// # Returns
    ///
    /// Returns `true` if it's within the reveal range, `false` otherwise.
    pub fn is_reveal_block_range(netuid: u16, commit_block: u64) -> bool {
        let interval: u64 = Self::get_commit_reveal_weights_interval(netuid);
        if interval == 0 {
            return true; // Prevent division by 0
        }

        let commit_interval_start: u64 = commit_block - (commit_block % interval);
        let reveal_interval_start: u64 = commit_interval_start + interval;
        let current_block: u64 = Self::get_current_block_as_u64();

        // Allow reveal if the current block is within the interval following the commit's interval
        current_block >= reveal_interval_start
            && current_block < reveal_interval_start + interval
    }
}

// TODO: Implement error handling for edge cases in weight calculations
// TODO: Add unit tests for each helper function to ensure correctness
// TODO: Consider optimizing performance for large networks with many UIDs
