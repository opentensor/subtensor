use frame_support::storage::IterableStorageDoubleMap;
use mev_shield_ibe_runtime_api::{DkgAuthorityInfo, DkgConsensusKeyKind, DkgConsensusSource};
use sp_core::{H256, sr25519};
use sp_runtime::{
    RuntimeAppPublic,
    traits::{UniqueSaturatedInto, Verify},
};

use sp_std::vec::Vec;

use crate::{AccountId32, NetUid, Runtime};

pub struct RuntimeIbeDkgAuthorityProvider;

#[derive(Clone)]
struct RootValidatorEntry {
    hotkey_id: Vec<u8>,
    stake: u128,
}

impl RuntimeIbeDkgAuthorityProvider {
    fn root_validator_entries() -> Vec<RootValidatorEntry> {
        let root = NetUid::ROOT;
        let permits = pallet_subtensor::Pallet::<Runtime>::get_validator_permit(root);
        let mut keys: Vec<(u16, AccountId32)> =
            <pallet_subtensor::Keys<Runtime> as IterableStorageDoubleMap<
                NetUid,
                u16,
                AccountId32,
            >>::iter_prefix(root)
            .collect();
        keys.sort_by_key(|(uid, _)| *uid);

        let mut out = Vec::new();
        for (uid, hotkey) in keys {
            if !permits.get(uid as usize).copied().unwrap_or(false) {
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

    fn aura_authority_ids() -> Vec<Vec<u8>> {
        let mut authority_ids = pallet_aura::Authorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|authority| authority.to_raw_vec())
            .collect::<Vec<_>>();
        authority_ids.sort();
        authority_ids
    }
    fn poa_handoff_root_entries(
        root_entries: &[RootValidatorEntry],
        aura_authority_ids: &[Vec<u8>],
    ) -> Vec<RootValidatorEntry> {
        if root_entries.is_empty() || aura_authority_ids.is_empty() {
            return Vec::new();
        }

        root_entries
            .iter()
            .take(aura_authority_ids.len())
            .cloned()
            .collect()
    }

    fn automatic_consensus_authorities(
        root_entries: &[RootValidatorEntry],
    ) -> Vec<DkgAuthorityInfo> {
        let authority_ids = Self::aura_authority_ids();
        let handoff_entries = Self::poa_handoff_root_entries(root_entries, &authority_ids);
        let mut out = if !handoff_entries.is_empty() && handoff_entries.len() == authority_ids.len()
        {
            authority_ids
                .into_iter()
                .zip(handoff_entries.iter())
                .map(|(authority_id, root)| DkgAuthorityInfo {
                    hotkey_account_id: root.hotkey_id.clone(),
                    consensus_key_kind: DkgConsensusKeyKind::AuraSr25519,
                    authority_id,
                    stake: root.stake,
                    // Transport keys are not runtime registration data. Nodes
                    // overlay signed DKG transport-key gossip before dealing.
                    dkg_x25519_public_key: [0u8; 32],
                })
                .collect::<Vec<_>>()
        } else {
            authority_ids
                .into_iter()
                .map(|authority_id| DkgAuthorityInfo {
                    hotkey_account_id: authority_id.clone(),
                    consensus_key_kind: DkgConsensusKeyKind::AuraSr25519,
                    authority_id,
                    stake: 1,
                    dkg_x25519_public_key: [0u8; 32],
                })
                .collect::<Vec<_>>()
        };
        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        out
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
    fn authorities_for_epoch(_: u64) -> Vec<DkgAuthorityInfo> {
        Self::automatic_consensus_authorities(&Self::root_validator_entries())
    }

    fn consensus_source_for_epoch(_epoch: u64) -> DkgConsensusSource {
        DkgConsensusSource::PoaAuraRootValidators
    }

    fn verify_authority_signature(
        authority_id: &[u8],
        payload_hash: H256,
        signature: &[u8],
    ) -> bool {
        Self::verify_sr25519(authority_id, payload_hash, signature)
    }
}
