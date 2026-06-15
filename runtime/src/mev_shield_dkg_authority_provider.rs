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

#[derive(Clone, Copy, Eq, PartialEq)]
enum AuthoritySourceMode {
    PoaAura,
    PosBabe,
}

impl RuntimeIbeDkgAuthorityProvider {
    fn current_ibe_epoch() -> u64 {
        pallet_shield::Pallet::<Runtime>::current_ibe_epoch()
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

    fn dedup_sorted_authorities(mut authority_ids: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        authority_ids.sort();
        authority_ids.dedup();
        authority_ids
    }

    fn aura_authority_ids() -> Vec<Vec<u8>> {
        Self::dedup_sorted_authorities(
            pallet_aura::Authorities::<Runtime>::get()
                .into_inner()
                .into_iter()
                .map(|authority| authority.to_raw_vec())
                .collect(),
        )
    }

    fn babe_authority_ids() -> Vec<Vec<u8>> {
        // Do not call the node-facing BABE runtime API from inside the runtime.
        // That API is implemented for the generated RuntimeApiImpl and is meant
        // for external runtime-api calls, not direct runtime self-calls.
        //
        // This runtime currently stages BABE constants for the transition but
        // does not expose a runtime-local BABE authority storage/provider that
        // shield can safely read here. Until that real provider exists, return
        // an empty set so `source_mode_for_epoch` fails back to the PoA/Aura
        // source rather than falsely labelling Aura authorities as BABE/NPoS.
        Vec::new()
    }

    fn source_mode_for_epoch(epoch: u64) -> AuthoritySourceMode {
        let Some(first_pos_epoch) =
            Self::current_ibe_epoch().checked_add(pallet_shield::IBE_DKG_EPOCHS_AHEAD)
        else {
            return AuthoritySourceMode::PoaAura;
        };

        if epoch >= first_pos_epoch && !Self::babe_authority_ids().is_empty() {
            AuthoritySourceMode::PosBabe
        } else {
            AuthoritySourceMode::PoaAura
        }
    }

    fn mapped_stake_for_authority(
        authority_id: &[u8],
        ordinal: usize,
        root_entries: &[RootValidatorEntry],
        allow_dev_equal_stake_fallback: bool,
    ) -> Option<(Vec<u8>, u128)> {
        if let Some(entry) = root_entries
            .iter()
            .find(|entry| entry.hotkey_id.as_slice() == authority_id)
        {
            return Some((entry.hotkey_id.clone(), entry.stake));
        }

        if let Some(entry) = root_entries.get(ordinal) {
            return Some((entry.hotkey_id.clone(), entry.stake));
        }

        if allow_dev_equal_stake_fallback {
            return Some((authority_id.to_vec(), 1));
        }

        None
    }

    fn build_authorities(
        root_entries: &[RootValidatorEntry],
        authority_ids: &[Vec<u8>],
        consensus_key_kind: DkgConsensusKeyKind,
        allow_dev_equal_stake_fallback: bool,
    ) -> Vec<DkgAuthorityInfo> {
        if authority_ids.is_empty() {
            return Vec::new();
        }

        let mut out = Vec::new();
        for (ordinal, authority_id) in authority_ids.iter().enumerate() {
            let Some((hotkey_account_id, stake)) = Self::mapped_stake_for_authority(
                authority_id.as_slice(),
                ordinal,
                root_entries,
                allow_dev_equal_stake_fallback,
            ) else {
                continue;
            };

            if stake == 0 {
                continue;
            }

            out.push(DkgAuthorityInfo {
                hotkey_account_id,
                consensus_key_kind,
                authority_id: authority_id.clone(),
                stake,
                // Transport keys are intentionally not stored in runtime state.
                // Validator nodes overlay signed X25519 DKG transport keys through
                // the DKG P2P protocol before encrypted dealing.
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
            root_entries,
            &Self::aura_authority_ids(),
            DkgConsensusKeyKind::AuraSr25519,
            true,
        )
    }

    fn pos_babe_authorities(root_entries: &[RootValidatorEntry]) -> Vec<DkgAuthorityInfo> {
        // Production PoS snapshots must come from the BABE runtime API. If the
        // runtime has not activated a BABE authority set yet, return an empty set
        // and let snapshot/liveness code fail closed rather than pretending the
        // PoA/Aura set is a BABE/NPoS source.
        Self::build_authorities(
            root_entries,
            &Self::babe_authority_ids(),
            DkgConsensusKeyKind::BabeSr25519,
            false,
        )
    }

    fn selected_authorities_for_epoch(epoch: u64) -> (DkgConsensusSource, Vec<DkgAuthorityInfo>) {
        let root_entries = Self::root_validator_entries();
        match Self::source_mode_for_epoch(epoch) {
            AuthoritySourceMode::PosBabe => {
                let authorities = Self::pos_babe_authorities(&root_entries);
                if !authorities.is_empty() {
                    return (DkgConsensusSource::PosBabeRootValidators, authorities);
                }
            }
            AuthoritySourceMode::PoaAura => {}
        }

        (
            DkgConsensusSource::PoaAuraRootValidators,
            Self::poa_authorities(&root_entries),
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
