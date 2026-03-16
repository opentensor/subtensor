use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use frame_system_rpc_runtime_api::AccountNonceApi;
use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
use node_subtensor_runtime as runtime;
use rand::rngs::OsRng;
use sp_api::ProvideRuntimeApi;
use sp_core::blake2_256;
use sp_runtime::{AccountId32, KeyTypeId};
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
#[freeze_struct("245b565abca7d403")]
#[derive(Clone)]
pub struct ShieldContext {
    pub keys: Arc<Mutex<ShieldKeys>>,
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
///  - at ~announce_at_ms announce the next key bytes on chain,
pub fn spawn_author_tasks<B, C, Pool>(
    task_spawner: &sc_service::SpawnTaskHandle,
    client: Arc<C>,
    pool: Arc<Pool>,
    keystore: sp_keystore::KeystorePtr,
    timing: TimeParams,
) -> ShieldContext
where
    B: sp_runtime::traits::Block,
    C: sc_client_api::HeaderBackend<B>
        + sc_client_api::BlockchainEvents<B>
        + ProvideRuntimeApi<B>
        + Send
        + Sync
        + 'static,
    C::Api: AccountNonceApi<B, AccountId32, u32>,
    Pool: sc_transaction_pool_api::TransactionPool<Block = B> + Send + Sync + 'static,
    B::Extrinsic: From<sp_runtime::OpaqueExtrinsic>,
{
    let ctx = ShieldContext {
        keys: Arc::new(Mutex::new(ShieldKeys::new())),
    };

    let aura_keys: Vec<sp_core::sr25519::Public> = keystore.sr25519_public_keys(AURA_KEY_TYPE);

    let local_aura_pub = match aura_keys.first().copied() {
        Some(k) => k,
        None => {
            log::warn!(
                target: "mev-shield",
                "spawn_author_tasks: no local Aura sr25519 key in keystore; \
                 this node will NOT announce MEV-Shield keys"
            );
            return ctx;
        }
    };

    let aura_account: AccountId32 = local_aura_pub.into();
    let ctx_clone = ctx.clone();
    let client_clone = client.clone();
    let pool_clone = pool.clone();
    let keystore_clone = keystore.clone();

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

                // 🔑 Fetch the current on-chain nonce for the Aura account using the best block hash.
                let best_hash = client_clone.info().best_hash;

                let nonce: u32 = match client_clone
                    .runtime_api()
                    .account_nonce(best_hash, aura_account.clone())
                {
                    Ok(n) => n,
                    Err(e) => {
                        log::debug!(
                            target: "mev-shield",
                            "spawn_author_tasks: failed to fetch account nonce for MEV-Shield author: {e:?}",
                        );
                        continue;
                    }
                };

                // Submit announce_next_key signed with the Aura key using the correct nonce.
                if let Err(e) = submit_announce_extrinsic::<B, C, Pool>(
                    client_clone.clone(),
                    pool_clone.clone(),
                    keystore_clone.clone(),
                    local_aura_pub,
                    next_pk.clone(),
                    nonce,
                )
                .await
                {
                    log::debug!(
                        target: "mev-shield",
                        "announce_next_key submit error (nonce={nonce:?}): {e:?}"
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

/// Build & submit the signed `announce_next_key` extrinsic OFF-CHAIN
pub async fn submit_announce_extrinsic<B, C, Pool>(
    client: Arc<C>,
    pool: Arc<Pool>,
    keystore: sp_keystore::KeystorePtr,
    aura_pub: sp_core::sr25519::Public,
    next_public_key: Vec<u8>,
    nonce: u32,
) -> anyhow::Result<()>
where
    B: sp_runtime::traits::Block,
    C: sc_client_api::HeaderBackend<B> + sp_api::ProvideRuntimeApi<B> + Send + Sync + 'static,
    C::Api: sp_api::Core<B>,
    Pool: sc_transaction_pool_api::TransactionPool<Block = B> + Send + Sync + 'static,
    B::Extrinsic: From<sp_runtime::OpaqueExtrinsic>,
    B::Hash: AsRef<[u8]>,
{
    use node_subtensor_runtime as runtime;
    use runtime::{RuntimeCall, SignedPayload, UncheckedExtrinsic};

    use sc_transaction_pool_api::TransactionSource;
    use sp_api::Core as _;
    use sp_core::H256;
    use sp_runtime::codec::Encode;
    use sp_runtime::{
        BoundedVec, MultiSignature,
        generic::Era,
        traits::{ConstU32, SaturatedConversion, TransactionExtension},
    };

    fn to_h256<H: AsRef<[u8]>>(h: H) -> H256 {
        let bytes = h.as_ref();
        let mut out = [0u8; 32];

        if bytes.is_empty() {
            return H256(out);
        }

        let n = bytes.len().min(32);
        let src_start = bytes.len().saturating_sub(n);
        let dst_start = 32usize.saturating_sub(n);

        let src_slice = bytes.get(src_start..).and_then(|s| s.get(..n));

        if let (Some(dst), Some(src)) = (out.get_mut(dst_start..32), src_slice) {
            dst.copy_from_slice(src);
            H256(out)
        } else {
            // Extremely defensive fallback.
            H256([0u8; 32])
        }
    }

    type MaxPk = ConstU32<2048>;
    let public_key: BoundedVec<u8, MaxPk> = BoundedVec::try_from(next_public_key)
        .map_err(|_| anyhow::anyhow!("public key too long (>2048 bytes)"))?;

    // 1) Runtime call carrying the public key bytes.
    let call = RuntimeCall::MevShield(pallet_shield::Call::announce_next_key { public_key });

    // 2) Build the transaction extensions exactly like the runtime.
    type Extra = runtime::TransactionExtensions;

    let info = client.info();
    let at_hash = info.best_hash;
    let at_hash_h256: H256 = to_h256(at_hash);
    let genesis_h256: H256 = to_h256(info.genesis_hash);

    const ERA_PERIOD: u64 = 12;
    let current_block: u64 = info.best_number.saturated_into();
    let era = Era::mortal(ERA_PERIOD, current_block);

    let extra: Extra = (
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(era),
        node_subtensor_runtime::check_nonce::CheckNonce::<runtime::Runtime>::from(nonce).into(),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        node_subtensor_runtime::transaction_payment_wrapper::ChargeTransactionPaymentWrapper::<
            runtime::Runtime,
        >::new(pallet_transaction_payment::ChargeTransactionPayment::<
            runtime::Runtime,
        >::from(0u64)),
        node_subtensor_runtime::sudo_wrapper::SudoTransactionExtension::<runtime::Runtime>::new(),
        pallet_subtensor::SubtensorTransactionExtension::<runtime::Runtime>::new(),
        pallet_drand::drand_priority::DrandPriority::<runtime::Runtime>::new(),
        frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
    );

    // 3) Manually construct the `Implicit` tuple that the runtime will also derive.
    type Implicit = <Extra as TransactionExtension<RuntimeCall>>::Implicit;

    // Try to get the *current* runtime version from on-chain WASM; if that fails,
    // fall back to the compiled runtime::VERSION.
    let (spec_version, tx_version) = match client.runtime_api().version(at_hash) {
        Ok(v) => (v.spec_version, v.transaction_version),
        Err(e) => {
            log::debug!(
                target: "mev-shield",
                "runtime_api::version failed at_hash={at_hash:?}: {e:?}; \
                 falling back to compiled runtime::VERSION",
            );
            (
                runtime::VERSION.spec_version,
                runtime::VERSION.transaction_version,
            )
        }
    };

    let implicit: Implicit = (
        (),           // CheckNonZeroSender
        spec_version, // dynamic or fallback spec_version
        tx_version,   // dynamic or fallback transaction_version
        genesis_h256, // CheckGenesis::Implicit = Hash
        at_hash_h256, // CheckEra::Implicit = hash of the block the tx is created at
        (),           // CheckNonce::Implicit = ()
        (),           // CheckWeight::Implicit = ()
        (),           // ChargeTransactionPaymentWrapper::Implicit = ()
        (),           // SudoTransactionExtension::Implicit = ()
        (),           // SubtensorTransactionExtension::Implicit = ()
        (),           // DrandPriority::Implicit = ()
        None,         // CheckMetadataHash::Implicit = Option<[u8; 32]>
    );

    // 4) Build the exact signable payload from call + extra + implicit.
    let payload: SignedPayload = SignedPayload::from_raw(call.clone(), extra.clone(), implicit);

    // 5) Sign with the local Aura key using the same SCALE bytes the runtime expects.
    let sig_opt = payload
        .using_encoded(|bytes| keystore.sr25519_sign(AURA_KEY_TYPE, &aura_pub, bytes))
        .map_err(|e| anyhow::anyhow!("keystore sr25519_sign error: {e:?}"))?;

    let sig = sig_opt
        .ok_or_else(|| anyhow::anyhow!("keystore sr25519_sign returned None for Aura key"))?;

    let signature: MultiSignature = sig.into();

    // 6) Sender address = AccountId32 derived from the Aura sr25519 public key.
    let who: AccountId32 = aura_pub.into();
    let address = sp_runtime::MultiAddress::Id(who);

    // 7) Assemble the signed extrinsic and submit it to the pool.
    let uxt: UncheckedExtrinsic = UncheckedExtrinsic::new_signed(call, address, signature, extra);

    let xt_bytes = uxt.encode();
    let xt_hash = sp_core::hashing::blake2_256(&xt_bytes);
    let xt_hash_hex = hex::encode(xt_hash);

    let opaque: sp_runtime::OpaqueExtrinsic = uxt.into();
    let xt: <B as sp_runtime::traits::Block>::Extrinsic = opaque.into();

    pool.submit_one(at_hash, TransactionSource::Local, xt)
        .await?;

    log::debug!(
        target: "mev-shield",
        "announce_next_key submitted: xt=0x{xt_hash_hex}, nonce={nonce:?}, \
         spec_version={spec_version}, tx_version={tx_version}, era={era:?}",
    );

    Ok(())
}
