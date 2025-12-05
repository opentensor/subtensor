use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
use node_subtensor_runtime as runtime;
use rand::rngs::OsRng;
use sp_core::blake2_256;
use sp_runtime::KeyTypeId;
use std::sync::{Arc, Mutex};
use subtensor_macros::freeze_struct;
use tokio::time::sleep;

/// Parameters controlling time windows inside the slot.
#[freeze_struct("5c7ce101b36950de")]
#[derive(Clone)]
pub struct TimeParams {
    pub slot_ms: u64,
    pub announce_at_ms: u64,
    pub decrypt_window_ms: u64,
}

/// Holds the current/next ML‑KEM keypairs and their 32‑byte fingerprints.
#[freeze_struct("5e3c8209248282c3")]
#[derive(Clone)]
pub struct ShieldKeys {
    pub current_sk: Vec<u8>,  // ML‑KEM secret key bytes (encoded form)
    pub current_pk: Vec<u8>,  // ML‑KEM public  key bytes (encoded form)
    pub current_fp: [u8; 32], // blake2_256(pk)
    pub next_sk: Vec<u8>,
    pub next_pk: Vec<u8>,
    pub next_fp: [u8; 32],
}

impl ShieldKeys {
    pub fn new() -> Self {
        let (sk, pk) = MlKem768::generate(&mut OsRng);

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

        Self {
            current_sk,
            current_pk,
            current_fp,
            next_sk,
            next_pk,
            next_fp,
        }
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
    }
}

