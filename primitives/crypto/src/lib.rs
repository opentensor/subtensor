//! BLSAG (Back's Linkable Spontaneous Anonymous Group) ring signatures over Ristretto255.
//!
//! This crate provides sign, verify, key image generation, and linkability detection
//! for BLSAG ring signatures using the Ristretto255 group (compatible with Sr25519 keys).
//!
//! # Algorithm Reference
//!
//! The implementation follows "Zero to Monero: Second Edition" (ZtM2), Section 3.4
//! "Back's Linkable Spontaneous Anonymous Group (bLSAG) signatures", pages 29-31.
//! <https://www.getmonero.org/library/Zero-to-Monero-2-0-0.pdf>
//!
//! # Deviations from ZtM2 Section 3.4
//!
//! The following hardening measures go beyond the basic algorithm described in ZtM2:
//!
//! 1. **Ring binding (key prefixing):** The ring and key image are pre-hashed into a
//!    64-byte digest included in every challenge hash. ZtM2 notes "adding the prefix is
//!    standard practice" but the bLSAG description omits it. CLSAG (ZtM2 §3.6) includes
//!    it. This prevents ring substitution / Fiat-Shamir transcript manipulation.
//!
//! 2. **Domain separation:** Each hash function (Hp, challenge, ring binding) uses a unique
//!    domain-separated prefix. This prevents outputs from one function being valid inputs
//!    to another, blocking cross-protocol attacks. (ZtM2 §3.6 footnote 19 recommends this.)
//!
//! 3. **Identity point rejection:** Both the key image and all ring members are checked
//!    against the identity point (all-zero bytes in Ristretto). ZtM2 §3.4 Verification
//!    Step 1 checks `l * K_tilde == 0` for the key image; our Ristretto choice makes this
//!    unnecessary (cofactor = 1), but we must still reject the identity explicitly.
//!
//! 4. **Canonical scalar validation:** All scalar inputs (challenge, responses) are checked
//!    to be in canonical form (< group order) via `Scalar::from_canonical_bytes()`.
//!
//! 5. **Secret zeroization:** The private key copy and random nonce are wiped from memory
//!    after signing to mitigate memory-dump attacks.
//!
//! 6. **Blake2b512 hardcoded:** Instead of a generic hash parameter, the hash function is
//!    fixed to Blake2b512. This avoids misuse from weak hash choices and simplifies auditing.
//!
//! 7. **Ristretto255 (cofactor 1):** Using Ristretto instead of raw Ed25519 eliminates the
//!    cofactor-related key image forgery described in ZtM2 §3.4 (the `l * K_tilde == 0`
//!    check). Ristretto points are always in the prime-order subgroup.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "signing")]
use alloc::vec;
use alloc::vec::Vec;
use blake2::Blake2b512;
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::MultiscalarMul,
};
use digest::Digest;
#[cfg(feature = "signing")]
use rand_core::{CryptoRng, RngCore};
#[cfg(feature = "signing")]
use zeroize::Zeroize;

// ==========================================================================
// Domain separators
// ==========================================================================
//
// These prevent hash outputs from one function being valid for another.
// They are protocol-binding: changing them breaks all existing signatures.
//
// SECURITY: Domain separation is not present in the basic bLSAG description
// (ZtM2 §3.4) but is recommended for all new hash function uses (ZtM2 §3.6
// footnote 19). Without it, an attacker could potentially swap hash outputs
// between Hp and the challenge hash.

/// Domain separator for the hash-to-point function Hp (ZtM2 §3.4 notation: `Hp`).
const DOMAIN_HASH_TO_POINT: &[u8] = b"SubtensorBLSAG_hash_to_point";

/// Domain separator for the challenge hash function Hn (ZtM2 §3.4 notation: `Hn`).
const DOMAIN_CHALLENGE: &[u8] = b"SubtensorBLSAG_challenge";

/// Domain separator for the ring binding pre-hash (not in ZtM2; added for key prefixing).
const DOMAIN_RING_BINDING: &[u8] = b"SubtensorBLSAG_ring_binding";

/// Errors that can occur during BLSAG operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub enum BlsagError {
    /// Ring must contain at least 2 members for anonymity.
    RingTooSmall,
    /// Key image bytes are not a valid compressed Ristretto point, or represent the identity.
    InvalidKeyImage,
    /// A ring member's bytes are not a valid compressed Ristretto point, or represent the
    /// identity.
    InvalidRingMember,
    /// Scalar bytes are not in canonical encoding (must be < group order).
    InvalidScalar,
    /// The number of response scalars does not match the ring size.
    ResponseCountMismatch,
    /// The signer's derived public key was not found in the ring.
    SignerNotInRing,
}

