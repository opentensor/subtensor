use super::*;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};

impl<T: Config> Pallet<T> {
// ---- The implementation for revealing committed weights.
    ///
    /// # Args:
    /// * `origin`: (`<T as frame_system::Config>::RuntimeOrigin`):
    ///   - The signature of the revealing hotkey.
    ///
    /// * `netuid` (`u16`):
    ///   - The u16 network identifier.
    ///
    /// * `uids` (`Vec<u16>`):
    ///   - The uids for the weights being revealed.
    ///
    /// * `values` (`Vec<u16>`):
    ///   - The values of the weights being revealed.
    ///
    /// * `salt` (`Vec<u8>`):
    ///   - The values of the weights being revealed.
    ///
    /// * `version_key` (`u64`):
    ///   - The network version key.
    ///
    /// # Raises:
    /// * `NoWeightsCommitFound`:
    ///   - Attempting to reveal weights without an existing commit.
    ///
    /// * `InvalidRevealCommitHashNotMatchTempo`:
    ///   - Attempting to reveal weights outside the valid tempo.
    ///
    /// * `InvalidRevealCommitHashNotMatch`:
    ///   - The revealed hash does not match the committed hash.
    ///
    pub fn do_reveal_weights(
        origin: T::RuntimeOrigin,
        netuid: u16,
        uids: Vec<u16>,
        values: Vec<u16>,
        salt: Vec<u16>,
        version_key: u64,
    ) -> DispatchResult {
        let who = ensure_signed(origin.clone())?;

        log::info!("do_reveal_weights( hotkey:{:?} netuid:{:?})", who, netuid);

        ensure!(
            Self::get_commit_reveal_weights_enabled(netuid),
            Error::<T>::CommitRevealDisabled
        );

        WeightCommits::<T>::try_mutate_exists(netuid, &who, |maybe_commit| -> DispatchResult {
            let (commit_hash, commit_block) = maybe_commit
                .as_ref()
                .ok_or(Error::<T>::NoWeightsCommitFound)?;

            ensure!(
                Self::is_reveal_block_range(netuid, *commit_block),
                Error::<T>::InvalidRevealCommitTempo
            );

            let provided_hash: H256 = BlakeTwo256::hash_of(&(
                who.clone(),
                netuid,
                uids.clone(),
                values.clone(),
                salt.clone(),
                version_key,
            ));
            ensure!(
                provided_hash == *commit_hash,
                Error::<T>::InvalidRevealCommitHashNotMatch
            );

            Self::do_set_weights(origin, netuid, uids, values, version_key)
        })
    }
}