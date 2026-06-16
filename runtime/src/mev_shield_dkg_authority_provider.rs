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
struct RootValidatorStake {
    hotkey_id: Vec<u8>,
    stake: u128,
}

impl RuntimeIbeDkgAuthorityProvider {
    fn epoch_source(epoch: u64) -> DkgConsensusSource {
        pallet_shield::IbeDkgConsensusSources::<Runtime>::get(epoch)
            .unwrap_or(DkgConsensusSource::PoaAuraRootValidators)
    }

    fn root_stake_snapshot() -> Vec<RootValidatorStake> {
        let root = NetUid::ROOT;
        let permits = pallet_subtensor::Pallet::<Runtime>::get_validator_permit(root);
        let mut out = Vec::new();
        let keys = pallet_subtensor::Keys::<Runtime>::iter_prefix(root);
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
            out.push(RootValidatorStake { hotkey_id, stake });
        }
        out.sort_by(|a, b| a.hotkey_id.cmp(&b.hotkey_id));
        out
    }

    fn stake_for_authority(
        authority_id: &[u8],
        root_stake: &[RootValidatorStake],
    ) -> Option<(Vec<u8>, u128)> {
        root_stake
            .iter()
            .find(|entry| entry.hotkey_id.as_slice() == authority_id)
            .map(|entry| (entry.hotkey_id.clone(), entry.stake))
    }

    fn aura_authorities() -> Vec<Vec<u8>> {
        let mut authorities = pallet_aura::Authorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|authority| authority.to_raw_vec())
            .collect::<Vec<_>>();
        authorities.sort();
        authorities
    }

    fn babe_current_authorities() -> Vec<(Vec<u8>, u64)> {
        let mut authorities = pallet_babe::Authorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|(authority, weight)| (authority.to_raw_vec(), weight))
            .collect::<Vec<_>>();
        authorities.sort_by(|a, b| a.0.cmp(&b.0));
        authorities
    }

    fn babe_next_authorities() -> Vec<(Vec<u8>, u64)> {
        let mut authorities = pallet_babe::NextAuthorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|(authority, weight)| (authority.to_raw_vec(), weight))
            .collect::<Vec<_>>();
        authorities.sort_by(|a, b| a.0.cmp(&b.0));
        authorities
    }

    pub fn babe_api_authorities() -> Vec<(sp_consensus_babe::AuthorityId, u64)> {
        let next = pallet_babe::NextAuthorities::<Runtime>::get().into_inner();
        if !next.is_empty() {
            return next;
        }
        pallet_babe::Authorities::<Runtime>::get().into_inner()
    }

    fn from_authority_ids(
        key_kind: DkgConsensusKeyKind,
        authority_ids: Vec<(Vec<u8>, u64)>,
        allow_poa_equal_stake_fallback: bool,
    ) -> Vec<DkgAuthorityInfo> {
        let root_stake = Self::root_stake_snapshot();
        let mut out = Vec::new();

        for (authority_id, consensus_weight) in authority_ids {
            let Some((hotkey_account_id, stake)) =
                Self::stake_for_authority(&authority_id, &root_stake).or_else(|| {
                    if allow_poa_equal_stake_fallback {
                        Some((authority_id.clone(), u128::from(consensus_weight.max(1))))
                    } else {
                        None
                    }
                })
            else {
                continue;
            };
            if stake == 0 {
                continue;
            }
            out.push(DkgAuthorityInfo {
                hotkey_account_id,
                consensus_key_kind: key_kind,
                authority_id,
                stake,
                dkg_x25519_public_key: [0u8; 32],
            });
        }

        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        out.dedup_by(|a, b| {
            a.authority_id == b.authority_id && a.consensus_key_kind == b.consensus_key_kind
        });
        out
    }

    fn poa_aura_authorities() -> Vec<DkgAuthorityInfo> {
        let aura = Self::aura_authorities()
            .into_iter()
            .map(|authority| (authority, 1u64))
            .collect::<Vec<_>>();
        Self::from_authority_ids(DkgConsensusKeyKind::AuraSr25519, aura, true)
    }

    fn pos_babe_authorities(epoch: u64) -> Vec<DkgAuthorityInfo> {
        let current = pallet_shield::Pallet::<Runtime>::current_ibe_epoch();
        let authorities = if epoch > current {
            Self::babe_next_authorities()
        } else {
            Self::babe_current_authorities()
        };
        Self::from_authority_ids(DkgConsensusKeyKind::BabeSr25519, authorities, false)
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
        match Self::epoch_source(epoch) {
            DkgConsensusSource::PoaAuraRootValidators => Self::poa_aura_authorities(),
            DkgConsensusSource::PosBabeRootValidators => Self::pos_babe_authorities(epoch),
        }
    }

    fn consensus_source_for_epoch(epoch: u64) -> DkgConsensusSource {
        Self::epoch_source(epoch)
    }

    fn verify_authority_signature(
        authority_id: &[u8],
        payload_hash: H256,
        signature: &[u8],
    ) -> bool {
        Self::verify_sr25519(authority_id, payload_hash, signature)
    }
}
