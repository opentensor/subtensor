use super::*;
use sp_core::{H256, U256};

impl<T: Config> Pallet<T> {

    pub fn do_faucet(
        origin: T::RuntimeOrigin,
        block_number: u64,
        nonce: u64,
        work: Vec<u8>,
    ) -> DispatchResult {
        // --- 0. Ensure the faucet is enabled.
        // ensure!(AllowFaucet::<T>::get(), Error::<T>::FaucetDisabled);

        // --- 1. Check that the caller has signed the transaction.
        let coldkey = ensure_signed(origin)?;
        log::info!("do_faucet( coldkey:{:?} )", coldkey);

        // --- 2. Ensure the passed block number is valid, not in the future or too old.
        // Work must have been done within 3 blocks (stops long range attacks).
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure!(
            block_number <= current_block_number,
            Error::<T>::InvalidWorkBlock
        );
        ensure!(
            current_block_number - block_number < 3,
            Error::<T>::InvalidWorkBlock
        );

        // --- 3. Ensure the supplied work passes the difficulty.
        let difficulty: U256 = U256::from(1_000_000); // Base faucet difficulty.
        let work_hash: H256 = Self::vec_to_hash(work.clone());
        ensure!(
            Self::hash_meets_difficulty(&work_hash, difficulty),
            Error::<T>::InvalidDifficulty
        ); // Check that the work meets difficulty.

        // --- 4. Check Work is the product of the nonce, the block number, and hotkey. Add this as used work.
        let seal: H256 = Self::create_seal_hash(block_number, nonce, &coldkey);
        ensure!(seal == work_hash, Error::<T>::InvalidSeal);
        UsedWork::<T>::insert(work.clone(), current_block_number);

        // --- 5. Add Balance via faucet.
        let balance_to_add: u64 = 100_000_000_000;
        Self::coinbase(100_000_000_000); // We are creating tokens here from the coinbase.

        Self::add_balance_to_coldkey_account(&coldkey, balance_to_add);

        // --- 6. Deposit successful event.
        log::info!(
            "Faucet( coldkey:{:?} amount:{:?} ) ",
            coldkey,
            balance_to_add
        );
        Self::deposit_event(Event::Faucet(coldkey, balance_to_add));

        // --- 7. Ok and done.
        Ok(())
    }
}