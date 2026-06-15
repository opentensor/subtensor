use mev_shield_ibe_runtime_api::{DkgAuthorityInfo, DkgConsensusKeyKind, DkgConsensusSource};
use sp_core::{H256, sr25519};
use sp_runtime::{
    RuntimeAppPublic,
    traits::{UniqueSaturatedInto, Verify},
};
use sp_std::vec::Vec;

use crate::{AccountId32, MevShieldIbeEpochLength, NetUid, Runtime};

pub struct RuntimeIbeDkgAuthorityProvider;

#[derive(Clone)]
struct RootValidatorEntry {
    hotkey_id: Vec<u8>,
    stake: u128,
}

impl RuntimeIbeDkgAuthorityProvider {
    const POS_DKG_SNAPSHOT_OFFSET: u64 = pallet_shield::IBE_DKG_EPOCHS_AHEAD;

    fn current_ibe_epoch() -> u64 {
        let block_number: u64 =
            frame_system::Pallet::<Runtime>::block_number().unique_saturated_into();
        let epoch_len = MevShieldIbeEpochLength::get().max(1);
        block_number.checked_div(epoch_len).unwrap_or(0)
    }

    fn root_validator_entries() -> Vec<RootValidatorEntry> {
        let root = NetUid::ROOT;
        let permits = pallet_subtensor::Pallet::<Runtime>::get_validator_permit(root);
        let mut keys: Vec<(u16, AccountId32)> =
            pallet_subtensor::Keys::<Runtime>::iter_prefix(root).collect();
        keys.sort_by_key(|(uid, _)| *uid);

        let mut out = Vec::new();
        for (uid, hotkey) in keys {
            if !permits.get(usize::from(uid)).copied().unwrap_or(false) {
                continue;
            }

            let stake: u128 =
                pallet_subtensor::Pallet::<Runtime>::get_total_stake_for_hotkey(&hotkey)
                    .unique_saturated_into();
            if stake == 0 {
                continue;
            }

            let hotkey_id: Vec<u8> =
                <AccountId32 as core::convert::AsRef<[u8]>>::as_ref(&hotkey).to_vec();
            out.push(RootValidatorEntry { hotkey_id, stake });
        }
        out
    }

    fn consensus_authority_ids() -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        for authority in pallet_aura::Authorities::<Runtime>::get().into_inner() {
            let raw = authority.to_raw_vec();
            if !out
                .iter()
                .any(|known: &Vec<u8>| known.as_slice() == raw.as_slice())
            {
                out.push(raw);
            }
        }
        out
    }

    fn build_authorities(
        root_entries: &[RootValidatorEntry],
        authority_ids: &[Vec<u8>],
        consensus_key_kind: DkgConsensusKeyKind,
    ) -> Vec<DkgAuthorityInfo> {
        if authority_ids.is_empty() {
            return Vec::new();
        }

        let mut out = Vec::new();

        if root_entries.is_empty() {
            for authority_id in authority_ids.iter() {
                out.push(DkgAuthorityInfo {
                    hotkey_account_id: authority_id.clone(),
                    consensus_key_kind: consensus_key_kind.clone(),
                    authority_id: authority_id.clone(),
                    stake: 1,
                    // DKG transport keys are intentionally not a runtime-registration surface.
                    // Validator nodes overlay signed X25519 transport keys through the DKG P2P
                    // protocol before dealing.
                    dkg_x25519_public_key: [0u8; 32],
                });
            }
            out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
            return out;
        }

        let count = core::cmp::min(root_entries.len(), authority_ids.len());
        for index in 0..count {
            let root = &root_entries[index];
            out.push(DkgAuthorityInfo {
                hotkey_account_id: root.hotkey_id.clone(),
                consensus_key_kind: consensus_key_kind.clone(),
                authority_id: authority_ids[index].clone(),
                stake: root.stake,
                // DKG transport keys are intentionally not a runtime-registration surface.
                // Validator nodes overlay signed X25519 transport keys through the DKG P2P
                // protocol before dealing.
                dkg_x25519_public_key: [0u8; 32],
            });
        }

        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        out
    }

    fn epoch_uses_pos_babe(epoch: u64) -> bool {
        epoch >= Self::current_ibe_epoch().saturating_add(Self::POS_DKG_SNAPSHOT_OFFSET)
    }

    fn poa_authorities(
        root_entries: &[RootValidatorEntry],
        authority_ids: &[Vec<u8>],
    ) -> Vec<DkgAuthorityInfo> {
        Self::build_authorities(
            root_entries,
            authority_ids,
            DkgConsensusKeyKind::AuraSr25519,
        )
    }

    fn pos_babe_authorities(
        root_entries: &[RootValidatorEntry],
        authority_ids: &[Vec<u8>],
    ) -> Vec<DkgAuthorityInfo> {
        // During the staged PoA->PoS handoff the initial BABE authority cohort is the
        // existing PoA/Aura cohort. The node service copies Aura sr25519 keys into the
        // BABE keystore, so no on-chain DKG authority-registration extrinsic is needed.
        // Later validator additions enter automatically when they are present in the
        // consensus authority set before a future N+2 snapshot is frozen.
        Self::build_authorities(
            root_entries,
            authority_ids,
            DkgConsensusKeyKind::BabeSr25519,
        )
    }

    fn verify_sr25519(public: &[u8], payload_hash: H256, signature: &[u8]) -> bool {
        let Ok(public_bytes) = <[u8; 32]>::try_from(public) else {
            return false;
        };
        let Ok(signature_bytes) = <[u8; 64]>::try_from(signature) else {
            return false;
        };

        let signature = sr25519::Signature::from_raw(signature_bytes);
        let public = sr25519::Public::from_raw(public_bytes);
        signature.verify(&payload_hash.as_fixed_bytes()[..], &public)
    }
}

impl pallet_shield::IbeDkgAuthorityProvider for RuntimeIbeDkgAuthorityProvider {
    fn authorities_for_epoch(epoch: u64) -> Vec<DkgAuthorityInfo> {
        let root_entries = Self::root_validator_entries();
        let authority_ids = Self::consensus_authority_ids();

        if Self::epoch_uses_pos_babe(epoch) {
            let pos_authorities = Self::pos_babe_authorities(&root_entries, &authority_ids);
            if !pos_authorities.is_empty() {
                return pos_authorities;
            }
        }

        Self::poa_authorities(&root_entries, &authority_ids)
    }

    fn consensus_source_for_epoch(epoch: u64) -> DkgConsensusSource {
        if Self::epoch_uses_pos_babe(epoch) && !Self::consensus_authority_ids().is_empty() {
            DkgConsensusSource::PosBabeRootValidators
        } else {
            DkgConsensusSource::PoaAuraRootValidators
        }
    }

    fn verify_authority_signature(
        authority_id: &[u8],
        payload_hash: H256,
        signature: &[u8],
    ) -> bool {
        Self::verify_sr25519(authority_id, payload_hash, signature)
    }
}
