use frame_support::storage::IterableStorageDoubleMap;
use mev_shield_ibe_runtime_api::{DkgAuthorityInfo, DkgConsensusKeyKind, DkgConsensusSource};
use sp_core::{H256, sr25519};
use sp_runtime::{
    RuntimeAppPublic,
    traits::{UniqueSaturatedInto, Verify},
};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

use crate::{AccountId32, NetUid, Runtime};

pub struct RuntimeIbeDkgAuthorityProvider;

impl RuntimeIbeDkgAuthorityProvider {
    fn root_validator_stakes_by_hotkey() -> BTreeMap<Vec<u8>, (Vec<u8>, u128)> {
        let root = NetUid::ROOT;
        let permits = pallet_subtensor::Pallet::<Runtime>::get_validator_permit(root);
        let mut out = BTreeMap::new();

        let hotkeys: Vec<(u16, AccountId32)> =
            <pallet_subtensor::Keys<Runtime> as IterableStorageDoubleMap<
                NetUid,
                u16,
                AccountId32,
            >>::iter_prefix(root)
            .collect();

        for (uid, hotkey) in hotkeys {
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
            let hotkey_id = hotkey_bytes.to_vec();
            out.insert(hotkey_id.clone(), (hotkey_id, stake));
        }

        out
    }

    fn observed_consensus_source() -> DkgConsensusSource {
        let digest = frame_system::Pallet::<Runtime>::digest();
        let has_babe_predigest = digest
            .logs()
            .iter()
            .filter_map(|log| log.as_pre_runtime())
            .any(|(engine_id, _)| engine_id == sp_consensus_babe::BABE_ENGINE_ID);

        if has_babe_predigest {
            DkgConsensusSource::PosBabeRootValidators
        } else {
            DkgConsensusSource::PoaAuraRootValidators
        }
    }

    fn consensus_key_kind(source: DkgConsensusSource) -> DkgConsensusKeyKind {
        match source {
            DkgConsensusSource::PosBabeRootValidators => DkgConsensusKeyKind::BabeSr25519,
            DkgConsensusSource::PoaAuraRootValidators => DkgConsensusKeyKind::AuraSr25519,
        }
    }

    fn authority_ids_from_runtime() -> Vec<Vec<u8>> {
        pallet_aura::Authorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|authority| authority.to_raw_vec())
            .collect()
    }

    fn selected_authorities() -> (DkgConsensusSource, Vec<DkgAuthorityInfo>) {
        let source = Self::observed_consensus_source();
        let key_kind = Self::consensus_key_kind(source);
        let authority_ids = Self::authority_ids_from_runtime();
        let stakes_by_hotkey = Self::root_validator_stakes_by_hotkey();
        let has_direct_stake_matches = authority_ids
            .iter()
            .any(|authority_id| stakes_by_hotkey.contains_key(authority_id));

        let mut out = Vec::new();
        for authority_id in authority_ids {
            let (hotkey_account_id, stake) = match stakes_by_hotkey.get(&authority_id) {
                Some((hotkey_account_id, stake)) => (hotkey_account_id.clone(), *stake),
                None if !has_direct_stake_matches => (authority_id.clone(), 1),
                None => continue,
            };

            out.push(DkgAuthorityInfo {
                hotkey_account_id,
                consensus_key_kind: key_kind,
                authority_id,
                stake,
                dkg_x25519_public_key: [0u8; 32],
            });
        }

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
    fn authorities_for_epoch(_epoch: u64) -> Vec<DkgAuthorityInfo> {
        Self::selected_authorities().1
    }

    fn consensus_source_for_epoch(_epoch: u64) -> DkgConsensusSource {
        Self::selected_authorities().0
    }

    fn verify_authority_signature(
        authority_id: &[u8],
        payload_hash: H256,
        signature: &[u8],
    ) -> bool {
        Self::verify_sr25519(authority_id, payload_hash, signature)
    }
}