/// A BLSAG ring signature.
///
/// Corresponds to ZtM2 §3.4: `sigma(m) = (c_1, r_1, ..., r_n)` with key image `K_tilde`.
///
/// The ring R is NOT included — it must be provided separately for verification.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    codec::Encode,
    codec::Decode,
    codec::DecodeWithMemTracking,
    scale_info::TypeInfo,
)]
pub struct BlsagSignature {
    /// Initial challenge scalar c_0 (32 bytes, canonical encoding).
    /// Called `c_1` in ZtM2 §3.4 (1-indexed), we use 0-indexed.
    pub challenge: [u8; 32],
    /// Response scalars, one per ring member (each 32 bytes, canonical encoding).
    /// Called `r_1, ..., r_n` in ZtM2 §3.4.
    pub responses: Vec<[u8; 32]>,
    /// Key image: compressed Ristretto point (32 bytes).
    /// Called `K_tilde` in ZtM2 §3.4. Deterministic per private key.
    pub key_image: [u8; 32],
}

// ==========================================================================
// Internal helpers
// ==========================================================================
//
// These are shared between sign() and verify() to guarantee identical hash
// computations. Any mismatch between sign and verify would silently break
// all signatures, so factoring them out is a critical correctness measure.

/// Deserialize 32 bytes to a Scalar, rejecting non-canonical encodings.
///
/// SECURITY (not in ZtM2): Ensures all scalars are < group order `l`.
/// Non-canonical scalars could cause subtle verification bypass.
fn deserialize_scalar(bytes: &[u8; 32]) -> Result<Scalar, BlsagError> {
    Option::from(Scalar::from_canonical_bytes(*bytes)).ok_or(BlsagError::InvalidScalar)
}

/// Decompress 32 bytes to a RistrettoPoint.
///
/// Returns `None` if the bytes are not a valid compressed Ristretto encoding.
fn decompress_point(bytes: &[u8; 32]) -> Option<RistrettoPoint> {
    CompressedRistretto(*bytes).decompress()
}

/// `Hp`: Deterministically hash a Ristretto point to another Ristretto point.
///
/// ZtM2 §3.4: "Assume the existence of a hash function `Hp`, which maps to curve points."
/// (ZtM2 page 30, footnotes 10-11)
///
/// Used to compute key images: `K_tilde = k * Hp(K)`.
///
/// Uses `RistrettoPoint::from_hash()` which internally applies the Elligator 2 map,
/// a standard and secure hash-to-curve method for Ristretto. This is simpler and more
/// robust than the try-and-increment approach used with secp256k1 curves.
///
/// SECURITY: The domain separator ensures this function's outputs are independent from
/// the challenge hash. If they shared a domain, an attacker could manipulate key images.
fn hash_to_point(point: &RistrettoPoint) -> RistrettoPoint {
    RistrettoPoint::from_hash(
        Blake2b512::new()
            .chain_update(DOMAIN_HASH_TO_POINT)
            .chain_update(point.compress().as_bytes()),
    )
}

/// Pre-compute a binding digest of the ring composition and key image.
///
/// NOT IN ZtM2 §3.4 — added for Fiat-Shamir security (key prefixing).
///
/// The Fiat-Shamir heuristic requires hashing the *entire public statement* to prevent
/// transcript manipulation. The basic bLSAG description (ZtM2 §3.4) does not include
/// the ring in the challenge hash. ZtM2 itself notes (page 31, bottom):
/// "adding the prefix is standard practice for similar signature schemes."
/// CLSAG (ZtM2 §3.6) explicitly includes the ring R in every challenge.
///
/// We pre-hash the ring + key image into a 64-byte digest (rather than hashing the
/// full ring in every challenge iteration) for efficiency: one extra hash at the start,
/// instead of O(n) extra data per iteration.
fn compute_ring_binding(ring: &[[u8; 32]], key_image: &[u8; 32]) -> [u8; 64] {
    let mut h = Blake2b512::new();
    h.update(DOMAIN_RING_BINDING);
    for pubkey in ring {
        h.update(pubkey);
    }
    h.update(key_image);
    h.finalize().into()
}

/// Compute a challenge scalar from the ring binding, message, and two commitment points.
///
/// ZtM2 §3.4 Signature Step 3 / Step 4 (and Verification Step 2):
/// ```text
/// c_{i+1} = Hn(m, [r_i * G + c_i * K_i], [r_i * Hp(K_i) + c_i * K_tilde])
/// ```
///
/// Our version adds domain separation and ring binding:
/// ```text
/// c = H(DOMAIN_CHALLENGE || ring_binding || message || compress(L0) || compress(L1))
/// ```
///
/// Uses `Scalar::from_hash` with a 512-bit hash to ensure uniform distribution over
/// the scalar field with negligible bias (256-bit output would have ~1-bit bias for
/// a ~253-bit field order).
fn compute_challenge(
    ring_binding: &[u8; 64],
    message: &[u8],
    l0: &RistrettoPoint,
    l1: &RistrettoPoint,
) -> Scalar {
    Scalar::from_hash(
        Blake2b512::new()
            .chain_update(DOMAIN_CHALLENGE)
            .chain_update(ring_binding)
            .chain_update(message)
            .chain_update(l0.compress().as_bytes())
            .chain_update(l1.compress().as_bytes()),
    )
}

