use super::*;
use frame_support::weights::Weight;
use sp_core::Get;

impl<T: Config> Pallet<T> {
    pub fn do_root_claim(coldkey: T::AccountId) -> Weight {
        let mut weight = Weight::default();

        let hotkeys = StakingHotkeys::<T>::get(&coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        hotkeys.iter().for_each(|hotkey| {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            weight.saturating_accrue(Self::root_claim_all(hotkey, &coldkey));
        });

        weight.into()
    }

	pub fn block_hash_to_indices(block_hash: T::Hash, k: usize, n: usize) -> Vec<usize> {
		let block_hash_bytes = block_hash.as_bytes_slice();
		let mut indices = Vec::new();
		// k < n
		let start_index: usize = block_hash_bytes[0..8].to_vec().into();
		let mut last_idx = start_index;
		for i in 0..k {
			let bh_idx = (i * 8) % 32;
			let idx_step = block_hash_bytes[bh_idx..(bh_idx+8)].to_vec().into();
			let idx = (last_idx + idx_step) % n;
			indices.push(idx as usize);
			last_idx = idx;
		}
		indices
	}

	pub fn run_auto_claim_root_divs(last_block_hash: T::Hash) {

	}
}
