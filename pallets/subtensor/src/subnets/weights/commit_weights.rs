use super::*;
use sp_core::H256;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for committing weight hashes.
    ///
    /// # Args:
    /// * `origin`: (`<T as frame_system::Config>::RuntimeOrigin`):
    ///   - The signature of the committing hotkey.
    ///
    /// * `netuid` (`u16`):
    ///   - The u16 network identifier.
    ///
    /// * `commit_hash` (`H256`):
    ///   - The hash representing the committed weights.
    ///
    /// # Raises:
    /// * `WeightsCommitNotAllowed`:
    ///   - Attempting to commit when it is not allowed.
    ///
    pub fn do_commit_weights(
        origin: T::RuntimeOrigin,
        netuid: u16,
        commit_hash: H256,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        log::info!("do_commit_weights( hotkey:{:?} netuid:{:?})", who, netuid);

        ensure!(
            Self::get_commit_reveal_weights_enabled(netuid),
            Error::<T>::CommitRevealDisabled
        );

        ensure!(
            Self::can_commit(netuid, &who),
            Error::<T>::WeightsCommitNotAllowed
        );

        WeightCommits::<T>::insert(
            netuid,
            &who,
            (commit_hash, Self::get_current_block_as_u64()),
        );
        Ok(())
    }
}