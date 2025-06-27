use super::*;
use ark_serialize::CanonicalDeserialize;
use codec::Decode;
use frame_support::dispatch;
use frame_support::traits::OriginTrait;
use subtensor_runtime_common::NetUid;
use tle::curves::drand::TinyBLS381;
use tle::stream_ciphers::AESGCMStreamCipherProvider;
use tle::tlock::TLECiphertext;
use tle::tlock::tld;
use w3f_bls::EngineBLS;

/// Contains all necesarry information to set weights.
///
/// In the context of commit-reveal v3, this is the payload which should be
/// encrypted, compressed, serialized, and submitted to the `commit_crv3_weights`
/// extrinsic.
#[derive(Encode, Decode)]
#[freeze_struct("46e75a8326ba3665")]
pub struct WeightsTlockPayload {
    pub uids: Vec<u16>,
    pub values: Vec<u16>,
    pub version_key: u64,
}

impl<T: Config> Pallet<T> {
    /// The `reveal_crv3_commits` function is run at the very beginning of epoch `n`,
    pub fn reveal_crv3_commits(netuid: NetUid) -> dispatch::DispatchResult {
        let cur_block = Self::get_current_block_as_u64();
        let cur_epoch = Self::get_epoch_index(netuid, cur_block);

        // Weights revealed must have been committed during epoch `cur_epoch - reveal_period`.
        let reveal_epoch =
            cur_epoch.saturating_sub(Self::get_reveal_period(netuid).saturating_sub(1));

        // Clean expired commits
        for (epoch, _) in CRV3WeightCommits::<T>::iter_prefix(netuid) {
            if epoch < reveal_epoch {
                CRV3WeightCommits::<T>::remove(netuid, epoch);
            }
        }

        // No commits to reveal until at least epoch 2.
        if cur_epoch < 2 {
            log::warn!("Failed to reveal commit for subnet {} Too early", netuid);
            return Ok(());
        }

        let mut entries = CRV3WeightCommits::<T>::take(netuid, reveal_epoch);

        // Keep popping item off the end of the queue until we sucessfully reveal a commit.
        while let Some((who, serialized_compresssed_commit, round_number)) = entries.pop_front() {
            let reader = &mut &serialized_compresssed_commit[..];
            let commit = match TLECiphertext::<TinyBLS381>::deserialize_compressed(reader) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!(
                        "Failed to reveal commit for subnet {} submitted by {:?} due to error deserializing the commit: {:?}",
                        netuid,
                        who,
                        e
                    );
                    continue;
                }
            };

            // Try to get the round number from pallet_drand.
            let pulse = match pallet_drand::Pulses::<T>::get(round_number) {
                Some(p) => p,
                None => {
                    // Round number used was not found on the chain. Skip this commit.
                    log::warn!(
                        "Failed to reveal commit for subnet {} submitted by {:?} due to missing round number {} at time of reveal.",
                        netuid,
                        who,
                        round_number
                    );
                    continue;
                }
            };

            let signature_bytes = pulse
                .signature
                .strip_prefix(b"0x")
                .unwrap_or(&pulse.signature);

            let sig_reader = &mut &signature_bytes[..];
            let sig = match <TinyBLS381 as EngineBLS>::SignatureGroup::deserialize_compressed(
                sig_reader,
            ) {
                Ok(s) => s,
                Err(e) => {
                    log::error!(
                        "Failed to reveal commit for subnet {} submitted by {:?} due to error deserializing signature from drand pallet: {:?}",
                        netuid,
                        who,
                        e
                    );
                    continue;
                }
            };

            let decrypted_bytes: Vec<u8> = match tld::<TinyBLS381, AESGCMStreamCipherProvider>(
                commit, sig,
            ) {
                Ok(d) => d,
                Err(e) => {
                    log::warn!(
                        "Failed to reveal commit for subnet {} submitted by {:?} due to error decrypting the commit: {:?}",
                        netuid,
                        who,
                        e
                    );
                    continue;
                }
            };

            // Decrypt the bytes into WeightsPayload
            let mut reader = &decrypted_bytes[..];
            let payload: WeightsTlockPayload = match Decode::decode(&mut reader) {
                Ok(w) => w,
                Err(e) => {
                    log::warn!(
                        "Failed to reveal commit for subnet {} submitted by {:?} due to error deserializing WeightsPayload: {:?}",
                        netuid,
                        who,
                        e
                    );
                    continue;
                }
            };

            if let Err(e) = Self::do_set_weights(
                T::RuntimeOrigin::signed(who.clone()),
                netuid,
                payload.uids,
                payload.values,
                payload.version_key,
            ) {
                log::warn!(
                    "Failed to `do_set_weights` for subnet {} submitted by {:?}: {:?}",
                    netuid,
                    who,
                    e
                );
                continue;
            } else {
                Self::deposit_event(Event::CRV3WeightsRevealed(netuid, who));
            };
        }

        Ok(())
    }
}
