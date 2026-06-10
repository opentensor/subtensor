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
    uid: u16,
    hotkey: AccountId32,
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

            let hotkey_id: Vec<u8> =
                <AccountId32 as core::convert::AsRef<[u8]>>::as_ref(&hotkey).to_vec();
            out.push(RootValidatorEntry {
                uid,
                hotkey,
                hotkey_id,
                stake,
            });
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

    /// Return the root-validator cohort that corresponds to the current PoA/Aura
    /// authority set.
    ///
    /// During the PoA -> PoS transition the first BABE DKG epoch must use the
    /// exact same validator cohort that was authoring under PoA. New root
    /// validators can exist in Subtensor storage before they are meant to join
    /// MEV Shield DKG, so the handoff cohort is capped to the current Aura
    /// authority count. If explicit Aura DKG registrations exist, use them to
    /// identify the cohort; otherwise preserve the existing UID-ordered PoA
    /// compatibility mapping.
    fn poa_handoff_root_entries(
        root_entries: &[RootValidatorEntry],
        aura_authority_ids: &[Vec<u8>],
    ) -> Vec<RootValidatorEntry> {
        if root_entries.is_empty() || aura_authority_ids.is_empty() {
            return Vec::new();
        }

        let mut aura_ids = aura_authority_ids.to_vec();
        aura_ids.sort();

        let mut registered_aura_roots = Vec::new();
        for root in root_entries {
            let Some(registration) =
                pallet_shield::IbeDkgAuthorityRegistrations::<Runtime>::get(&root.hotkey)
            else {
                continue;
            };
            if registration.consensus_key_kind == DkgConsensusKeyKind::AuraSr25519
                && aura_ids.binary_search(&registration.authority_id).is_ok()
            {
                registered_aura_roots.push(root.clone());
            }
        }

        if registered_aura_roots.len() == aura_authority_ids.len() {
            registered_aura_roots.sort_by_key(|entry| entry.uid);
            return registered_aura_roots;
        }

        if root_entries.len() <= aura_authority_ids.len() {
            return root_entries.to_vec();
        }

        root_entries
            .iter()
            .take(aura_authority_ids.len())
            .cloned()
            .collect()
    }

    fn poa_aura_authorities(root_entries: &[RootValidatorEntry]) -> Vec<DkgAuthorityInfo> {
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

    fn registered_authorities_for_kind(
        root_entries: &[RootValidatorEntry],
        key_kind: DkgConsensusKeyKind,
    ) -> Vec<DkgAuthorityInfo> {
        let mut out = Vec::new();
        for root in root_entries {
            let Some(registration) =
                pallet_shield::IbeDkgAuthorityRegistrations::<Runtime>::get(&root.hotkey)
            else {
                continue;
            };
            if registration.consensus_key_kind != key_kind {
                continue;
            }
            if registration.authority_id.len() != 32
                || registration.dkg_x25519_public_key == [0u8; 32]
            {
                continue;
            }
            out.push(DkgAuthorityInfo {
                hotkey_account_id: root.hotkey_id.clone(),
                consensus_key_kind: registration.consensus_key_kind,
                authority_id: registration.authority_id,
                stake: root.stake,
                dkg_x25519_public_key: registration.dkg_x25519_public_key,
            });
        }
        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        out
    }

    fn pos_babe_snapshot_exists() -> bool {
        pallet_shield::IbeDkgConsensusSources::<Runtime>::iter()
            .any(|(_, source)| matches!(source, DkgConsensusSource::PosBabeRootValidators))
    }

    fn selected_authorities_for_epoch(epoch: u64) -> (DkgConsensusSource, Vec<DkgAuthorityInfo>) {
        let root_entries = Self::root_validator_entries();
        let aura_authority_ids = Self::aura_authority_ids();
        let pos_window =
            epoch >= Self::current_epoch().saturating_add(2) && !root_entries.is_empty();

        if pos_window {
            let handoff_entries =
                Self::poa_handoff_root_entries(&root_entries, &aura_authority_ids);
            let handoff_babe_authorities = Self::registered_authorities_for_kind(
                &handoff_entries,
                DkgConsensusKeyKind::BabeSr25519,
            );
            let handoff_complete =
                !handoff_entries.is_empty() && handoff_entries.len() == aura_authority_ids.len();

            if !Self::pos_babe_snapshot_exists() {
                if handoff_complete
                    && !handoff_babe_authorities.is_empty()
                    && handoff_babe_authorities.len() == handoff_entries.len()
                {
                    // First PoS DKG plan: exact same validator cohort as PoA,
                    // but using their registered BABE + X25519 DKG keys.
                    return (
                        DkgConsensusSource::PosBabeRootValidators,
                        handoff_babe_authorities,
                    );
                }
                if !handoff_babe_authorities.is_empty() {
                    // The handoff has started but the complete PoA cohort has
                    // not registered BABE/X25519 keys yet. Fail closed instead
                    // of freezing a mixed or partial future validator set.
                    return (DkgConsensusSource::PosBabeRootValidators, Vec::new());
                }
            } else {
                if !handoff_complete || handoff_babe_authorities.len() != handoff_entries.len() {
                    // Once the PoS handoff is active, never fall back to PoA for
                    // future DKG. Missing registration for an existing handoff
                    // validator is a liveness problem handled by DKG retry/key
                    // extension, not by silently changing trust assumptions.
                    return (DkgConsensusSource::PosBabeRootValidators, Vec::new());
                }

                let babe_authorities = Self::registered_authorities_for_kind(
                    &root_entries,
                    DkgConsensusKeyKind::BabeSr25519,
                );
                if !babe_authorities.is_empty() {
                    // After the first PoS snapshot, new root validators join MEV
                    // Shield DKG automatically once they have stake, permit, and
                    // registered BABE/X25519 DKG material. Existing frozen
                    // N..N+2 snapshots provide the slow-addition delay.
                    return (DkgConsensusSource::PosBabeRootValidators, babe_authorities);
                }
            }
        }

        (
            DkgConsensusSource::PoaAuraRootValidators,
            Self::poa_aura_authorities(&root_entries),
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
