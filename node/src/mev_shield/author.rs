//! MEV-shield author-side helpers: ML‑KEM-768 ephemeral keys + timed key announcement.

use std::{sync::{Arc, Mutex}, time::Duration};

use futures::StreamExt;
use sc_client_api::HeaderBackend;
use sc_transaction_pool_api::TransactionSource;
use sp_api::ProvideRuntimeApi;
use sp_core::{sr25519, blake2_256};
use sp_runtime::traits::Block as BlockT;
use sp_runtime::KeyTypeId;
use tokio::time::sleep;
use blake2::Blake2b512;

use hkdf::Hkdf;
use sha2::Sha256;
use chacha20poly1305::{XChaCha20Poly1305, KeyInit, aead::{Aead, Payload}, XNonce};

use node_subtensor_runtime as runtime; // alias for easier type access
use runtime::{RuntimeCall, UncheckedExtrinsic};

use ml_kem::{MlKem768, KemCore, EncodedSizeUser};
use rand::rngs::OsRng;

/// Parameters controlling time windows inside the slot (milliseconds).
#[derive(Clone)]
pub struct TimeParams {
    pub slot_ms: u64,          // e.g., 12_000
    pub announce_at_ms: u64,   // 7_000
    pub decrypt_window_ms: u64 // 3_000
}

/// Holds the current/next ML‑KEM keypairs and their 32‑byte fingerprints.
#[derive(Clone)]
pub struct MevShieldKeys {
    pub current_sk: Vec<u8>,   // ML‑KEM secret key bytes (encoded form)
    pub current_pk: Vec<u8>,   // ML‑KEM public  key bytes (encoded form)
    pub current_fp: [u8; 32],  // blake2_256(pk)
    pub next_sk: Vec<u8>,
    pub next_pk: Vec<u8>,
    pub next_fp: [u8; 32],
    pub epoch: u64,
}

impl MevShieldKeys {
    pub fn new(epoch: u64) -> Self {
        let (sk, pk) = MlKem768::generate(&mut OsRng);

        // Bring EncodedSizeUser into scope so as_bytes() is available
        let sk_bytes = sk.as_bytes();
        let pk_bytes = pk.as_bytes();
        let sk_slice: &[u8] = sk_bytes.as_ref();
        let pk_slice: &[u8] = pk_bytes.as_ref();

        let current_sk = sk_slice.to_vec();
        let current_pk = pk_slice.to_vec();
        let current_fp = blake2_256(pk_slice);

        let (nsk, npk) = MlKem768::generate(&mut OsRng);
        let nsk_bytes = nsk.as_bytes();
        let npk_bytes = npk.as_bytes();
        let nsk_slice: &[u8] = nsk_bytes.as_ref();
        let npk_slice: &[u8] = npk_bytes.as_ref();
        let next_sk = nsk_slice.to_vec();
        let next_pk = npk_slice.to_vec();
        let next_fp = blake2_256(npk_slice);

        Self { current_sk, current_pk, current_fp, next_sk, next_pk, next_fp, epoch }
    }

    pub fn roll_for_next_slot(&mut self) {
        // Move next -> current
        self.current_sk = core::mem::take(&mut self.next_sk);
        self.current_pk = core::mem::take(&mut self.next_pk);
        self.current_fp = self.next_fp;

        // Generate fresh next
        let (nsk, npk) = MlKem768::generate(&mut OsRng);
        let nsk_bytes = nsk.as_bytes();
        let npk_bytes = npk.as_bytes();
        let nsk_slice: &[u8] = nsk_bytes.as_ref();
        let npk_slice: &[u8] = npk_bytes.as_ref();
        self.next_sk = nsk_slice.to_vec();
        self.next_pk = npk_slice.to_vec();
        self.next_fp = blake2_256(npk_slice);

        self.epoch = self.epoch.saturating_add(1);
    }
}

/// Shared context state.
#[derive(Clone)]
pub struct MevShieldContext {
    pub keys: Arc<Mutex<MevShieldKeys>>,
    pub timing: TimeParams,
}

