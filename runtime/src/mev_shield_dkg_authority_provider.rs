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
            if !permits.get(usize::from(uid)).copied().unwrap_or(false) {
                continue;
            }

            let stake: u128 =
                pallet_subtensor::Pallet::<Runtime>::get_total_stake_for_hotkey(&hotkey)
                    .unique_saturated_into();
            if stake == 0 {
                continue;
            }

            let hotkey_id = <AccountId32 as core::convert::AsRef<[u8]>>::as_ref(&hotkey).to_vec();
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

    fn poa_aura_authorities() -> Vec<DkgAuthorityInfo> {
        let root_entries = Self::root_validator_entries();
        let authority_ids = Self::aura_authority_ids();
        let mut out = Vec::new();

        for (ordinal, authority_id) in authority_ids.into_iter().enumerate() {
            let (hotkey_account_id, stake) = root_entries
                .iter()
                .find(|root| root.hotkey_id.as_slice() == authority_id.as_slice())
                .or_else(|| root_entries.get(ordinal))
                .map(|root| (root.hotkey_id.clone(), root.stake))
                .unwrap_or_else(|| (authority_id.clone(), 1));

            if stake == 0 {
                continue;
            }

            out.push(DkgAuthorityInfo {
                hotkey_account_id,
                consensus_key_kind: DkgConsensusKeyKind::AuraSr25519,
                authority_id,
                stake,
                // X25519 DKG transport keys are node/P2P material. Runtime authority
                // snapshots define stake and consensus signing keys; workers overlay
                // transport keys from signed DKG transport-key gossip.
                dkg_x25519_public_key: [0u8; 32],
            });
        }

        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        out
    }

    fn npos_babe_authorities(epoch: u64) -> Vec<DkgAuthorityInfo> {
        let mut out = pallet_shield::IbeNposDkgAuthoritySnapshots::<Runtime>::get(epoch);
        out.retain(|authority| {
            authority.consensus_key_kind == DkgConsensusKeyKind::BabeSr25519
                && !authority.authority_id.is_empty()
                && authority.stake > 0
        });
        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        out
    }

    fn source_for_epoch(epoch: u64) -> DkgConsensusSource {
        pallet_shield::IbeDkgConsensusSources::<Runtime>::get(epoch)
            .unwrap_or(DkgConsensusSource::PoaAuraRootValidators)
    }

    fn verify_sr25519(public: &[u8], payload_hash: H256, signature: &[u8]) -> bool {
        let Ok(public_bytes) = <[u8; 32]>::try_from(public) else {
            return false;
        };
        let Ok(signature_bytes) = <[u8; 64]>::try_from(signature) else {
            return false;
        };

        let public = sr25519::Public::from_raw(public_bytes);
        let signature = sr25519::Signature::from_raw(signature_bytes);
        signature.verify(&payload_hash.as_fixed_bytes()[..], &public)
    }
}

impl pallet_shield::IbeDkgAuthorityProvider for RuntimeIbeDkgAuthorityProvider {
    fn authorities_for_epoch(epoch: u64) -> Vec<DkgAuthorityInfo> {
        match Self::source_for_epoch(epoch) {
            DkgConsensusSource::PosBabeRootValidators => Self::npos_babe_authorities(epoch),
            DkgConsensusSource::PoaAuraRootValidators => Self::poa_aura_authorities(),
        }
    }

    fn consensus_source_for_epoch(epoch: u64) -> DkgConsensusSource {
        match Self::source_for_epoch(epoch) {
            DkgConsensusSource::PosBabeRootValidators => {
                if Self::npos_babe_authorities(epoch).is_empty() {
                    // Fail closed rather than silently relabeling the PoA/Aura set as
                    // the production BABE/NPoS source.
                    DkgConsensusSource::PosBabeRootValidators
                } else {
                    DkgConsensusSource::PosBabeRootValidators
                }
            }
            DkgConsensusSource::PoaAuraRootValidators => DkgConsensusSource::PoaAuraRootValidators,
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