// ==========================================================================
// Public API
// ==========================================================================

/// Generate a key image from a private key.
///
/// ZtM2 §3.4 Signature Step 1:
/// ```text
/// K_tilde = k_pi * Hp(K_pi)
/// ```
///
/// The key image is deterministic: the same private key always produces the same image.
/// It does not reveal the private key (due to the DLP on Hp(K_pi)).
///
/// Requires the `signing` feature.
#[cfg(feature = "signing")]
pub fn generate_key_image(private_key: &[u8; 32]) -> Result<[u8; 32], BlsagError> {
    // ZtM2 §3.4 Step 1: K_tilde = k_pi * Hp(K_pi)
    let k = deserialize_scalar(private_key)?;
    let k_point = k * RISTRETTO_BASEPOINT_POINT; // K_pi = k_pi * G
    let hp = hash_to_point(&k_point); // Hp(K_pi)
    let key_image = k * hp; // K_tilde = k_pi * Hp(K_pi)
    Ok(key_image.compress().to_bytes())
}

/// Create a BLSAG ring signature.
///
/// Implements ZtM2 §3.4 "Signature" (page 30-31), with additional hardening.
///
/// # Arguments
///
/// * `private_key` — signer's private key (32-byte canonical scalar).
///   Called `k_pi` in ZtM2 §3.4.
/// * `ring` — the **complete** ring R of public keys as compressed Ristretto points,
///   including the signer's own public key K_pi. Must contain at least 2 members.
/// * `message` — the message `m` to sign.
/// * `rng` — a cryptographically secure RNG. A weak or deterministic RNG **will** leak the
///   private key or destroy anonymity. See ZtM2 §2.3.4: reusing alpha leaks k.
///
/// The function automatically locates the signer's secret index `pi` by deriving
/// the public key from `private_key` and searching the ring.
///
/// Requires the `signing` feature.
#[cfg(feature = "signing")]
pub fn sign(
    private_key: &[u8; 32],
    ring: &[[u8; 32]],
    message: &[u8],
    rng: &mut (impl CryptoRng + RngCore),
) -> Result<BlsagSignature, BlsagError> {
    let n = ring.len();

    // SECURITY (not in ZtM2): Minimum ring size check.
    // A ring of 1 provides zero anonymity — the signer is trivially identified.
    if n < 2 {
        return Err(BlsagError::RingTooSmall);
    }

    // Deserialize the private key k_pi and derive the public key K_pi = k_pi * G.
    let k = deserialize_scalar(private_key)?;
    let k_point = k * RISTRETTO_BASEPOINT_POINT;

    // Decompress and validate every ring member.
    // SECURITY (not in ZtM2): Reject identity points. If P_i = identity, then c_i * P_i
    // vanishes in L0, decoupling that member from the challenge chain. An attacker could
    // insert dummy members that don't correspond to real keys.
    let ring_points: Vec<RistrettoPoint> = ring
        .iter()
        .map(|bytes| {
            if *bytes == [0u8; 32] {
                return Err(BlsagError::InvalidRingMember);
            }
            decompress_point(bytes).ok_or(BlsagError::InvalidRingMember)
        })
        .collect::<Result<_, _>>()?;

    // Find the signer's secret index `pi` in the ring.
    // ZtM2 §3.4: "k_pi the signer's private key corresponding to his public key K_pi in R,
    // where pi is a secret index."
    let secret_index = ring_points
        .iter()
        .position(|p| p == &k_point)
        .ok_or(BlsagError::SignerNotInRing)?;

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Signature Step 1: Calculate key image
    //   K_tilde = k_pi * Hp(K_pi)
    // ---------------------------------------------------------------
    let hp_signer = hash_to_point(&k_point);
    let key_image = k * hp_signer;
    let key_image_bytes = key_image.compress().to_bytes();

    // ADDED (not in ZtM2): Pre-compute the ring binding digest for key prefixing.
    // This binds the entire ring composition and key image into every challenge hash,
    // preventing ring substitution attacks on the Fiat-Shamir transcript.
    let ring_binding = compute_ring_binding(ring, &key_image_bytes);

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Signature Step 2: Generate random numbers
    //   alpha in_R Z_l   (the signer's secret nonce)
    //   r_i   in_R Z_l   for i != pi  (fake responses for non-signers)
    // ---------------------------------------------------------------
    //
    // SECURITY: alpha is the core secret of this signature instance.
    // If alpha is ever reused across different challenges, the private key k is
    // trivially recoverable: k = (r - r') / (c - c'). See ZtM2 §2.3.4.
    let alpha = Scalar::random(&mut *rng);

    // Pre-fill ALL positions with random responses; the signer's slot (index pi)
    // will be overwritten in Step 5. This is equivalent to the ZtM2 formulation
    // where r_i for i != pi are generated randomly.
    let mut responses: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut *rng)).collect();
    let mut challenges: Vec<Scalar> = vec![Scalar::ZERO; n];

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Signature Step 3: Compute initial challenge
    //   c_{pi+1} = Hn(m, [alpha * G], [alpha * Hp(K_pi)])
    // ---------------------------------------------------------------
    let l0 = alpha * RISTRETTO_BASEPOINT_POINT; // alpha * G
    let l1 = alpha * hp_signer; // alpha * Hp(K_pi)

    let start = (secret_index + 1) % n;
    challenges[start] = compute_challenge(&ring_binding, message, &l0, &l1);

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Signature Step 4: For i = pi+1, ..., n, 1, ..., pi-1
    //   compute c_{i+1} = Hn(m, [r_i*G + c_i*K_i], [r_i*Hp(K_i) + c_i*K_tilde])
    //
    // This walks the ring from (pi+1) back around to pi, building the
    // chain of challenges. Each step uses a non-signer's random response
    // r_i and the previous challenge c_i.
    // ---------------------------------------------------------------
    let mut i = start;
    while i != secret_index {
        let hp_i = hash_to_point(&ring_points[i]); // Hp(K_i)

        // L0_i = r_i * G + c_i * K_i
        let l0_i = RistrettoPoint::multiscalar_mul(
            &[responses[i], challenges[i]],
            &[RISTRETTO_BASEPOINT_POINT, ring_points[i]],
        );
        // L1_i = r_i * Hp(K_i) + c_i * K_tilde
        let l1_i = RistrettoPoint::multiscalar_mul(
            &[responses[i], challenges[i]],
            &[hp_i, key_image],
        );

        let next = (i + 1) % n;
        challenges[next] = compute_challenge(&ring_binding, message, &l0_i, &l1_i);
        i = next;
    }

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Signature Step 5: Define the real response
    //   r_pi = alpha - c_pi * k_pi  (mod l)
    //
    // This "closes the ring": it makes the challenge chain consistent
    // so that verification starting from c_0 will loop back to c_0.
    // This is the ONLY step that uses the private key k_pi.
    // ---------------------------------------------------------------
    responses[secret_index] = alpha - (challenges[secret_index] * k);

    // ---------------------------------------------------------------
    // ZtM2 §3.4: "The signature will be sigma(m) = (c_1, r_1, ..., r_n),
    //             with key image K_tilde and ring R."
    //
    // We use 0-indexed: sigma = (c_0, r_0, ..., r_{n-1}), key_image.
    // The ring R is NOT included in the signature — it is provided
    // separately for verification (from on-chain storage).
    // ---------------------------------------------------------------
    let result = BlsagSignature {
        challenge: challenges[0].to_bytes(),
        responses: responses.iter().map(|s| s.to_bytes()).collect(),
        key_image: key_image_bytes,
    };

    // SECURITY (not in ZtM2): Wipe secret material from memory.
    //
    // k (private key copy) and alpha (nonce) are the critical secrets.
    // If alpha is recovered from a memory dump alongside the published signature,
    // the private key is trivially computable:
    //   k = (alpha - r_pi) / c_pi
    //
    // curve25519-dalek's Scalar implements Zeroize, which overwrites the
    // memory with zeros before deallocation.
    let mut k = k;
    let mut alpha = alpha;
    k.zeroize();
    alpha.zeroize();

    Ok(result)
}

