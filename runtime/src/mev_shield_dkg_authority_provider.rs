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
    fn future_dkg_epoch() -> Option<u64> {
        pallet_shield::Pallet::<Runtime>::current_ibe_epoch()
            .checked_add(pallet_shield::IBE_DKG_EPOCHS_AHEAD)
    }

    fn root_validator_entries() -> Vec<RootValidatorEntry> {
        let root = NetUid::ROOT;
        let permits = pallet_subtensor::Pallet::<Runtime>::get_validator_permit(root);
        let mut keys: Vec<(u16, AccountId32)> =
            <pallet_subtensor::Keys<Runtime> as frame_support::storage::IterableStorageDoubleMap<
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
        let mut ids = pallet_aura::Authorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|authority| authority.to_raw_vec())
            .collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        ids
    }

    fn babe_authority_ids() -> Vec<Vec<u8>> {
        // This runtime does not yet include pallet-session/pallet-babe authority
        // storage in construct_runtime!, and the generated node-facing BabeApi
        // currently exposes empty epoch authority vectors. Do not relabel Aura
        // authorities as BABE here: DKG output signatures would then be checked
        // against the wrong production authority source. Once runtime-local
        // session/BABE storage is wired in, this function is the single source
        // for the finalized N+2 BABE authority ids.
        Vec::new()
    }

    fn source_for_epoch(epoch: u64) -> DkgConsensusSource {
        if let Some(stored) = pallet_shield::IbeDkgConsensusSources::<Runtime>::get(epoch) {
            return stored;
        }

        let Some(future_epoch) = Self::future_dkg_epoch() else {
            return DkgConsensusSource::PoaAuraRootValidators;
        };

        if epoch >= future_epoch && !Self::babe_authority_ids().is_empty() {
            DkgConsensusSource::PosBabeRootValidators
        } else {
            DkgConsensusSource::PoaAuraRootValidators
        }
    }

    fn stake_for_authority(
        authority_id: &[u8],
        ordinal: usize,
        root_entries: &[RootValidatorEntry],
        allow_ordinal_fallback: bool,
        allow_equal_stake_fallback: bool,
    ) -> Option<(Vec<u8>, u128)> {
        if let Some(root) = root_entries
            .iter()
            .find(|root| root.hotkey_id.as_slice() == authority_id)
        {
            return Some((root.hotkey_id.clone(), root.stake));
        }

        if allow_ordinal_fallback {
            if let Some(root) = root_entries.get(ordinal) {
                return Some((root.hotkey_id.clone(), root.stake));
            }
        }

        if allow_equal_stake_fallback {
            return Some((authority_id.to_vec(), 1));
        }

        None
    }

    fn build_authorities(
        consensus_key_kind: DkgConsensusKeyKind,
        authority_ids: Vec<Vec<u8>>,
        root_entries: &[RootValidatorEntry],
        allow_ordinal_fallback: bool,
        allow_equal_stake_fallback: bool,
    ) -> Vec<DkgAuthorityInfo> {
        let mut out = Vec::new();
        for (ordinal, authority_id) in authority_ids.into_iter().enumerate() {
            let Some((hotkey_account_id, stake)) = Self::stake_for_authority(
                authority_id.as_slice(),
                ordinal,
                root_entries,
                allow_ordinal_fallback,
                allow_equal_stake_fallback,
            ) else {
                return Vec::new();
            };

            if stake == 0 {
                continue;
            }

            out.push(DkgAuthorityInfo {
                hotkey_account_id,
                consensus_key_kind,
                authority_id,
                stake,
                // DKG transport keys are node-local/P2P state, not runtime
                // authority-set state. The worker overlays signed transport-key
                // gossip after the runtime has chosen the stake-weighted cohort.
                dkg_x25519_public_key: [0u8; 32],
            });
        }

        out.sort_by(|a, b| {
            (a.consensus_key_kind, &a.authority_id).cmp(&(b.consensus_key_kind, &b.authority_id))
        });
        out.dedup_by(|a, b| {
            a.consensus_key_kind == b.consensus_key_kind && a.authority_id == b.authority_id
        });
        out
    }

    fn poa_authorities(root_entries: &[RootValidatorEntry]) -> Vec<DkgAuthorityInfo> {
        Self::build_authorities(
            DkgConsensusKeyKind::AuraSr25519,
            Self::aura_authority_ids(),
            root_entries,
            true,
            root_entries.is_empty(),
        )
    }

    fn pos_babe_authorities(root_entries: &[RootValidatorEntry]) -> Vec<DkgAuthorityInfo> {
        Self::build_authorities(
            DkgConsensusKeyKind::BabeSr25519,
            Self::babe_authority_ids(),
            root_entries,
            false,
            false,
        )
    }

    fn selected_authorities_for_epoch(epoch: u64) -> (DkgConsensusSource, Vec<DkgAuthorityInfo>) {
        let root_entries = Self::root_validator_entries();
        match Self::source_for_epoch(epoch) {
            DkgConsensusSource::PoaAuraRootValidators => (
                DkgConsensusSource::PoaAuraRootValidators,
                Self::poa_authorities(&root_entries),
            ),
            DkgConsensusSource::PosBabeRootValidators => (
                DkgConsensusSource::PosBabeRootValidators,
                Self::pos_babe_authorities(&root_entries),
            ),
        }
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
