use super::*;
use sp_core::{H256, U256};
use sp_io::hashing::{keccak_256, sha2_256};
use system::pallet_prelude::BlockNumberFor;
const LOG_TARGET: &str = "runtime::subtensor::registration";

impl<T: Config> Pallet<T> {
    pub fn vec_to_hash(vec_hash: Vec<u8>) -> H256 {
        let de_ref_hash = &vec_hash; // b: &Vec<u8>
        let de_de_ref_hash: &[u8] = de_ref_hash; // c: &[u8]
        let real_hash: H256 = H256::from_slice(de_de_ref_hash);
        real_hash
    }

    /// Determine which peer to prune from the network by finding the element with the lowest pruning score out of
    /// immunity period. If all neurons are in immunity period, return node with lowest prunning score.
    /// This function will always return an element to prune.
    pub fn get_neuron_to_prune(netuid: u16) -> u16 {
        let mut min_score: u16 = u16::MAX;
        let mut min_score_in_immunity_period = u16::MAX;
        let mut uid_with_min_score = 0;
        let mut uid_with_min_score_in_immunity_period: u16 = 0;

        let neurons_n = Self::get_subnetwork_n(netuid);
        if neurons_n == 0 {
            return 0; // If there are no neurons in this network.
        }

        let current_block: u64 = Self::get_current_block_as_u64();
        let immunity_period: u64 = Self::get_immunity_period(netuid) as u64;
        for neuron_uid_i in 0..neurons_n {
            let pruning_score: u16 = Self::get_pruning_score_for_uid(netuid, neuron_uid_i);
            let block_at_registration: u64 =
                Self::get_neuron_block_at_registration(netuid, neuron_uid_i);
            #[allow(clippy::comparison_chain)]
            if min_score == pruning_score {
                if current_block - block_at_registration < immunity_period {
                    //neuron is in immunity period
                    if min_score_in_immunity_period > pruning_score {
                        min_score_in_immunity_period = pruning_score;
                        uid_with_min_score_in_immunity_period = neuron_uid_i;
                    }
                } else {
                    uid_with_min_score = neuron_uid_i;
                }
            }
            // Find min pruning score.
            else if min_score > pruning_score {
                if current_block - block_at_registration < immunity_period {
                    //neuron is in immunity period
                    if min_score_in_immunity_period > pruning_score {
                        min_score_in_immunity_period = pruning_score;
                        uid_with_min_score_in_immunity_period = neuron_uid_i;
                    }
                } else {
                    min_score = pruning_score;
                    uid_with_min_score = neuron_uid_i;
                }
            }
        }
        if min_score == u16::MAX {
            //all neuorns are in immunity period
            Self::set_pruning_score_for_uid(
                netuid,
                uid_with_min_score_in_immunity_period,
                u16::MAX,
            );
            uid_with_min_score_in_immunity_period
        } else {
            // We replace the pruning score here with u16 max to ensure that all peers always have a
            // pruning score. In the event that every peer has been pruned this function will prune
            // the last element in the network continually.
            Self::set_pruning_score_for_uid(netuid, uid_with_min_score, u16::MAX);
            uid_with_min_score
        }
    }

    /// Determine whether the given hash satisfies the given difficulty.
    /// The test is done by multiplying the two together. If the product
    /// overflows the bounds of U256, then the product (and thus the hash)
    /// was too high.
    pub fn hash_meets_difficulty(hash: &H256, difficulty: U256) -> bool {
        let bytes: &[u8] = hash.as_bytes();
        let num_hash: U256 = U256::from(bytes);
        let (value, overflowed) = num_hash.overflowing_mul(difficulty);

        log::trace!(
            target: LOG_TARGET,
            "Difficulty: hash: {:?}, hash_bytes: {:?}, hash_as_num: {:?}, difficulty: {:?}, value: {:?} overflowed: {:?}",
            hash,
            bytes,
            num_hash,
            difficulty,
            value,
            overflowed
        );
        !overflowed
    }

    pub fn get_block_hash_from_u64(block_number: u64) -> H256 {
        let block_number: BlockNumberFor<T> = TryInto::<BlockNumberFor<T>>::try_into(block_number)
            .ok()
            .expect("convert u64 to block number.");
        let block_hash_at_number: <T as frame_system::Config>::Hash =
            system::Pallet::<T>::block_hash(block_number);
        let vec_hash: Vec<u8> = block_hash_at_number.as_ref().to_vec();
        let deref_vec_hash: &[u8] = &vec_hash; // c: &[u8]
        let real_hash: H256 = H256::from_slice(deref_vec_hash);

        log::trace!(
            target: LOG_TARGET,
            "block_number: {:?}, vec_hash: {:?}, real_hash: {:?}",
            block_number,
            vec_hash,
            real_hash
        );

        real_hash
    }