/// Derive AEAD key directly from the 32‑byte ML‑KEM shared secret.
/// This matches the FFI exactly: AEAD key = shared secret bytes.
pub fn derive_aead_key(ss: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let n = ss.len().min(32);
    key[..n].copy_from_slice(&ss[..n]);
    key
}

/// Plain XChaCha20-Poly1305 decrypt helper
pub fn aead_decrypt(
    key: [u8; 32],
    nonce24: [u8; 24],
    ciphertext: &[u8],
    aad: &[u8],
) -> Option<Vec<u8>> {
    let aead = XChaCha20Poly1305::new((&key).into());
    aead.decrypt(XNonce::from_slice(&nonce24), Payload { msg: ciphertext, aad }).ok()
}

const AURA_KEY_TYPE: KeyTypeId = KeyTypeId(*b"aura");

/// Start background tasks:
///  - per-slot ML‑KEM key rotation
///  - at ~announce_at_ms announce the *next* key **bytes** on chain,
///    signed by the local block author (Aura authority), not an env var.
pub fn spawn_author_tasks<B, C, Pool>(
    task_spawner: &sc_service::SpawnTaskHandle,
    client: std::sync::Arc<C>,
    pool:   std::sync::Arc<Pool>,
    keystore: sp_keystore::KeystorePtr,
    initial_epoch: u64,
    timing: TimeParams,
) -> MevShieldContext
where
    B: sp_runtime::traits::Block,
    // Need block import notifications and headers.
    C: sc_client_api::HeaderBackend<B>
        + sc_client_api::BlockchainEvents<B>
        + Send
        + Sync
        + 'static,
    Pool: sc_transaction_pool_api::TransactionPool<Block = B> + Send + Sync + 'static,
    // We submit an OpaqueExtrinsic into the pool.
    B::Extrinsic: From<sp_runtime::OpaqueExtrinsic>,
{
    let ctx = MevShieldContext {
        keys:   std::sync::Arc::new(std::sync::Mutex::new(MevShieldKeys::new(initial_epoch))),
        timing: timing.clone(),
    };

    // Pick the local Aura authority key that actually authors blocks on this node.
    // We just grab the first Aura sr25519 key in the keystore.
    let aura_keys: Vec<sp_core::sr25519::Public> = keystore.sr25519_public_keys(AURA_KEY_TYPE);
    let local_aura_pub: Option<sp_core::sr25519::Public> = aura_keys.get(0).cloned();

    if local_aura_pub.is_none() {
        log::warn!(
            target: "mev-shield",
            "spawn_author_tasks: no local Aura sr25519 key in keystore; \
             this node will NOT announce MEV‑Shield keys"
        );
        return ctx;
    }

    let local_aura_pub = local_aura_pub.expect("checked is_some; qed");

    // Clone handles for the async task.
    let ctx_clone       = ctx.clone();
    let client_clone    = client.clone();
    let pool_clone      = pool.clone();
    let keystore_clone  = keystore.clone();

    // Slot tick / key-announce loop (roll at end of slot).
    task_spawner.spawn(
        "mev-shield-keys-and-announce",
        None,
        async move {
            use futures::StreamExt;
            use sp_consensus::BlockOrigin;

            let mut import_stream = client_clone.import_notification_stream();
            let mut local_nonce: u32 = 0;

            while let Some(notif) = import_stream.next().await {
                // ✅ Only act on blocks that *this node* authored.
                if notif.origin != BlockOrigin::Own {
                    continue;
                }

                // This block is the start of a slot for which we are the author.
                let (epoch_now, curr_pk_len, next_pk_len) = {
                    let k = ctx_clone.keys.lock().unwrap();
                    (k.epoch, k.current_pk.len(), k.next_pk.len())
                };

                log::info!(
                    target: "mev-shield",
                    "Slot start (local author): epoch={} (pk sizes: curr={}B, next={}B)",
                    epoch_now, curr_pk_len, next_pk_len
                );

                // Wait until the announce window in this slot.
                tokio::time::sleep(std::time::Duration::from_millis(timing.announce_at_ms)).await;

                // Read the *next* key we intend to use for the following epoch.
                let (next_pk, next_epoch) = {
                    let k = ctx_clone.keys.lock().unwrap();
                    (k.next_pk.clone(), k.epoch.saturating_add(1))
                };

                // Submit announce_next_key once, signed with the local Aura authority
                // (the same identity that authors this block).
                match submit_announce_extrinsic::<B, C, Pool>(
                    client_clone.clone(),
                    pool_clone.clone(),
                    keystore_clone.clone(),
                    local_aura_pub.clone(),
                    next_pk.clone(),
                    next_epoch,
                    timing.announce_at_ms,
                    local_nonce,
                )
                .await
                {
                    Ok(()) => {
                        local_nonce = local_nonce.saturating_add(1);
                    }
                    Err(e) => {
                        let msg = format!("{e:?}");
                        // If the nonce is stale, bump once and retry.
                        if msg.contains("InvalidTransaction::Stale") || msg.contains("Stale") {
                            if submit_announce_extrinsic::<B, C, Pool>(
                                client_clone.clone(),
                                pool_clone.clone(),
                                keystore_clone.clone(),
                                local_aura_pub.clone(),
                                next_pk,
                                next_epoch,
                                timing.announce_at_ms,
                                local_nonce.saturating_add(1),
                            )
                            .await
                            .is_ok()
                            {
                                local_nonce = local_nonce.saturating_add(2);
                            } else {
                                log::warn!(
                                    target: "mev-shield",
                                    "announce_next_key retry failed after stale nonce: {e:?}"
                                );
                            }
                        } else {
                            log::warn!(
                                target: "mev-shield",
                                "announce_next_key submit error: {e:?}"
                            );
                        }
                    }
                }

                // Sleep the remainder of the slot (includes decrypt window).
                let tail = timing.slot_ms.saturating_sub(timing.announce_at_ms);
                tokio::time::sleep(std::time::Duration::from_millis(tail)).await;

                // Roll keys for the next slot / epoch.
                {
                    let mut k = ctx_clone.keys.lock().unwrap();
                    k.roll_for_next_slot();
                    log::info!(
                        target: "mev-shield",
                        "Rolled ML‑KEM key at slot boundary (local author): new epoch={}",
                        k.epoch
                    );
                }
            }
        }
    );

    ctx
}


