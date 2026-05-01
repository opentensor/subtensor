use std::{collections::BTreeMap, ops::Mul};

use ark_ec::Group;
use ark_ff::{Field, One, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use codec::{Decode, Encode};
use sp_core::H256;
use stp_mev_shield_ibe::{
    BoundedIdentityKey, IbeBlockDecryptionKeyV1, IbeEpochPublicKey, IbePartialDecryptionKeyShareV1,
    KEY_ID_LEN, MEV_SHIELD_IBE_VERSION, block_identity_bytes,
};
use tle::{curves::drand::TinyBLS381, ibe::fullident::Identity};
use w3f_bls::EngineBLS;

pub type Scalar = <TinyBLS381 as EngineBLS>::Scalar;
pub type PublicShare = <TinyBLS381 as EngineBLS>::PublicKeyGroup;
pub type IdentityKeyShare = <TinyBLS381 as EngineBLS>::SignatureGroup;

#[derive(Clone, Debug, Encode, Decode)]
pub struct PublicShareAtom {
    pub share_id: u32,
    pub weight: u128,

    /// Compressed g2^s_i.
    pub public_share: Vec<u8>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct WeightedSecretShareAtom {
    pub public: PublicShareAtom,

    /// Compressed scalar s_i.
    pub secret_scalar: Vec<u8>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct EpochDkgPublicOutput {
    pub epoch_key: IbeEpochPublicKey,

    /// All public weighted Shamir atoms for this epoch/key_id.
    ///
    /// Remote shares are accepted only if their share_id, weight, and public_share
    /// match one of these atoms.
    pub public_atoms: Vec<PublicShareAtom>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct EpochSecretShareBundle {
    pub public: EpochDkgPublicOutput,

    /// Local validator authority id/account bytes for diagnostics.
    pub validator_authority: Vec<u8>,

    /// Only this validator’s secret atoms.
    pub local_atoms: Vec<WeightedSecretShareAtom>,
}

impl EpochDkgPublicOutput {
    pub fn public_atom(&self, share_id: u32) -> Option<&PublicShareAtom> {
        self.public_atoms
            .iter()
            .find(|atom| atom.share_id == share_id)
    }

    pub fn total_public_weight(&self) -> u128 {
        self.public_atoms
            .iter()
            .fold(0u128, |acc, atom| acc.saturating_add(atom.weight))
    }
}

impl EpochSecretShareBundle {
    pub fn epoch_key(&self) -> &IbeEpochPublicKey {
        &self.public.epoch_key
    }

    pub fn epoch(&self) -> u64 {
        self.public.epoch_key.epoch
    }

    pub fn key_id(&self) -> [u8; KEY_ID_LEN] {
        self.public.epoch_key.key_id
    }
}

pub fn identity(
    genesis_hash: H256,
    epoch: u64,
    target_block: u64,
    key_id: [u8; KEY_ID_LEN],
) -> Identity {
    let identity_bytes = block_identity_bytes(genesis_hash, epoch, target_block, key_id);

    Identity::new(stp_mev_shield_ibe::IBE_DOMAIN, vec![identity_bytes])
}

pub fn derive_partial_identity_key(
    genesis_hash: H256,
    bundle: &EpochSecretShareBundle,
    target_block: u64,
    atom: &WeightedSecretShareAtom,
    finalized_ordering_block_number: u64,
    finalized_ordering_block_hash: H256,
) -> Result<IbePartialDecryptionKeyShareV1, String> {
    let epoch_key = bundle.epoch_key();

    let id = identity(
        genesis_hash,
        epoch_key.epoch,
        target_block,
        epoch_key.key_id,
    );

    let scalar = Scalar::deserialize_compressed(&mut &atom.secret_scalar[..])
        .map_err(|e| format!("bad secret scalar: {e:?}"))?;

    let partial = id.public::<TinyBLS381>().mul(scalar);

    let mut partial_identity_key = Vec::new();

    partial
        .serialize_compressed(&mut partial_identity_key)
        .map_err(|e| format!("serialize partial identity key: {e:?}"))?;

    Ok(IbePartialDecryptionKeyShareV1 {
        version: MEV_SHIELD_IBE_VERSION,
        epoch: epoch_key.epoch,
        target_block,
        key_id: epoch_key.key_id,
        finalized_ordering_block_number,
        finalized_ordering_block_hash,
        share_id: atom.public.share_id,
        weight: atom.public.weight,
        public_share: atom.public.public_share.clone(),
        partial_identity_key,
    })
}

pub fn verify_partial_identity_key(
    genesis_hash: H256,
    public_atom: &PublicShareAtom,
    share: &IbePartialDecryptionKeyShareV1,
) -> bool {
    if share.version != MEV_SHIELD_IBE_VERSION {
        return false;
    }

    if share.share_id != public_atom.share_id {
        return false;
    }

    if share.weight != public_atom.weight {
        return false;
    }

    if share.public_share != public_atom.public_share {
        return false;
    }

    let id = identity(genesis_hash, share.epoch, share.target_block, share.key_id);

    let Ok(public_share) = PublicShare::deserialize_compressed(&mut &share.public_share[..]) else {
        return false;
    };

    let Ok(partial_key) =
        IdentityKeyShare::deserialize_compressed(&mut &share.partial_identity_key[..])
    else {
        return false;
    };

    let q_id = id.public::<TinyBLS381>();
    let g2 = <PublicShare as Group>::generator();

    // e(g2^s_i, H(identity)) == e(g2, H(identity)^s_i)
    TinyBLS381::pairing(public_share, q_id) == TinyBLS381::pairing(g2, partial_key)
}

fn lagrange_coeff_at_zero(id: u32, ids: &[u32]) -> Scalar {
    let x_i = Scalar::from(id as u64);

    let mut num = Scalar::one();
    let mut den = Scalar::one();

    for other in ids {
        if *other == id {
            continue;
        }

        let x_j = Scalar::from(*other as u64);
        num *= -x_j;
        den *= x_i - x_j;
    }

    num * den
        .inverse()
        .expect("unique non-zero share ids; denominator nonzero")
}

pub fn combine_identity_key(
    public_output: &EpochDkgPublicOutput,
    target_block: u64,
    shares: &[IbePartialDecryptionKeyShareV1],
) -> Result<IbeBlockDecryptionKeyV1, String> {
    let epoch_key = &public_output.epoch_key;

    let mut by_id = BTreeMap::<u32, &IbePartialDecryptionKeyShareV1>::new();
    let mut total_weight = 0u128;
    let mut finalized_number = 0u64;
    let mut finalized_hash = H256::zero();

    for share in shares {
        if share.version != MEV_SHIELD_IBE_VERSION
            || share.epoch != epoch_key.epoch
            || share.target_block != target_block
            || share.key_id != epoch_key.key_id
        {
            continue;
        }

        let Some(public_atom) = public_output.public_atom(share.share_id) else {
            continue;
        };

        if share.weight != public_atom.weight {
            continue;
        }

        if share.public_share != public_atom.public_share {
            continue;
        }

        if by_id.insert(share.share_id, share).is_none() {
            total_weight = total_weight.saturating_add(share.weight);
        }

        if share.finalized_ordering_block_number >= finalized_number {
            finalized_number = share.finalized_ordering_block_number;
            finalized_hash = share.finalized_ordering_block_hash;
        }

        if total_weight >= epoch_key.threshold_weight {
            break;
        }
    }

    if total_weight < epoch_key.threshold_weight {
        return Err(format!(
            "not enough stake weight for threshold IBE key: have {total_weight}, need {}",
            epoch_key.threshold_weight,
        ));
    }

    let ids: Vec<u32> = by_id.keys().copied().collect();

    let mut acc = IdentityKeyShare::zero();

    for (share_id, share) in by_id {
        let partial =
            IdentityKeyShare::deserialize_compressed(&mut &share.partial_identity_key[..])
                .map_err(|e| format!("bad partial identity key: {e:?}"))?;

        let lambda = lagrange_coeff_at_zero(share_id, &ids);

        acc += partial.mul(lambda);
    }

    let mut identity_decryption_key = Vec::new();

    acc.serialize_compressed(&mut identity_decryption_key)
        .map_err(|e| format!("serialize combined identity key: {e:?}"))?;

    let bounded_key: BoundedIdentityKey = identity_decryption_key
        .try_into()
        .map_err(|_| "identity key has wrong length".to_string())?;

    Ok(IbeBlockDecryptionKeyV1 {
        version: MEV_SHIELD_IBE_VERSION,
        epoch: epoch_key.epoch,
        target_block,
        key_id: epoch_key.key_id,
        identity_decryption_key: bounded_key,
        finalized_ordering_block_number: finalized_number,
        finalized_ordering_block_hash: finalized_hash,
    })
}
