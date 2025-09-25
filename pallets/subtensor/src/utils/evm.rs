use super::*;
use alloc::string::ToString;
use frame_support::ensure;
use frame_system::ensure_signed;

use sp_core::{H160, ecdsa::Signature, hashing::keccak_256};
use sp_std::vec::Vec;
use subtensor_runtime_common::NetUid;

const MESSAGE_PREFIX: &str = "\x19Ethereum Signed Message:\n";

impl<T: Config> Pallet<T> {
    pub(crate) fn hash_message_eip191<M: AsRef<[u8]>>(message: M) -> [u8; 32] {
        let msg_len = message.as_ref().len().to_string();
        keccak_256(
            &[
                MESSAGE_PREFIX.as_bytes(),
                msg_len.as_bytes(),
                message.as_ref(),
            ]
            .concat(),
        )
    }

    /// Associate an EVM key with a hotkey.
    ///
    /// This function accepts a Signature, which is a signed message containing the hotkey
    /// concatenated with the hashed block number. It will then attempt to recover the EVM key from
    /// the signature and compare it with the `evm_key` parameter, and ensures that they match.
    ///
    /// The EVM key is expected to sign the message according to this formula to produce the
    /// signature:
    /// ```text
    /// keccak_256(hotkey ++ keccak_256(block_number))
    /// ```
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call, which should be the coldkey that owns the hotkey.
    /// * `netuid` - The unique identifier for the subnet that the hotkey belongs to.
    /// * `hotkey` - The hotkey associated with the `origin` coldkey.
    /// * `evm_key` - The EVM address to associate with the `hotkey`.
    /// * `block_number` - The block number used in the `signature`.
    /// * `signature` - A signed message by the `evm_key` containing the `hotkey` and the hashed
    ///   `block_number`.
    pub fn do_associate_evm_key(
        origin: T::RuntimeOrigin,
        netuid: NetUid,
        evm_key: H160,
        block_number: u64,
        mut signature: Signature,
    ) -> dispatch::DispatchResult {
        let hotkey = ensure_signed(origin)?;

        // Normalize the v value to 0 or 1
        if signature.0[64] >= 27 {
            signature.0[64] = signature.0[64].saturating_sub(27);
        }

        let uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey)?;
        Self::ensure_evm_key_associate_rate_limit(netuid, uid)?;

        let block_hash = keccak_256(block_number.encode().as_ref());
        let message = [hotkey.encode().as_ref(), block_hash.as_ref()].concat();
        let public = signature
            .recover_prehashed(&Self::hash_message_eip191(message))
            .ok_or(Error::<T>::InvalidIdentity)?;
        let secp_pubkey = libsecp256k1::PublicKey::parse_compressed(&public.0)
            .map_err(|_| Error::<T>::UnableToRecoverPublicKey)?;
        let uncompressed = secp_pubkey.serialize();
        let hashed_evm_key = H160::from_slice(&keccak_256(&uncompressed[1..])[12..]);

        ensure!(
            evm_key == hashed_evm_key,
            Error::<T>::InvalidRecoveredPublicKey
        );

        let current_block_number = Self::get_current_block_as_u64();

        AssociatedEvmAddress::<T>::insert(netuid, uid, (evm_key, current_block_number));

        Self::deposit_event(Event::EvmKeyAssociated {
            netuid,
            hotkey,
            evm_key,
            block_associated: current_block_number,
        });

        Ok(())
    }

    pub fn uid_lookup(netuid: NetUid, evm_key: H160, limit: u16) -> Vec<(u16, u64)> {
        let mut ret_val = AssociatedEvmAddress::<T>::iter_prefix(netuid)
            .take(limit as usize)
            .filter_map(|(uid, (stored_evm_key, block_associated))| {
                if stored_evm_key != evm_key {
                    return None;
                }

                Some((uid, block_associated))
            })
            .collect::<Vec<(u16, u64)>>();
        ret_val.sort_by(|(_, block1), (_, block2)| block1.cmp(block2));
        ret_val
    }

    pub fn ensure_evm_key_associate_rate_limit(netuid: NetUid, uid: u16) -> DispatchResult {
        let now = Self::get_current_block_as_u64();
        let block_associated = match AssociatedEvmAddress::<T>::get(netuid, uid) {
            Some((_, block_associated)) => block_associated,
            None => 0,
        };
        let block_diff = now.saturating_sub(block_associated);
        if block_diff < T::EvmKeyAssociateRateLimit::get() {
            return Err(Error::<T>::EvmKeyAssociateRateLimitExceeded.into());
        }
        Ok(())
    }
}
