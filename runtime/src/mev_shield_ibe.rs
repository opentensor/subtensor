use ark_ec::Group;
use ark_serialize::CanonicalDeserialize;
use codec::Decode;
use frame_support::dispatch::{DispatchInfo, GetDispatchInfo};
use pallet_shield::{
    DecryptedExtrinsicExecutor, IbeAppliedExtrinsic, IbeDecryptOutcome,
    IbeEncryptedTxDecryptor, IbeKeyVerifier,
};
use sp_core::H256;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchError};
use sp_std::vec;
use stp_mev_shield_ibe::{
    block_identity_bytes, IbeEncryptedExtrinsicV1, IbeEpochPublicKey,
    MEV_SHIELD_IBE_VERSION,
};
use tle::{
    curves::drand::TinyBLS381,
    ibe::fullident::Identity,
    stream_ciphers::AESGCMStreamCipherProvider,
    tlock::{tld, TLECiphertext},
};
use w3f_bls::EngineBLS;

use crate::{Executive, Runtime, RuntimeCall, UncheckedExtrinsic};

pub struct MevShieldIbeVerifier;

impl<HashT> IbeKeyVerifier<HashT> for MevShieldIbeVerifier
where
    HashT: Into<H256>,
{
    fn verify_block_identity_key(
        genesis_hash: HashT,
        epoch_key: &IbeEpochPublicKey,
        target_block: u64,
        identity_decryption_key: &[u8],
    ) -> bool {
        let genesis_hash: H256 = genesis_hash.into();

        let identity_bytes = block_identity_bytes(
            genesis_hash,
            epoch_key.epoch,
            target_block,
            epoch_key.key_id,
        );

        let identity = Identity::new(stp_mev_shield_ibe::IBE_DOMAIN, vec![identity_bytes]);

        let Ok(master_public_key) =
            <TinyBLS381 as EngineBLS>::PublicKeyGroup::deserialize_compressed(
                &mut &epoch_key.master_public_key[..],
            )
        else {
            return false;
        };

        let Ok(identity_key) =
            <TinyBLS381 as EngineBLS>::SignatureGroup::deserialize_compressed(
                &mut &identity_decryption_key[..],
            )
        else {
            return false;
        };

        let q_id = identity.public::<TinyBLS381>();
        let g2 = <<TinyBLS381 as EngineBLS>::PublicKeyGroup as Group>::generator();

        // d_id = H(identity)^msk
        //
        // Verify:
        //   e(g2^msk, H(identity)) == e(g2, d_id)
        TinyBLS381::pairing(master_public_key, q_id)
            == TinyBLS381::pairing(g2, identity_key)
    }
}

pub struct MevShieldIbeDecryptor;

impl IbeEncryptedTxDecryptor<UncheckedExtrinsic> for MevShieldIbeDecryptor {
    fn decrypt(data: &[u8]) -> IbeDecryptOutcome<UncheckedExtrinsic> {
        let Ok(envelope) = IbeEncryptedExtrinsicV1::decode_v2(data) else {
            return IbeDecryptOutcome::InvalidAfterKeyAvailable;
        };

        if envelope.version != MEV_SHIELD_IBE_VERSION {
            return IbeDecryptOutcome::InvalidAfterKeyAvailable;
        }

        let current_block: u64 =
            frame_system::Pallet::<Runtime>::block_number().unique_saturated_into();

        if current_block < envelope.target_block {
            return IbeDecryptOutcome::NotReady;
        }

        let Some(block_key) = pallet_shield::Pallet::<Runtime>::ibe_block_decryption_key(
            envelope.epoch,
            envelope.target_block,
            envelope.key_id,
        ) else {
            return IbeDecryptOutcome::NotReady;
        };

        let Ok(ciphertext) =
            TLECiphertext::<TinyBLS381>::deserialize_compressed(
                &mut &envelope.ciphertext[..],
            )
        else {
            return IbeDecryptOutcome::InvalidAfterKeyAvailable;
        };

        let Ok(identity_key) =
            <TinyBLS381 as EngineBLS>::SignatureGroup::deserialize_compressed(
                &mut &block_key.identity_decryption_key[..],
            )
        else {
            return IbeDecryptOutcome::InvalidAfterKeyAvailable;
        };

        let Ok(plaintext) =
            tld::<TinyBLS381, AESGCMStreamCipherProvider>(ciphertext, identity_key)
        else {
            return IbeDecryptOutcome::InvalidAfterKeyAvailable;
        };

        if stp_mev_shield_ibe::plaintext_commitment(&plaintext) != envelope.commitment {
            return IbeDecryptOutcome::InvalidAfterKeyAvailable;
        }

        let Ok(inner_xt) = UncheckedExtrinsic::decode(&mut &plaintext[..]) else {
            return IbeDecryptOutcome::InvalidAfterKeyAvailable;
        };

        IbeDecryptOutcome::Ready(inner_xt)
    }
}

pub struct MevShieldInnerExtrinsicExecutor;
impl DecryptedExtrinsicExecutor<UncheckedExtrinsic> for MevShieldInnerExtrinsicExecutor {
    fn dispatch_info(inner: &UncheckedExtrinsic) -> Option<DispatchInfo> {
        Some(inner.0.function.get_dispatch_info())
    }

    fn apply(inner: UncheckedExtrinsic) -> IbeAppliedExtrinsic {
        let fallback_weight = Self::dispatch_info(&inner)
            .map(|info| info.call_weight)
            .unwrap_or_default();

        match Executive::apply_extrinsic(inner) {
            Ok(Ok(())) => IbeAppliedExtrinsic {
                consumed_weight: fallback_weight,
                success: true,
            },

            Ok(Err(_dispatch_error)) => IbeAppliedExtrinsic {
                consumed_weight: fallback_weight,
                success: false,
            },

            Err(_validity_error) => IbeAppliedExtrinsic {
                consumed_weight: fallback_weight,
                success: false,
            },
        }
    }
}

/// Legacy queue decryptor kept for non-v2 queued payloads. v1 live shielding
/// still uses the existing author-side unshielding path and is not routed here.
pub struct LegacyDeferredRuntimeCallDecryptor;

impl pallet_shield::ExtrinsicDecryptor<RuntimeCall> for LegacyDeferredRuntimeCallDecryptor {
    fn decrypt(_data: &[u8]) -> Result<RuntimeCall, DispatchError> {
        Err(DispatchError::Other(
            "Legacy deferred runtime call decryptor disabled",
        ))
    }
}