    pub fn hash_to_vec(hash: H256) -> Vec<u8> {
        let hash_as_bytes: &[u8] = hash.as_bytes();
        let hash_as_vec: Vec<u8> = hash_as_bytes.to_vec();
        hash_as_vec
    }

    pub fn hash_block_and_hotkey(block_hash_bytes: &[u8; 32], hotkey: &T::AccountId) -> H256 {
        let binding = hotkey.encode();
        // Safe because Substrate guarantees that all AccountId types are at least 32 bytes
        let (hotkey_bytes, _) = binding.split_at(32);
        let mut full_bytes = [0u8; 64];
        let (first_half, second_half) = full_bytes.split_at_mut(32);
        first_half.copy_from_slice(block_hash_bytes);
        second_half.copy_from_slice(hotkey_bytes);
        let keccak_256_seal_hash_vec: [u8; 32] = keccak_256(&full_bytes[..]);

        H256::from_slice(&keccak_256_seal_hash_vec)
    }

    pub fn hash_hotkey_to_u64(hotkey: &T::AccountId) -> u64 {
        let binding = hotkey.encode();
        let (hotkey_bytes, _) = binding.split_at(32);
        let mut full_bytes = [0u8; 64];
        // Copy the hotkey_bytes into the first half of full_bytes
        full_bytes[..32].copy_from_slice(hotkey_bytes);
        let keccak_256_seal_hash_vec: [u8; 32] = keccak_256(&full_bytes[..]);
        let hash_u64: u64 = u64::from_le_bytes(keccak_256_seal_hash_vec[0..8].try_into().unwrap());
        hash_u64
    }

    pub fn create_seal_hash(block_number_u64: u64, nonce_u64: u64, hotkey: &T::AccountId) -> H256 {
        let nonce = nonce_u64.to_le_bytes();
        let block_hash_at_number: H256 = Self::get_block_hash_from_u64(block_number_u64);
        let block_hash_bytes: &[u8; 32] = block_hash_at_number.as_fixed_bytes();
        let binding = Self::hash_block_and_hotkey(block_hash_bytes, hotkey);
        let block_and_hotkey_hash_bytes: &[u8; 32] = binding.as_fixed_bytes();

        let mut full_bytes = [0u8; 40];
        let (first_chunk, second_chunk) = full_bytes.split_at_mut(8);
        first_chunk.copy_from_slice(&nonce);
        second_chunk.copy_from_slice(block_and_hotkey_hash_bytes);
        let sha256_seal_hash_vec: [u8; 32] = sha2_256(&full_bytes[..]);
        let keccak_256_seal_hash_vec: [u8; 32] = keccak_256(&sha256_seal_hash_vec);
        let seal_hash: H256 = H256::from_slice(&keccak_256_seal_hash_vec);

        log::trace!(
            "\n hotkey:{:?} \nblock_number: {:?}, \nnonce_u64: {:?}, \nblock_hash: {:?}, \nfull_bytes: {:?}, \nsha256_seal_hash_vec: {:?},  \nkeccak_256_seal_hash_vec: {:?}, \nseal_hash: {:?}",
            hotkey,
            block_number_u64,
            nonce_u64,
            block_hash_at_number,
            full_bytes,
            sha256_seal_hash_vec,
            keccak_256_seal_hash_vec,
            seal_hash
        );

        seal_hash
    }

    /// Helper function for creating nonce and work.
    pub fn create_work_for_block_number(
        netuid: u16,
        block_number: u64,
        start_nonce: u64,
        hotkey: &T::AccountId,
    ) -> (u64, Vec<u8>) {
        let difficulty: U256 = Self::get_difficulty(netuid);
        let mut nonce: u64 = start_nonce;
        let mut work: H256 = Self::create_seal_hash(block_number, nonce, hotkey);
        while !Self::hash_meets_difficulty(&work, difficulty) {
            nonce += 1;
            work = Self::create_seal_hash(block_number, nonce, hotkey);
        }
        let vec_work: Vec<u8> = Self::hash_to_vec(work);
        (nonce, vec_work)
    }
}