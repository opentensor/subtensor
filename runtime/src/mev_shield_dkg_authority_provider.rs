//! Concrete Subtensor runtime authority provider for threshold-IBE epoch DKG.
//!
//! This provider is POS-transition aware.  DKG eligibility is derived from the
//! stake-bearing Subtensor root validators, not directly from Aura.  Each hotkey
//! registers the consensus key it controls and a durable DKG X25519 transport
//! key.  The provider prefers BABE registrations when they cover a 2/3+1 stake
//! threshold; otherwise it uses Aura registrations for the current POA phase.
//! This makes the DKG authority source switch automatically once the POS/BABE
//! transition has enough registered BABE keys.

use codec::{Decode, Encode};
use frame_support::storage::IterableStorageDoubleMap;
use mev_shield_ibe_runtime_api::{DkgAuthorityInfo, DkgConsensusKeyKind, DkgConsensusSource};
use sp_core::{H256, hashing::blake2_256, sr25519};
use sp_runtime::traits::{UniqueSaturatedInto, Verify};

use crate::{AccountId32, NetUid, Runtime};
use sp_std::vec::Vec;

pub struct RuntimeIbeDkgAuthorityProvider;

impl RuntimeIbeDkgAuthorityProvider {
    fn root_validator_hotkeys() -> Vec<(u16, AccountId32, u128)> {
        let root = NetUid::ROOT;
        let permits = pallet_subtensor::Pallet::<Runtime>::get_validator_permit(root);
        let mut out = Vec::new();

        let hotkeys: Vec<(u16, AccountId32)> =
            <pallet_subtensor::Keys<Runtime> as IterableStorageDoubleMap<
                NetUid,
                u16,
                AccountId32,
            >>::iter_prefix(root)
            .collect();

        for (uid, hotkey) in hotkeys {
            let permitted = permits.get(uid as usize).copied().unwrap_or(false);
            if !permitted {
                continue;
            }
            let stake: u128 =
                pallet_subtensor::Pallet::<Runtime>::get_total_stake_for_hotkey(&hotkey)
                    .unique_saturated_into();
            if stake == 0 {
                continue;
            }
            out.push((uid, hotkey, stake));
        }

        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn two_thirds_plus_one(total: u128) -> u128 {
        total.saturating_mul(2) / 3 + 1
    }

    fn authorities_for_kind(kind: DkgConsensusKeyKind) -> Vec<DkgAuthorityInfo> {
        let mut out = Vec::new();
        for (_uid, hotkey, stake) in Self::root_validator_hotkeys() {
            let hotkey_bytes = hotkey.encode();
            let Some(reg) = pallet_shield::IbeDkgAuthorityRegistrations::<Runtime>::get(
                hotkey_bytes.clone(),
                kind,
            ) else {
                continue;
            };
            out.push(DkgAuthorityInfo {
                hotkey_account_id: hotkey_bytes,
                consensus_key_kind: kind,
                authority_id: reg.consensus_authority_id,
                stake,
                dkg_x25519_public_key: reg.dkg_x25519_public_key,
            });
        }
        out.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));
        out
    }

    fn selected_authorities() -> (DkgConsensusSource, Vec<DkgAuthorityInfo>) {
        let validator_total_stake = Self::root_validator_hotkeys()
            .into_iter()
            .fold(0u128, |acc, (_, _, stake)| acc.saturating_add(stake));
        let threshold = Self::two_thirds_plus_one(validator_total_stake);

        let babe = Self::authorities_for_kind(DkgConsensusKeyKind::BabeSr25519);
        let babe_stake = babe
            .iter()
            .fold(0u128, |acc, a| acc.saturating_add(a.stake));
        if !babe.is_empty() && babe_stake >= threshold {
            return (DkgConsensusSource::PosBabeRootValidators, babe);
        }

        let aura = Self::authorities_for_kind(DkgConsensusKeyKind::AuraSr25519);
        (DkgConsensusSource::PoaAuraRootValidators, aura)
    }

    pub fn selected_consensus_source() -> DkgConsensusSource {
        Self::selected_authorities().0
    }

    fn registration_payload_hash(
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

    fn verify_hotkey_signature(
        hotkey_account_id: &[u8],
        payload_hash: H256,
        signature: &[u8],
    ) -> bool {
        let Ok(account) = AccountId32::decode(&mut &hotkey_account_id[..]) else {
            return false;
        };
        Self::verify_sr25519(account.as_ref(), payload_hash, signature)
    }

    pub fn verify_authority_registration(
        registration: &mev_shield_ibe_runtime_api::DkgAuthorityRegistration,
    ) -> bool {
        let payload = Self::registration_payload_hash(
            &registration.hotkey_account_id,
            registration.consensus_key_kind,
            &registration.consensus_authority_id,
            &registration.dkg_x25519_public_key,
        );

        if !Self::verify_hotkey_signature(
            &registration.hotkey_account_id,
            payload,
            &registration.hotkey_signature,
        ) {
            return false;
        }

        match registration.consensus_key_kind {
            DkgConsensusKeyKind::AuraSr25519 | DkgConsensusKeyKind::BabeSr25519 => {
                Self::verify_sr25519(
                    &registration.consensus_authority_id,
                    payload,
                    &registration.consensus_signature,
                )
            }
            DkgConsensusKeyKind::BabeEd25519 => false,
        }
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

    fn verify_dkg_authority_registration(
        registration: &mev_shield_ibe_runtime_api::DkgAuthorityRegistration,
    ) -> bool {
        Self::verify_authority_registration(registration)
    }
}
