use std::{fs, path::PathBuf};

use ark_ec::Group;
use ark_ff::{One, UniformRand, Zero};
use ark_serialize::CanonicalSerialize;
use ark_std::rand::thread_rng;
use clap::Parser;
use codec::Encode;
use sp_core::H256;
use stp_mev_shield_ibe::{BoundedMasterPublicKey, IbeEpochPublicKey, KEY_ID_LEN};
use tle::curves::drand::TinyBLS381;
use w3f_bls::EngineBLS;

use node_subtensor::mev_shield_ibe::crypto::{
    EpochDkgPublicOutput, EpochSecretShareBundle, PublicShareAtom, WeightedSecretShareAtom,
};

type Scalar = <TinyBLS381 as EngineBLS>::Scalar;
type PublicKeyGroup = <TinyBLS381 as EngineBLS>::PublicKeyGroup;

#[derive(Parser)]
struct Args {
    /// Output root. The generator creates:
    ///
    ///   validator-N/mev_shield_ibe/dkg/public/*.scale
    ///   validator-N/mev_shield_ibe/dkg/secret/*.scale
    #[arg(long)]
    out: PathBuf,

    #[arg(long)]
    epoch: u64,

    #[arg(long)]
    validators: usize,

    /// Number of weighted Shamir atoms assigned to each validator in dev.
    ///
    /// Production must allocate atoms proportional to stake.
    #[arg(long, default_value_t = 1)]
    atoms_per_validator: usize,

    #[arg(long)]
    genesis_hash: H256,

    /// 16-byte key id as hex, with or without 0x.
    #[arg(long)]
    key_id_hex: String,

    #[arg(long, default_value_t = 0)]
    first_block: u64,

    #[arg(long, default_value_t = u64::MAX)]
    last_block: u64,
}

fn parse_key_id(hex: &str) -> [u8; KEY_ID_LEN] {
    let bytes = hex::decode(hex.trim_start_matches("0x")).expect("valid hex key id");

    bytes
        .as_slice()
        .try_into()
        .expect("key id must be exactly 16 bytes")
}

fn eval_poly(coeffs: &[Scalar], x: Scalar) -> Scalar {
    let mut y = Scalar::zero();
    let mut power = Scalar::one();

    for coeff in coeffs {
        y += *coeff * power;
        power *= x;
    }

    y
}

fn write_scale(path: PathBuf, value: impl Encode) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }

    fs::write(path, value.encode()).expect("write SCALE file");
}

fn main() {
    let args = Args::parse();

    assert!(args.validators > 0, "validators must be > 0");
    assert!(
        args.atoms_per_validator > 0,
        "atoms_per_validator must be > 0"
    );

    let key_id = parse_key_id(&args.key_id_hex);

    let total_atoms = args.validators * args.atoms_per_validator;

    // Dev weighting: one atom = one unit of weight.
    //
    // Production must replace this with stake-proportional quantization.
    let total_weight = total_atoms as u128;
    let threshold_weight = ((total_weight * 2) / 3) + 1;

    let degree = threshold_weight as usize - 1;

    let mut rng = thread_rng();

    let coeffs: Vec<Scalar> = (0..=degree).map(|_| Scalar::rand(&mut rng)).collect();

    let master_secret = coeffs[0];
    let master_public = PublicKeyGroup::generator() * master_secret;

    let mut master_public_key_bytes = Vec::new();

    master_public
        .serialize_compressed(&mut master_public_key_bytes)
        .expect("serialize master public key");

    let master_public_key: BoundedMasterPublicKey = master_public_key_bytes
        .try_into()
        .expect("compressed BLS12-381 G2 key length");

    let epoch_key = IbeEpochPublicKey {
        epoch: args.epoch,
        key_id,
        master_public_key,
        total_weight,
        threshold_weight,
        first_block: args.first_block,
        last_block: args.last_block,
    };

    let mut all_public_atoms = Vec::<PublicShareAtom>::new();
    let mut validator_local_atoms = vec![Vec::<WeightedSecretShareAtom>::new(); args.validators];

    for validator_index in 0..args.validators {
        for local_atom in 0..args.atoms_per_validator {
            let global_atom_index = validator_index * args.atoms_per_validator + local_atom;

            // Shamir x coordinate. Must be nonzero.
            let share_id = (global_atom_index + 1) as u32;
            let x = Scalar::from(share_id as u64);

            let secret_scalar = eval_poly(&coeffs, x);
            let public_share = PublicKeyGroup::generator() * secret_scalar;

            let mut secret_scalar_bytes = Vec::new();

            secret_scalar
                .serialize_compressed(&mut secret_scalar_bytes)
                .expect("serialize secret scalar");

            let mut public_share_bytes = Vec::new();

            public_share
                .serialize_compressed(&mut public_share_bytes)
                .expect("serialize public share");

            let public_atom = PublicShareAtom {
                share_id,
                weight: 1,
                public_share: public_share_bytes,
            };

            all_public_atoms.push(public_atom.clone());

            validator_local_atoms[validator_index].push(WeightedSecretShareAtom {
                public: public_atom,
                secret_scalar: secret_scalar_bytes,
            });
        }
    }

    let public_output = EpochDkgPublicOutput {
        epoch_key: epoch_key.clone(),
        public_atoms: all_public_atoms,
    };

    let key_id_hex = hex::encode(key_id);
    let filename = format!("epoch-{}-{key_id_hex}.scale", args.epoch);

    fs::create_dir_all(&args.out).expect("create output root");

    write_scale(
        args.out.join("public").join(&filename),
        public_output.clone(),
    );

    for validator_index in 0..args.validators {
        let bundle = EpochSecretShareBundle {
            public: public_output.clone(),
            validator_authority: validator_index.to_be_bytes().to_vec(),
            local_atoms: validator_local_atoms[validator_index].clone(),
        };

        let validator_dkg_dir = args
            .out
            .join(format!("validator-{validator_index}"))
            .join("mev_shield_ibe")
            .join("dkg");

        write_scale(
            validator_dkg_dir.join("public").join(&filename),
            public_output.clone(),
        );

        write_scale(validator_dkg_dir.join("secret").join(&filename), bundle);
    }

    println!("Generated dev MEVShield v2 DKG bundles:");
    println!("  epoch: {}", args.epoch);
    println!("  key_id: 0x{}", key_id_hex);
    println!("  total_weight: {}", total_weight);
    println!("  threshold_weight: {}", threshold_weight);
    println!("  validators: {}", args.validators);
    println!("  atoms_per_validator: {}", args.atoms_per_validator);
}
