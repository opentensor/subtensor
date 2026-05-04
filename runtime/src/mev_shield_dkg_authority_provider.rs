use frame_support::storage::IterableStorageDoubleMap;
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
    fn epoch_length() -> u64 {
        MevShieldIbeEpochLength::get().max(1)
    }

    fn current_epoch() -> u64 {
        let block_number: u64 =
            frame_system::Pallet::<Runtime>::block_number().unique_saturated_into();
        block_number / Self::epoch_length()
    }

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

            let hotkey_bytes: &[u8] = hotkey.as_ref();
            out.push(RootValidatorEntry {
                hotkey_id: hotkey_bytes.to_vec(),
                stake,
            });
        }
        out
    }

    fn source_for_epoch(epoch: u64) -> DkgConsensusSource {
        if epoch >= Self::current_epoch().saturating_add(2) {
            DkgConsensusSource::PosBabeRootValidators
        } else {
            DkgConsensusSource::PoaAuraRootValidators
        }
    }

    fn key_kind_for_source(source: DkgConsensusSource) -> DkgConsensusKeyKind {
        match source {
            DkgConsensusSource::PosBabeRootValidators => DkgConsensusKeyKind::BabeSr25519,
            DkgConsensusSource::PoaAuraRootValidators => DkgConsensusKeyKind::AuraSr25519,
        }
    }

    fn authority_ids_for_source(_source: DkgConsensusSource) -> Vec<Vec<u8>> {
        pallet_aura::Authorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|authority| authority.to_raw_vec())
            .collect()
    }

    fn selected_authorities_for_epoch(epoch: u64) -> (DkgConsensusSource, Vec<DkgAuthorityInfo>) {
        let source = Self::source_for_epoch(epoch);
        let key_kind = Self::key_kind_for_source(source);
        let authority_ids = Self::authority_ids_for_source(source);
        let root_entries = Self::root_validator_entries();

        let mut out = if root_entries.is_empty() {
            authority_ids
                .into_iter()
                .map(|authority_id| DkgAuthorityInfo {
                    hotkey_account_id: authority_id.clone(),
                    consensus_key_kind: key_kind,
                    authority_id,
                    stake: 1,
                    dkg_x25519_public_key: [0u8; 32],
                })
                .collect::<Vec<_>>()
        } else if root_entries.len() == authority_ids.len() {
            authority_ids
                .into_iter()
                .zip(root_entries)
                .map(|(authority_id, root)| DkgAuthorityInfo {
                    hotkey_account_id: root.hotkey_id,
                    consensus_key_kind: key_kind,
                    authority_id,
                    stake: root.stake,
                    dkg_x25519_public_key: [0u8; 32],
                })
                .collect::<Vec<_>>()
        } else {
            authority_ids
                .into_iter()
                .map(|authority_id| DkgAuthorityInfo {
                    hotkey_account_id: authority_id.clone(),
                    consensus_key_kind: key_kind,
                    authority_id,
                    stake: 1,
                    dkg_x25519_public_key: [0u8; 32],
                })
                .collect::<Vec<_>>()
        };

        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        (source, out)
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
        Self::selected_authorities_for_epoch(epoch).1
    }

    fn consensus_source_for_epoch(epoch: u64) -> DkgConsensusSource {
        Self::selected_authorities_for_epoch(epoch).0
    }

    fn verify_authority_signature(
        authority_id: &[u8],
        payload_hash: H256,
        signature: &[u8],
    ) -> bool {
        Self::verify_sr25519(authority_id, payload_hash, signature)
    }
}