/// Build & submit the signed `announce_next_key` extrinsic OFF-CHAIN,
/// using the local Aura authority key stored in the keystore.
pub async fn submit_announce_extrinsic<B, C, Pool>(
    client: std::sync::Arc<C>,
    pool:   std::sync::Arc<Pool>,
    keystore: sp_keystore::KeystorePtr,
    aura_pub: sp_core::sr25519::Public,   // local Aura authority public key
    next_public_key: Vec<u8>,            // full ML‑KEM pubkey bytes (expected 1184B)
    epoch: u64,
    at_ms: u64,
    nonce: u32,                          // nonce for CheckNonce extension
) -> anyhow::Result<()>
where
    B: sp_runtime::traits::Block,
    // Only need best/genesis from the client
    C: sc_client_api::HeaderBackend<B> + Send + Sync + 'static,
    Pool: sc_transaction_pool_api::TransactionPool<Block = B> + Send + Sync + 'static,
    // Convert to the pool's extrinsic type
    B::Extrinsic: From<sp_runtime::OpaqueExtrinsic>,
    // Allow generic conversion of block hash to bytes for H256
    B::Hash: AsRef<[u8]>,
{
    use node_subtensor_runtime as runtime;
    use runtime::{RuntimeCall, UncheckedExtrinsic, SignedPayload};

    use sc_transaction_pool_api::TransactionSource;
    use sp_core::H256;
    use sp_runtime::{
        AccountId32, MultiSignature, generic::Era, BoundedVec,
        traits::{ConstU32, TransactionExtension}
    };
    use sp_runtime::codec::Encode;

    // Helper: map a generic Block hash to H256 without requiring Into<H256>
    fn to_h256<H: AsRef<[u8]>>(h: H) -> H256 {
        let bytes = h.as_ref();
        let mut out = [0u8; 32];
        let n = bytes.len().min(32);
        out[32 - n..].copy_from_slice(&bytes[bytes.len() - n..]);
        H256(out)
    }

    // 0) Bounded public key (max 2 KiB) as required by the pallet.
    type MaxPk = ConstU32<2048>;
    let public_key: BoundedVec<u8, MaxPk> =
        BoundedVec::try_from(next_public_key)
            .map_err(|_| anyhow::anyhow!("public key too long (>2048 bytes)"))?;

    // 1) The runtime call carrying **full public key bytes**.
    let call = RuntimeCall::MevShield(
        pallet_mev_shield::Call::announce_next_key {
            public_key,
            epoch,
            at_ms,
        }
    );

    // 2) Extensions tuple (must match your runtime's `type TransactionExtensions`).
    type Extra = runtime::TransactionExtensions;
    let extra: Extra = (
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(Era::Immortal),
        // Use the passed-in nonce here:
        node_subtensor_runtime::check_nonce::CheckNonce::<runtime::Runtime>::from(nonce).into(),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        node_subtensor_runtime::transaction_payment_wrapper::ChargeTransactionPaymentWrapper::<runtime::Runtime>::new(
            pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0u64)
        ),
        pallet_subtensor::transaction_extension::SubtensorTransactionExtension::<runtime::Runtime>::new(),
        pallet_drand::drand_priority::DrandPriority::<runtime::Runtime>::new(),
        frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
    );

    // 3) Implicit (AdditionalSigned) values.
    type Implicit = <Extra as TransactionExtension<RuntimeCall>>::Implicit;

    let info          = client.info();
    let genesis_h256: H256 = to_h256(info.genesis_hash);

    let implicit: Implicit = (
        (),                                   // CheckNonZeroSender
        runtime::VERSION.spec_version,        // CheckSpecVersion
        runtime::VERSION.transaction_version, // CheckTxVersion
        genesis_h256,                         // CheckGenesis
        genesis_h256,                         // CheckEra (Immortal)
        (),                                   // CheckNonce (additional part)
        (),                                   // CheckWeight
        (),                                   // ChargeTransactionPaymentWrapper (additional part)
        (),                                   // SubtensorTransactionExtension (additional part)
        (),                                   // DrandPriority
        None,                                 // CheckMetadataHash (disabled)
    );

    // 4) Build the exact signable payload.
    let payload: SignedPayload = SignedPayload::from_raw(
        call.clone(),
        extra.clone(),
        implicit.clone(),
    );

    let raw_payload = payload.encode();

    // Sign with the local Aura key from the keystore (synchronous `Keystore` API).
    let sig_opt = keystore
        .sr25519_sign(AURA_KEY_TYPE, &aura_pub, &raw_payload)
        .map_err(|e| anyhow::anyhow!("keystore sr25519_sign error: {e:?}"))?;
    let sig = sig_opt
        .ok_or_else(|| anyhow::anyhow!("keystore sr25519_sign returned None for Aura key"))?;

    let signature: MultiSignature = sig.into();

    // 5) Assemble and submit (also log the extrinsic hash for observability).
    let who: AccountId32 = aura_pub.into();
    let address = sp_runtime::MultiAddress::Id(who);

    let uxt: UncheckedExtrinsic = UncheckedExtrinsic::new_signed(
        call,
        address,
        signature,
        extra,
    );

    let xt_bytes = uxt.encode();
    let xt_hash  = sp_core::hashing::blake2_256(&xt_bytes);

    let opaque: sp_runtime::OpaqueExtrinsic = uxt.into();
    let xt: <B as sp_runtime::traits::Block>::Extrinsic = opaque.into();

    pool.submit_one(info.best_hash, TransactionSource::Local, xt).await?;

    log::info!(
        target: "mev-shield",
        "announce_next_key submitted: xt=0x{}, epoch={}, nonce={}",
        hex::encode(xt_hash),
        epoch,
        nonce
    );

    Ok(())
}