impl Default for ShieldKeys {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared context state.
#[freeze_struct("62af7d26cf7c1271")]
#[derive(Clone)]
pub struct ShieldContext {
    pub keys: Arc<Mutex<ShieldKeys>>,
    pub timing: TimeParams,
}

/// Derive AEAD key directly from the 32‑byte ML‑KEM shared secret.
pub fn derive_aead_key(ss: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let n = ss.len().min(32);

    if let (Some(dst), Some(src)) = (key.get_mut(..n), ss.get(..n)) {
        dst.copy_from_slice(src);
    }
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
    aead.decrypt(
        XNonce::from_slice(&nonce24),
        Payload {
            msg: ciphertext,
            aad,
        },
    )
    .ok()
}

const AURA_KEY_TYPE: KeyTypeId = KeyTypeId(*b"aura");

/// Start background tasks:
///  - per-slot ML‑KEM key rotation
///  - at ~announce_at_ms announce the next key bytes on chain (as an UNSIGNED tx),
pub fn spawn_author_tasks<B, C, Pool>(
    task_spawner: &sc_service::SpawnTaskHandle,
    client: Arc<C>,
    pool: Arc<Pool>,
    keystore: sp_keystore::KeystorePtr,
    timing: TimeParams,
) -> ShieldContext
where
    B: sp_runtime::traits::Block,
    C: sc_client_api::HeaderBackend<B> + sc_client_api::BlockchainEvents<B> + Send + Sync + 'static,
    Pool: sc_transaction_pool_api::TransactionPool<Block = B> + Send + Sync + 'static,
    B::Extrinsic: From<sp_runtime::OpaqueExtrinsic>,
{
    let ctx = ShieldContext {
        keys: Arc::new(Mutex::new(ShieldKeys::new())),
        timing: timing.clone(),
    };

    // Only run these tasks on nodes that actually have an Aura key in their keystore.
    let aura_keys: Vec<sp_core::sr25519::Public> = keystore.sr25519_public_keys(AURA_KEY_TYPE);
    if aura_keys.is_empty() {
        log::warn!(
            target: "mev-shield",
            "spawn_author_tasks: no local Aura sr25519 key in keystore; \
             this node will NOT announce MEV-Shield keys"
        );
        return ctx;
    }

    let ctx_clone = ctx.clone();
    let client_clone = client.clone();
    let pool_clone = pool.clone();

    // Slot tick / key-announce loop.
    task_spawner.spawn(
        "mev-shield-keys-and-announce",
        None,
        async move {
            use futures::StreamExt;
            use sp_consensus::BlockOrigin;

            let slot_ms = timing.slot_ms;

            // Clamp announce_at_ms so it never exceeds slot_ms.
            let mut announce_at_ms = timing.announce_at_ms;
            if announce_at_ms > slot_ms {
                log::warn!(
                    target: "mev-shield",
                    "spawn_author_tasks: announce_at_ms ({announce_at_ms}) > slot_ms ({slot_ms}); clamping to slot_ms",
                );
                announce_at_ms = slot_ms;
            }
            let tail_ms = slot_ms.saturating_sub(announce_at_ms);

            log::debug!(
                target: "mev-shield",
                "author timing: slot_ms={slot_ms} announce_at_ms={announce_at_ms} (effective) tail_ms={tail_ms}",
            );

            let mut import_stream = client_clone.import_notification_stream();

            while let Some(notif) = import_stream.next().await {
                // Only act on blocks that this node authored.
                if notif.origin != BlockOrigin::Own {
                    continue;
                }

                let (curr_pk_len, next_pk_len) = match ctx_clone.keys.lock() {
                    Ok(k) => (k.current_pk.len(), k.next_pk.len()),
                    Err(e) => {
                        log::debug!(
                            target: "mev-shield",
                            "spawn_author_tasks: failed to lock ShieldKeys (poisoned?): {e:?}",
                        );
                        continue;
                    }
                };

                log::debug!(
                    target: "mev-shield",
                    "Slot start (local author): (pk sizes: curr={curr_pk_len}B, next={next_pk_len}B)",
                );

                // Wait until the announce window in this slot.
                if announce_at_ms > 0 {
                    sleep(std::time::Duration::from_millis(announce_at_ms)).await;
                }

                // Read the next key we intend to use for the following block.
                let next_pk = match ctx_clone.keys.lock() {
                    Ok(k) => k.next_pk.clone(),
                    Err(e) => {
                        log::debug!(
                            target: "mev-shield",
                            "spawn_author_tasks: failed to lock ShieldKeys for next_pk: {e:?}",
                        );
                        continue;
                    }
                };

                // Submit announce_next_key as an UNSIGNED extrinsic (Origin::None).
                if let Err(e) = submit_announce_extrinsic::<B, C, Pool>(
                    client_clone.clone(),
                    pool_clone.clone(),
                    next_pk.clone(),
                )
                .await
                {
                    log::debug!(
                        target: "mev-shield",
                        "announce_next_key unsigned submit error: {e:?}"
                    );
                }

                // Sleep the remainder of the slot (if any).
                if tail_ms > 0 {
                    sleep(std::time::Duration::from_millis(tail_ms)).await;
                }

                // Roll keys for the next block.
                match ctx_clone.keys.lock() {
                    Ok(mut k) => {
                        k.roll_for_next_slot();
                        log::debug!(
                            target: "mev-shield",
                            "Rolled ML-KEM key at slot boundary",
                        );
                    }
                    Err(e) => {
                        log::debug!(
                            target: "mev-shield",
                            "spawn_author_tasks: failed to lock ShieldKeys for roll_for_next_slot: {e:?}",
                        );
                    }
                }
            }
        },
    );

    ctx
}

/// Build & submit the **unsigned** `announce_next_key` extrinsic OFF-CHAIN
pub async fn submit_announce_extrinsic<B, C, Pool>(
    client: Arc<C>,
    pool: Arc<Pool>,
    next_public_key: Vec<u8>,
) -> anyhow::Result<()>
where
    B: sp_runtime::traits::Block,
    C: sc_client_api::HeaderBackend<B> + Send + Sync + 'static,
    Pool: sc_transaction_pool_api::TransactionPool<Block = B> + Send + Sync + 'static,
    B::Extrinsic: From<sp_runtime::OpaqueExtrinsic>,
{
    use runtime::{RuntimeCall, UncheckedExtrinsic};
    use sc_transaction_pool_api::TransactionSource;
    use sp_runtime::codec::Encode;
    use sp_runtime::{BoundedVec, traits::ConstU32};

    type MaxPk = ConstU32<2048>;
    let public_key: BoundedVec<u8, MaxPk> = BoundedVec::try_from(next_public_key)
        .map_err(|_| anyhow::anyhow!("public key too long (>2048 bytes)"))?;

    // Runtime call carrying the public key bytes.
    let call = RuntimeCall::MevShield(pallet_shield::Call::announce_next_key { public_key });

    // Build UNSIGNED extrinsic (origin = None) using Frontier's `new_bare`.
    let uxt: UncheckedExtrinsic = UncheckedExtrinsic::new_bare(call);

    let xt_bytes = uxt.encode();
    let xt_hash = blake2_256(&xt_bytes);
    let xt_hash_hex = hex::encode(xt_hash);

    let opaque: sp_runtime::OpaqueExtrinsic = uxt.into();
    let xt: <B as sp_runtime::traits::Block>::Extrinsic = opaque.into();

    let best_hash = client.info().best_hash;
    pool.submit_one(best_hash, TransactionSource::Local, xt)
        .await?;

    log::debug!(
        target: "mev-shield",
        "announce_next_key (unsigned) submitted: xt=0x{xt_hash_hex}",
    );

    Ok(())
}