/// Verify a BLSAG ring signature.
///
/// Implements ZtM2 §3.4 "Verification" (page 31), with additional hardening.
///
/// # Arguments
///
/// * `signature` — the BLSAG signature sigma(m) = (c_0, r_0, ..., r_{n-1}).
/// * `ring` — the ring R of public keys (compressed Ristretto points), in the
///   **same order** used during signing.
/// * `message` — the message `m` that was signed.
///
/// # Returns
///
/// * `Ok(true)` — signature is valid (the challenge chain closes).
/// * `Ok(false)` — signature is mathematically invalid.
/// * `Err(BlsagError)` — inputs are malformed.
pub fn verify(
    signature: &BlsagSignature,
    ring: &[[u8; 32]],
    message: &[u8],
) -> Result<bool, BlsagError> {
    let n = ring.len();

    // SECURITY (not in ZtM2): Minimum ring size.
    if n < 2 {
        return Err(BlsagError::RingTooSmall);
    }

    // SECURITY (not in ZtM2): Response count must match ring size.
    // A mismatch means the signature is structurally invalid.
    if signature.responses.len() != n {
        return Err(BlsagError::ResponseCountMismatch);
    }

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Verification Step 1: Check l * K_tilde == 0
    //
    // On Ed25519 (cofactor h=8), this ensures the key image is in the
    // prime-order subgroup, preventing cofactor-based forgeries (ZtM2 §3.4
    // page 31: "it is possible to add an EC point from the subgroup of
    // size h... make h unlinked valid signatures").
    //
    // On Ristretto255 (cofactor 1), ALL valid points are in the prime-order
    // subgroup by construction, so this check is automatically satisfied.
    // Instead, we explicitly reject the IDENTITY point, which is the only
    // "degenerate" Ristretto point that could cause problems.
    //
    // SECURITY: If I = identity, then c * I = identity for all c, and the
    // L1 term degenerates to just r * Hp(P_i). This decouples the key image
    // from the challenge chain, meaning ANY I would verify — enabling forgery.
    // ---------------------------------------------------------------
    if signature.key_image == [0u8; 32] {
        return Err(BlsagError::InvalidKeyImage);
    }

    // Decompress the key image K_tilde.
    let key_image = decompress_point(&signature.key_image).ok_or(BlsagError::InvalidKeyImage)?;

    // Decompress and validate ring members {K_1, ..., K_n}.
    // SECURITY (not in ZtM2): Identity points in the ring are rejected because
    // if P_i = identity, then c * P_i = identity in L0, and the challenge chain
    // loses binding to that member's key. An attacker could insert dummy members.
    let ring_points: Vec<RistrettoPoint> = ring
        .iter()
        .map(|bytes| {
            if *bytes == [0u8; 32] {
                return Err(BlsagError::InvalidRingMember);
            }
            decompress_point(bytes).ok_or(BlsagError::InvalidRingMember)
        })
        .collect::<Result<_, _>>()?;

    // SECURITY (not in ZtM2): Validate all scalars are canonical (< group order).
    let c0 = deserialize_scalar(&signature.challenge)?;
    let responses: Vec<Scalar> = signature
        .responses
        .iter()
        .map(|bytes| deserialize_scalar(bytes))
        .collect::<Result<_, _>>()?;

    // ADDED (not in ZtM2): Pre-compute the ring binding digest.
    // Must be identical to what sign() computed for the same ring and key image.
    let ring_binding = compute_ring_binding(ring, &signature.key_image);

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Verification Step 2:
    //   For i = 1, 2, ..., n iteratively compute, replacing n+1 -> 1:
    //     c'_{i+1} = Hn(m, [r_i*G + c_i*K_i], [r_i*Hp(K_i) + c_i*K_tilde])
    //
    // Starting from c_0, we recompute the entire challenge chain.
    // At the signer's position pi, the response r_pi was specifically
    // crafted so that:
    //   r_pi*G + c_pi*K_pi  =  alpha*G       (the L0 from signing)
    //   r_pi*Hp(K_pi) + c_pi*K_tilde  =  alpha*Hp(K_pi)  (the L1)
    //
    // This makes the reconstructed challenge at (pi+1) match the
    // original, and the chain "closes" back to c_0.
    // ---------------------------------------------------------------
    let mut reconstructed_c = c0;

    for j in 0..n {
        // Hp(K_j) — hash ring member's public key to a curve point
        let hp_j = hash_to_point(&ring_points[j]);

        // L0_j = r_j * G + c_j * K_j
        let l0 = RistrettoPoint::multiscalar_mul(
            &[responses[j], reconstructed_c],
            &[RISTRETTO_BASEPOINT_POINT, ring_points[j]],
        );

        // L1_j = r_j * Hp(K_j) + c_j * K_tilde
        let l1 = RistrettoPoint::multiscalar_mul(
            &[responses[j], reconstructed_c],
            &[hp_j, key_image],
        );

        // c_{j+1} = Hn(ring_binding, m, L0_j, L1_j)
        reconstructed_c = compute_challenge(&ring_binding, message, &l0, &l1);
    }

    // ---------------------------------------------------------------
    // ZtM2 §3.4 Verification Step 3:
    //   "If c_1 = c'_1 then the signature is valid."
    //
    // (0-indexed: if c_0 == reconstructed c_0)
    //
    // SECURITY: curve25519-dalek's Scalar PartialEq uses ct_eq internally,
    // making this comparison constant-time to prevent timing side-channels
    // that could leak information about the challenge values.
    // ---------------------------------------------------------------
    Ok(reconstructed_c == c0)
}

