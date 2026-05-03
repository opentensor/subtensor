#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use stp_mev_shield_ibe::{IbePendingIdentity, KEY_ID_LEN};

/// Runtime-side classification used by node block import.  The node must not
/// infer this from call indexes because runtime upgrades can move pallets/calls.
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub enum MevShieldExtrinsicClass {
    SubmitEncryptedV2 {
        epoch: u64,
        target_block: u64,
        key_id: [u8; KEY_ID_LEN],
        queue_commitment: H256,
    },
    SubmitBlockDecryptionKey {
        epoch: u64,
        target_block: u64,
        key_id: [u8; KEY_ID_LEN],
        finalized_ordering_block_number: u64,
        finalized_ordering_block_hash: H256,
    },
    Operational,
    UnencryptedNonOperational,
}

/// Runtime-computed composition of a candidate block, used by import code to
/// enforce the shielded-queue censorship and preemption invariants.
#[subtensor_macros::freeze_struct("c3f55507780a8117")]
#[derive(
    Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, Default,
)]
pub struct MevShieldBlockComposition {
    pub pending_queue_len_at_parent: u32,
    pub contains_encrypted_v2: bool,
    pub contains_plaintext_non_operational: bool,
    pub contains_only_operational_or_encrypted: bool,
    pub block_weight_ref_time: u64,
    pub max_normal_block_weight_ref_time: u64,
    pub block_len: u32,
    pub max_normal_block_len: u32,
}

impl MevShieldBlockComposition {
    pub fn is_full(&self) -> bool {
        self.block_weight_ref_time >= self.max_normal_block_weight_ref_time
            || self.block_len >= self.max_normal_block_len
    }
}

/// Which consensus key signs DKG traffic for an authority.
///
/// The DKG is deliberately keyed by stake-bearing Subtensor hotkeys, not by an
/// Aura-only authority list.  During POA the runtime will usually choose Aura
/// registrations; after the POS/BABE transition it automatically chooses BABE
/// registrations once they cover the active validator stake threshold.
#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    DecodeWithMemTracking,
    RuntimeDebug,
    TypeInfo,
)]
pub enum DkgConsensusKeyKind {
    AuraSr25519,
    BabeSr25519,
    BabeEd25519,
}

/// How the runtime derived the DKG authority set for this epoch.
#[derive(
    Clone, Copy, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo,
)]
pub enum DkgConsensusSource {
    /// Stake-bearing root validators using BABE registrations.  This is the
    /// production POS source and is preferred whenever enough BABE registrations
    /// exist for the active root validator set.
    PosBabeRootValidators,
    /// Stake-bearing root validators using Aura registrations.  This is the
    /// compatibility source used before the POS/BABE transition.
    PoaAuraRootValidators,
}

/// Full on-chain registration binding one Subtensor hotkey/account to its
/// current consensus signing key and its durable DKG X25519 transport key.
///
/// Registration is accepted only if both signatures verify:
/// * the hotkey/account signature proves the stake-bearing hotkey opted into DKG;
/// * the consensus key signature proves the node that can author blocks under
///   that key controls the DKG transport key.
#[subtensor_macros::freeze_struct("72edeb4c9eb13a62")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct DkgAuthorityRegistration {
    pub hotkey_account_id: Vec<u8>,
    pub consensus_key_kind: DkgConsensusKeyKind,
    pub consensus_authority_id: Vec<u8>,
    pub dkg_x25519_public_key: [u8; 32],
    pub hotkey_signature: Vec<u8>,
    pub consensus_signature: Vec<u8>,
}

/// One consensus authority eligible for an epoch-ahead DKG round.
#[subtensor_macros::freeze_struct("ac25806ef271a94d")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct DkgAuthorityInfo {
    /// Subtensor hotkey/account bytes whose stake is being used for threshold
    /// weighting and whose registration binds the consensus key.
    pub hotkey_account_id: Vec<u8>,
    /// Active consensus key kind for this epoch.
    pub consensus_key_kind: DkgConsensusKeyKind,
    /// Raw consensus authority public key bytes.  This is Aura sr25519 before
    /// the transition and BABE public key bytes once POS/BABE registrations are
    /// live.
    pub authority_id: Vec<u8>,
    /// Stake weight at the runtime snapshot used by the epoch plan.
    pub stake: u128,
    /// Durable node DKG transport key registered on-chain by this authority.
    pub dkg_x25519_public_key: [u8; 32],
}

#[subtensor_macros::freeze_struct("fbf7e7b834e77829")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct EpochDkgPlan {
    /// The epoch being prepared.  When queried during epoch N, the node DKG
    /// worker expects this to be N + 2.
    pub epoch: u64,
    pub first_block: u64,
    pub last_block: u64,
    pub consensus_source: DkgConsensusSource,
    pub authorities: Vec<DkgAuthorityInfo>,
    pub max_atoms: u32,
}

#[subtensor_macros::freeze_struct("9ae06c4561b734b4")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct DkgOutputAttestation {
    pub authority_id: Vec<u8>,
    pub stake: u128,
    pub public_output_hash: H256,
    pub signature: Vec<u8>,
}

/// Public output submitted to chain after a DKG round completes.
#[subtensor_macros::freeze_struct("f612a13738fb1f61")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct EpochDkgPublication {
    pub epoch: u64,
    pub key_id: [u8; KEY_ID_LEN],
    pub first_block: u64,
    pub last_block: u64,
    pub consensus_source: DkgConsensusSource,
    pub master_public_key: Vec<u8>,
    pub total_weight: u128,
    pub threshold_weight: u128,
    pub public_output_hash: H256,
    pub attestations: Vec<DkgOutputAttestation>,
}

/// Backward-compatible transport-only registration retained for existing nodes.
/// New code should use `DkgAuthorityRegistration`; the pallet upgrades this form
/// into an Aura registration whose hotkey/account is equal to the authority id.
#[subtensor_macros::freeze_struct("7558ce2f74ceae4c")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct DkgTransportKeyRegistration {
    pub authority_id: Vec<u8>,
    pub dkg_x25519_public_key: [u8; 32],
    pub signature: Vec<u8>,
}

sp_api::decl_runtime_apis! {
    pub trait MevShieldIbeApi {
        fn pending_ibe_identities(limit: u32) -> Vec<IbePendingIdentity>;
        fn has_ibe_block_key(epoch: u64, target_block: u64, key_id: [u8; KEY_ID_LEN]) -> bool;
        fn pending_encrypted_queue_len() -> u32;
        fn classify_extrinsic(encoded_xt: Vec<u8>) -> MevShieldExtrinsicClass;
        fn block_composition(encoded_xts: Vec<Vec<u8>>) -> MevShieldBlockComposition;
    }

    pub trait MevShieldDkgApi {
        fn active_epoch_dkg_plan() -> Option<EpochDkgPlan>;
        fn next_epoch_dkg_plan() -> Option<EpochDkgPlan>;
        fn verify_epoch_dkg_publication(publication: EpochDkgPublication) -> bool;
        fn verify_dkg_transport_key_registration(registration: DkgTransportKeyRegistration) -> bool;
        fn verify_dkg_authority_registration(registration: DkgAuthorityRegistration) -> bool;
    }
}
