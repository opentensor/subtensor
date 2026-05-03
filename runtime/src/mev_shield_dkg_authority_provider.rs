use codec::Encode;
use frame_support::storage::IterableStorageDoubleMap;
use mev_shield_ibe_runtime_api::{DkgAuthorityInfo, DkgConsensusKeyKind, DkgConsensusSource};
use sp_core::{H256, hashing::blake2_256, sr25519};
use sp_runtime::{
    RuntimeAppPublic,
    traits::{UniqueSaturatedInto, Verify},
};
use sp_std::vec::Vec;

use crate::{AccountId32, NetUid, Runtime};

pub struct RuntimeIbeDkgAuthorityProvider;

impl RuntimeIbeDkgAuthorityProvider {
    fn root_validator_hotkeys() -> Vec<(u16, AccountId32, u128)> {
        let root = NetUid::ROOT;
        let permits = pallet_subtensor::Pallet::<Runtime>::get_validator_permit(root);
        let hotkeys: Vec<(u16, AccountId32)> =
            <pallet_subtensor::Keys<Runtime> as IterableStorageDoubleMap<
                NetUid,
                u16,
                AccountId32,
            >>::iter_prefix(root)
            .collect();

        let mut out = Vec::new();
        for (uid, hotkey) in hotkeys {
            if !permits.get(uid as usize).copied().unwrap_or(false) {
                continue;
            }
            let stake: u128 =
                pallet_subtensor::Pallet::<Runtime>::get_total_stake_for_hotkey(&hotkey)
                    .unique_saturated_into();
            if stake > 0 {
                out.push((uid, hotkey, stake));
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn aura_authorities() -> Vec<Vec<u8>> {
        pallet_aura::Authorities::<Runtime>::get()
            .into_inner()
            .into_iter()
            .map(|id| id.to_raw_vec())
            .collect()
    }

    fn authority_rows(kind: DkgConsensusKeyKind) -> Vec<DkgAuthorityInfo> {
        let authority_ids = Self::aura_authorities();
        let root_validators = Self::root_validator_hotkeys();
        let mut out = Vec::new();

        for (idx, authority_id) in authority_ids.into_iter().enumerate() {
            let (hotkey_account_id, stake) = root_validators
                .get(idx)
                .map(|(_, hotkey, stake)| (hotkey.encode(), *stake))
                .unwrap_or_else(|| (authority_id.clone(), 1));

            out.push(DkgAuthorityInfo {
                hotkey_account_id,
                consensus_key_kind: kind,
                authority_id,
                stake: stake.max(1),
                dkg_x25519_public_key: [0u8; 32],
            });
        }

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

    fn authority_registration_payload_hash(
        hotkey_account_id: &[u8],
        consensus_key_kind: DkgConsensusKeyKind,
        consensus_authority_id: &[u8],
        dkg_x25519_public_key: &[u8; 32],
    ) -> H256 {
        H256::from(blake2_256(
            &(
                b"bittensor.mev-shield.v2.dkg.authority-registration",
                hotkey_account_id,
                consensus_key_kind,
                consensus_authority_id,
                dkg_x25519_public_key,
            )
                .encode(),
        ))
    }
}

impl pallet_shield::IbeDkgAuthorityProvider for RuntimeIbeDkgAuthorityProvider {
    fn authorities_for_epoch(_epoch: u64) -> Vec<DkgAuthorityInfo> {
        Self::authority_rows(DkgConsensusKeyKind::AuraSr25519)
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

    fn verify_dkg_authority_registration(
        registration: &mev_shield_ibe_runtime_api::DkgAuthorityRegistration,
    ) -> bool {
        let payload_hash = Self::authority_registration_payload_hash(
            &registration.hotkey_account_id,
            registration.consensus_key_kind,
            &registration.consensus_authority_id,
            &registration.dkg_x25519_public_key,
        );
        match registration.consensus_key_kind {
            DkgConsensusKeyKind::AuraSr25519 | DkgConsensusKeyKind::BabeSr25519 => {
                Self::verify_sr25519(
                    &registration.consensus_authority_id,
                    payload_hash,
                    &registration.consensus_signature,
                )
            }
            DkgConsensusKeyKind::BabeEd25519 => false,
        }
    }
}