/// Check whether two key images were produced by the same private key.
///
/// ZtM2 §3.4 "Linkability" (page 32):
/// "if K_tilde = K_tilde' then clearly both signatures come from the same private key."
///
/// If two valid BLSAG signatures yield the same key image, they were created
/// by the same signer — regardless of the ring or message used. This is how
/// double-spending / double-voting is detected.
pub fn link(key_image_1: &[u8; 32], key_image_2: &[u8; 32]) -> bool {
    key_image_1 == key_image_2
}

/// Check if 32 bytes represent a valid, non-identity compressed Ristretto point.
pub fn verify_point_valid(bytes: &[u8; 32]) -> bool {
    if *bytes == [0u8; 32] {
        return false;
    }
    decompress_point(bytes).is_some()
}

// ==========================================================================
// Tests
// ==========================================================================

#[cfg(test)]
#[cfg(feature = "signing")]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    /// Generate a random (private_key, public_key) pair as raw 32-byte arrays.
    fn random_keypair(rng: &mut (impl CryptoRng + RngCore)) -> ([u8; 32], [u8; 32]) {
        let k = Scalar::random(rng);
        let p = (k * RISTRETTO_BASEPOINT_POINT).compress().to_bytes();
        (k.to_bytes(), p)
    }

    /// Build a ring of `n` members with the signer at position `n / 2`.
    fn setup_ring(n: usize) -> (Vec<[u8; 32]>, [u8; 32]) {
        let mut rng = OsRng;
        let (signer_sk, signer_pk) = random_keypair(&mut rng);
        let signer_pos = n / 2;

        let mut ring = Vec::with_capacity(n);
        for i in 0..n {
            if i == signer_pos {
                ring.push(signer_pk);
            } else {
                let (_, pk) = random_keypair(&mut rng);
                ring.push(pk);
            }
        }
        (ring, signer_sk)
    }

    // -----------------------------------------------------------------------
    // Happy path
    // -----------------------------------------------------------------------

    #[test]
    fn sign_and_verify_basic() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);
        let msg = b"hello world";

        let sig = sign(&sk, &ring, msg, &mut rng).unwrap();
        assert!(verify(&sig, &ring, msg).unwrap());
    }

    #[test]
    fn sign_and_verify_various_ring_sizes() {
        let mut rng = OsRng;
        for size in [2, 3, 5, 8, 16, 32] {
            let (ring, sk) = setup_ring(size);
            let msg = b"ring size test";

            let sig = sign(&sk, &ring, msg, &mut rng).unwrap();
            assert!(verify(&sig, &ring, msg).unwrap(), "failed for ring size {size}");
        }
    }

    #[test]
    fn signer_at_every_position() {
        let mut rng = OsRng;
        let n = 5;
        let (sk, pk) = random_keypair(&mut rng);

        for pos in 0..n {
            let mut ring = Vec::with_capacity(n);
            for i in 0..n {
                if i == pos {
                    ring.push(pk);
                } else {
                    let (_, other_pk) = random_keypair(&mut rng);
                    ring.push(other_pk);
                }
            }

            let sig = sign(&sk, &ring, b"position test", &mut rng).unwrap();
            assert!(
                verify(&sig, &ring, b"position test").unwrap(),
                "failed with signer at position {pos}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Key image / linkability (ZtM2 §3.4 "Linkability", page 32)
    // -----------------------------------------------------------------------

    #[test]
    fn key_image_is_deterministic() {
        let mut rng = OsRng;
        let (sk, _) = random_keypair(&mut rng);

        let ki1 = generate_key_image(&sk).unwrap();
        let ki2 = generate_key_image(&sk).unwrap();
        assert_eq!(ki1, ki2);
    }

    #[test]
    fn key_image_matches_signature() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let ki = generate_key_image(&sk).unwrap();
        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        assert_eq!(ki, sig.key_image);
    }

    #[test]
    fn same_signer_different_messages_linked() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig1 = sign(&sk, &ring, b"message A", &mut rng).unwrap();
        let sig2 = sign(&sk, &ring, b"message B", &mut rng).unwrap();

        assert!(link(&sig1.key_image, &sig2.key_image));
    }

    #[test]
    fn same_signer_different_rings_linked() {
        let mut rng = OsRng;
        let (sk, pk) = random_keypair(&mut rng);

        let mut ring1 = vec![pk];
        let mut ring2 = vec![pk];
        for _ in 0..4 {
            let (_, other1) = random_keypair(&mut rng);
            let (_, other2) = random_keypair(&mut rng);
            ring1.push(other1);
            ring2.push(other2);
        }

        let sig1 = sign(&sk, &ring1, b"msg", &mut rng).unwrap();
        let sig2 = sign(&sk, &ring2, b"msg", &mut rng).unwrap();

        assert!(link(&sig1.key_image, &sig2.key_image));
    }

    #[test]
    fn different_signers_not_linked() {
        let mut rng = OsRng;
        let (ring1, sk1) = setup_ring(5);
        let (ring2, sk2) = setup_ring(5);

        let sig1 = sign(&sk1, &ring1, b"msg", &mut rng).unwrap();
        let sig2 = sign(&sk2, &ring2, b"msg", &mut rng).unwrap();

        assert!(!link(&sig1.key_image, &sig2.key_image));
    }

    // -----------------------------------------------------------------------
    // Verification failures (invalid signatures)
    // -----------------------------------------------------------------------

    #[test]
    fn wrong_message_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig = sign(&sk, &ring, b"correct", &mut rng).unwrap();
        assert!(!verify(&sig, &ring, b"wrong").unwrap());
    }

    #[test]
    fn wrong_ring_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);
        let (wrong_ring, _) = setup_ring(5);

        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        assert!(!verify(&sig, &wrong_ring, b"test").unwrap());
    }

    #[test]
    fn tampered_challenge_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        sig.challenge = Scalar::random(&mut rng).to_bytes();
        assert!(!verify(&sig, &ring, b"test").unwrap());
    }

    #[test]
    fn tampered_response_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        sig.responses[0] = Scalar::random(&mut rng).to_bytes();
        assert!(!verify(&sig, &ring, b"test").unwrap());
    }

    #[test]
    fn wrong_key_image_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        sig.key_image = RistrettoPoint::random(&mut rng).compress().to_bytes();
        assert!(!verify(&sig, &ring, b"test").unwrap());
    }

    // -----------------------------------------------------------------------
    // Input validation errors
    // -----------------------------------------------------------------------

    #[test]
    fn ring_too_small_sign() {
        let mut rng = OsRng;
        let (sk, pk) = random_keypair(&mut rng);

        assert_eq!(sign(&sk, &[pk], b"test", &mut rng), Err(BlsagError::RingTooSmall));
        assert_eq!(sign(&sk, &[], b"test", &mut rng), Err(BlsagError::RingTooSmall));
    }

    #[test]
    fn ring_too_small_verify() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);
        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();

        assert_eq!(verify(&sig, &[ring[0]], b"test"), Err(BlsagError::RingTooSmall));
    }

    #[test]
    fn signer_not_in_ring() {
        let mut rng = OsRng;
        let (ring, _) = setup_ring(5);
        let (outsider_sk, _) = random_keypair(&mut rng);

        assert_eq!(
            sign(&outsider_sk, &ring, b"test", &mut rng),
            Err(BlsagError::SignerNotInRing)
        );
    }

    #[test]
    fn response_count_mismatch() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        sig.responses.pop();
        assert_eq!(verify(&sig, &ring, b"test"), Err(BlsagError::ResponseCountMismatch));
    }

    #[test]
    fn identity_key_image_rejected() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        sig.key_image = [0u8; 32];
        assert_eq!(verify(&sig, &ring, b"test"), Err(BlsagError::InvalidKeyImage));
    }

    #[test]
    fn identity_ring_member_rejected_sign() {
        let mut rng = OsRng;
        let (sk, pk) = random_keypair(&mut rng);
        let (_, pk2) = random_keypair(&mut rng);
        let ring = [[0u8; 32], pk, pk2];

        assert_eq!(sign(&sk, &ring, b"test", &mut rng), Err(BlsagError::InvalidRingMember));
    }

    #[test]
    fn identity_ring_member_rejected_verify() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(3);

        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        let mut bad_ring = ring.clone();
        bad_ring[0] = [0u8; 32];
        assert_eq!(verify(&sig, &bad_ring, b"test"), Err(BlsagError::InvalidRingMember));
    }

    #[test]
    fn invalid_ring_member_bytes_rejected() {
        let mut rng = OsRng;
        let (sk, pk) = random_keypair(&mut rng);
        let (_, pk2) = random_keypair(&mut rng);
        let ring = [[0xFFu8; 32], pk, pk2];

        assert_eq!(sign(&sk, &ring, b"test", &mut rng), Err(BlsagError::InvalidRingMember));
    }

    // -----------------------------------------------------------------------
    // Additional coverage (inspired by Monero CLSAG test patterns)
    // -----------------------------------------------------------------------

    #[test]
    fn tamper_each_response_individually() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);
        let msg = b"tamper each response";

        let sig = sign(&sk, &ring, msg, &mut rng).unwrap();

        for idx in 0..sig.responses.len() {
            let mut bad = sig.clone();
            bad.responses[idx] = Scalar::random(&mut rng).to_bytes();
            assert!(
                !verify(&bad, &ring, msg).unwrap(),
                "tampered response at index {idx} should fail verification"
            );
        }
    }

    #[test]
    fn too_many_responses_rejected() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        sig.responses.push(Scalar::random(&mut rng).to_bytes());
        assert_eq!(verify(&sig, &ring, b"test"), Err(BlsagError::ResponseCountMismatch));
    }

    #[test]
    fn swap_single_ring_member_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();

        // Replace each ring member one at a time with a random key
        for idx in 0..ring.len() {
            let mut bad_ring = ring.clone().to_vec();
            let (_, imposter) = random_keypair(&mut rng);
            bad_ring[idx] = imposter;
            assert!(
                !verify(&sig, &bad_ring, b"test").unwrap(),
                "swapped ring member at index {idx} should fail verification"
            );
        }
    }

    #[test]
    fn non_canonical_challenge_rejected() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();

        // Set challenge to a value >= the group order l.
        // l = 2^252 + 27742317777372353535851937790883648493
        // A simple non-canonical value: all 0xFF bytes (much larger than l).
        sig.challenge = [0xFF; 32];
        assert_eq!(verify(&sig, &ring, b"test"), Err(BlsagError::InvalidScalar));
    }

    #[test]
    fn non_canonical_response_rejected() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        sig.responses[0] = [0xFF; 32];
        assert_eq!(verify(&sig, &ring, b"test"), Err(BlsagError::InvalidScalar));
    }

    #[test]
    fn duplicate_ring_members_sign() {
        let mut rng = OsRng;
        let (sk, pk) = random_keypair(&mut rng);
        let (_, other) = random_keypair(&mut rng);

        // Ring with a duplicate: [pk, other, other]
        let ring = [pk, other, other];
        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        // Sign succeeds (the algorithm doesn't forbid it), but verify should still work
        assert!(verify(&sig, &ring, b"test").unwrap());
    }

    #[test]
    fn empty_message() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig = sign(&sk, &ring, b"", &mut rng).unwrap();
        assert!(verify(&sig, &ring, b"").unwrap());
        // Different (non-empty) message must fail
        assert!(!verify(&sig, &ring, b"x").unwrap());
    }

    #[test]
    fn invalid_key_image_bytes_rejected() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let mut sig = sign(&sk, &ring, b"test", &mut rng).unwrap();
        // Non-decompressible key image (not identity, just garbage)
        sig.key_image = [0xDE; 32];
        assert_eq!(verify(&sig, &ring, b"test"), Err(BlsagError::InvalidKeyImage));
    }

    #[test]
    fn reordered_ring_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();

        // Swap first two ring members — ring order matters for challenges
        let mut swapped = ring.to_vec();
        swapped.swap(0, 1);
        assert!(!verify(&sig, &swapped, b"test").unwrap());
    }

    #[test]
    fn ring_with_extra_member_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();

        // Append an extra member — response count won't match
        let mut bigger = ring.to_vec();
        let (_, extra) = random_keypair(&mut rng);
        bigger.push(extra);
        assert_eq!(verify(&sig, &bigger, b"test"), Err(BlsagError::ResponseCountMismatch));
    }

    #[test]
    fn ring_with_fewer_members_rejects() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();

        // Remove last member — response count won't match
        let smaller = &ring[..4];
        assert_eq!(verify(&sig, smaller, b"test"), Err(BlsagError::ResponseCountMismatch));
    }

    #[test]
    fn large_message() {
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let msg = vec![0xAB; 10_000];
        let sig = sign(&sk, &ring, &msg, &mut rng).unwrap();
        assert!(verify(&sig, &ring, &msg).unwrap());
    }

    #[test]
    fn verify_does_not_mutate_state_after_failure() {
        // Ensures that a failed verification doesn't corrupt anything —
        // a valid signature still verifies after checking an invalid one.
        let mut rng = OsRng;
        let (ring, sk) = setup_ring(5);

        let sig = sign(&sk, &ring, b"test", &mut rng).unwrap();

        // Check a tampered signature first
        let mut bad = sig.clone();
        bad.responses[0] = Scalar::random(&mut rng).to_bytes();
        assert!(!verify(&bad, &ring, b"test").unwrap());

        // Original still verifies
        assert!(verify(&sig, &ring, b"test").unwrap());
    }

    #[test]
    fn zero_private_key_rejected() {
        // A zero private key gives k*G = identity, which can't be in a valid ring
        // (identity ring members are rejected). Should fail with SignerNotInRing.
        let mut rng = OsRng;
        let (ring, _) = setup_ring(5);

        let zero_sk = [0u8; 32];
        assert_eq!(sign(&zero_sk, &ring, b"test", &mut rng), Err(BlsagError::SignerNotInRing));
    }
}
