use super::author::ShieldContext;
use futures::StreamExt;
use ml_kem::kem::{Decapsulate, DecapsulationKey};
use ml_kem::{Ciphertext, Encoded, EncodedSizeUser, MlKem768, MlKem768Params};
use sc_service::SpawnTaskHandle;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_core::{H256, sr25519};
use sp_runtime::traits::{Header, SaturatedConversion};
use sp_runtime::{AccountId32, MultiSignature, OpaqueExtrinsic};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;

const KEY_FP_LEN: usize = 32;

/// Buffer of wrappers keyed by the block number in which they were included.
#[derive(Default, Clone)]
struct WrapperBuffer {
    by_id: HashMap<
        H256,
        (
            Vec<u8>,     // ciphertext blob
            u64,         // originating block number
            AccountId32, // wrapper author
        ),
    >,
}

impl WrapperBuffer {
    fn upsert(&mut self, id: H256, block_number: u64, author: AccountId32, ciphertext: Vec<u8>) {
        self.by_id.insert(id, (ciphertext, block_number, author));
    }

    /// Drain only wrappers whose `block_number` matches the given `block`.
    ///   - Wrappers with `block_number > block` are kept for future decrypt windows.
    ///   - Wrappers with `block_number < block` are considered stale and dropped.
    fn drain_for_block(
        &mut self,
        block: u64,
    ) -> Vec<(H256, u64, sp_runtime::AccountId32, Vec<u8>)> {
        let mut ready = Vec::new();
        let mut kept_future: usize = 0;
        let mut dropped_past: usize = 0;

        self.by_id.retain(|id, (ct, block_number, who)| {
            if *block_number == block {
                // Ready to process now; remove from buffer.
                ready.push((*id, *block_number, who.clone(), ct.clone()));
                false
            } else if *block_number > block {
                // Not yet reveal time; keep for future blocks.
                kept_future = kept_future.saturating_add(1);
                true
            } else {
                // block_number < block => stale / missed reveal window; drop.
                dropped_past = dropped_past.saturating_add(1);
                log::debug!(
                    target: "mev-shield",
                    "revealer: dropping stale wrapper id=0x{} block_number={} < block={}",
                    hex::encode(id.as_bytes()),
                    *block_number,
                    block
                );
                false
            }
        });

        log::debug!(
            target: "mev-shield",
            "revealer: drain_for_block(block={}): ready={}, kept_future={}, dropped_past={}",
            block,
            ready.len(),
            kept_future,
            dropped_past
        );

        ready
    }
}

/// Start a background worker that:
///   • watches imported blocks and captures `MevShield::submit_encrypted`
///   • buffers those wrappers per originating block,
///   • during the last `decrypt_window_ms` of the slot: decrypt & submit unsigned `execute_revealed`
pub fn spawn_revealer<B, C, Pool>(
    task_spawner: &SpawnTaskHandle,
    client: Arc<C>,
    pool: Arc<Pool>,
    ctx: ShieldContext,
) where
    B: sp_runtime::traits::Block<Extrinsic = OpaqueExtrinsic>,
    C: sc_client_api::HeaderBackend<B>
        + sc_client_api::BlockchainEvents<B>
        + sc_client_api::BlockBackend<B>
        + Send
        + Sync
        + 'static,
    Pool: TransactionPool<Block = B> + Send + Sync + 'static,
{
    use codec::{Decode, Encode};

    type Address = sp_runtime::MultiAddress<sp_runtime::AccountId32, ()>;
    type RUnchecked = node_subtensor_runtime::UncheckedExtrinsic;

    let buffer: Arc<Mutex<WrapperBuffer>> = Arc::new(Mutex::new(WrapperBuffer::default()));

    // ── 1) buffer wrappers ───────────────────────────────────────
    {
        let client = Arc::clone(&client);
        let buffer = Arc::clone(&buffer);

        task_spawner.spawn(
            "mev-shield-buffer-wrappers",
            None,
            async move {
                log::debug!(target: "mev-shield", "buffer-wrappers task started");
                let mut import_stream = client.import_notification_stream();

                while let Some(notif) = import_stream.next().await {
                    let at_hash = notif.hash;
                    let block_number_u64: u64 = (*notif.header.number()).saturated_into();

                    log::debug!(
                        target: "mev-shield",
                        "imported block hash={:?} number={} origin={:?}",
                        at_hash,
                        block_number_u64,
                        notif.origin
                    );

                    match client.block_body(at_hash) {
                        Ok(Some(body)) => {
                            log::debug!(
                                target: "mev-shield",
                                "  block has {} extrinsics",
                                body.len()
                            );

                            for (idx, opaque_xt) in body.into_iter().enumerate() {
                                let encoded = opaque_xt.encode();
                                log::debug!(
                                    target: "mev-shield",
                                    "    [xt #{idx}] opaque len={} bytes",
                                    encoded.len()
                                );

                                let uxt: RUnchecked = match RUnchecked::decode(&mut &encoded[..]) {
                                    Ok(u) => u,
                                    Err(e) => {
                                        log::debug!(
                                            target: "mev-shield",
                                            "    [xt #{idx}] failed to decode UncheckedExtrinsic: {e:?}",
                                        );
                                        continue;
                                    }
                                };

                                log::debug!(
                                    target: "mev-shield",
                                    "    [xt #{idx}] decoded call: {:?}",
                                    &uxt.0.function
                                );

                                let author_opt: Option<sp_runtime::AccountId32> =
                                    match &uxt.0.preamble {
                                        sp_runtime::generic::Preamble::Signed(
                                            addr,
                                            _sig,
                                            _ext,
                                        ) => match addr.clone() {
                                            Address::Id(acc) => Some(acc),
                                            Address::Address32(bytes) => {
                                                Some(sp_runtime::AccountId32::new(bytes))
                                            }
                                            _ => None,
                                        },
                                        _ => None,
                                    };

                                let Some(author) = author_opt else {
                                    log::debug!(
                                        target: "mev-shield",
                                        "    [xt #{idx}] not a Signed(AccountId32) extrinsic; skipping"
                                    );
                                    continue;
                                };

                                if let node_subtensor_runtime::RuntimeCall::MevShield(
                                    pallet_shield::Call::submit_encrypted {
                                        commitment,
                                        ciphertext,
                                    },
                                ) = &uxt.0.function
                                {
                                    let payload =
                                        (author.clone(), *commitment, ciphertext).encode();
                                    let id = H256(sp_core::hashing::blake2_256(&payload));

                                    log::debug!(
                                        target: "mev-shield",
                                        "    [xt #{idx}] buffered submit_encrypted: id=0x{}, block_number={}, author={}, ct_len={}, commitment={:?}",
                                        hex::encode(id.as_bytes()),
                                        block_number_u64,
                                        author,
                                        ciphertext.len(),
                                        commitment
                                    );

                                    if let Ok(mut buf) = buffer.lock() {
                                        buf.upsert(
                                            id,
                                            block_number_u64,
                                            author,
                                            ciphertext.to_vec(),
                                        );
                                    } else {
                                        log::debug!(
                                            target: "mev-shield",
                                            "    [xt #{idx}] failed to lock WrapperBuffer; dropping wrapper"
                                        );
                                    }
                                }
                            }
                        }
                        Ok(None) => log::debug!(
                            target: "mev-shield",
                            "  block_body returned None for hash={at_hash:?}",
                        ),
                        Err(e) => log::debug!(
                            target: "mev-shield",
                            "  block_body error for hash={at_hash:?}: {e:?}",
                        ),
                    }
                }
            },
        );
    }

    // ── 2) decrypt window revealer ──────────────────────────────
    {
        let client = Arc::clone(&client);
        let pool = Arc::clone(&pool);
        let buffer = Arc::clone(&buffer);
        let ctx = ctx.clone();

        task_spawner.spawn(
            "mev-shield-last-window-revealer",
            None,
            async move {
                log::debug!(target: "mev-shield", "last-window-revealer task started");

                // Respect the configured slot_ms, but clamp the decrypt window so it never
                // exceeds the slot length (important for fast runtimes).
                let slot_ms = ctx.timing.slot_ms;
                let mut decrypt_window_ms = ctx.timing.decrypt_window_ms;

                if decrypt_window_ms > slot_ms {
                    log::warn!(
                        target: "mev-shield",
                        "spawn_revealer: decrypt_window_ms ({decrypt_window_ms}) > slot_ms ({slot_ms}); clamping to slot_ms",
                    );
                    decrypt_window_ms = slot_ms;
                }

                let tail_ms = slot_ms.saturating_sub(decrypt_window_ms);

                log::debug!(
                    target: "mev-shield",
                    "revealer timing: slot_ms={slot_ms} decrypt_window_ms={decrypt_window_ms} (effective) tail_ms={tail_ms}",
                );

                loop {
                    log::debug!(
                        target: "mev-shield",
                        "revealer: sleeping {tail_ms} ms before decrypt window (slot_ms={slot_ms}, decrypt_window_ms={decrypt_window_ms})",
                    );

                    if tail_ms > 0 {
                        sleep(Duration::from_millis(tail_ms)).await;
                    }

                    // Snapshot the current ML‑KEM secret.
                    let snapshot_opt = match ctx.keys.lock() {
                        Ok(k) => {
                            let sk_hash = sp_core::hashing::blake2_256(&k.current_sk);
                            Some((
                                k.current_sk.clone(),
                                k.current_pk.len(),
                                k.next_pk.len(),
                                sk_hash,
                            ))
                        }
                        Err(e) => {
                            log::debug!(
                                target: "mev-shield",
                                "revealer: failed to lock ShieldKeys (poisoned?): {e:?}",
                            );
                            None
                        }
                    };

                    let (curr_sk_bytes, curr_pk_len, next_pk_len, sk_hash) =
                        match snapshot_opt {
                            Some(v) => v,
                            None => {
                                // Skip this decrypt window entirely, without holding any guard.
                                if decrypt_window_ms > 0 {
                                    sleep(Duration::from_millis(decrypt_window_ms)).await;
                                }
                                continue;
                            }
                        };

                    // Use best block number as the block whose submissions we reveal now.
                    let curr_block: u64 = client.info().best_number.saturated_into();

                    log::debug!(
                        target: "mev-shield",
                        "revealer: decrypt window start. reveal_block={} sk_len={} sk_hash=0x{} curr_pk_len={} next_pk_len={}",
                        curr_block,
                        curr_sk_bytes.len(),
                        hex::encode(sk_hash),
                        curr_pk_len,
                        next_pk_len
                    );

                    // Only process wrappers whose originating block matches the reveal_block.
                    let drained: Vec<(H256, u64, sp_runtime::AccountId32, Vec<u8>)> =
                        match buffer.lock() {
                            Ok(mut buf) => buf.drain_for_block(curr_block),
                            Err(e) => {
                                log::debug!(
                                    target: "mev-shield",
                                    "revealer: failed to lock WrapperBuffer for drain_for_block: {e:?}",
                                );
                                Vec::new()
                            }
                        };

                    log::debug!(
                        target: "mev-shield",
                        "revealer: drained {} buffered wrappers for reveal_block={}",
                        drained.len(),
                        curr_block
                    );

                    let mut to_submit: Vec<(H256, node_subtensor_runtime::RuntimeCall)> =
                        Vec::new();
                    let mut failed_calls: Vec<(H256, node_subtensor_runtime::RuntimeCall)> =
                        Vec::new();

                    // Helper to create mark_decryption_failed call
                    let create_failed_call = |id: H256, reason: &str| -> node_subtensor_runtime::RuntimeCall {
                        use sp_runtime::BoundedVec;
                        let reason_bytes = reason.as_bytes();
                        let reason_bounded = BoundedVec::try_from(reason_bytes.to_vec())
                            .unwrap_or_else(|_| { // Fallback if the reason is too long
                                BoundedVec::try_from(b"Decryption failed".to_vec()).unwrap_or_default()
                            });

                        node_subtensor_runtime::RuntimeCall::MevShield(
                            pallet_shield::Call::mark_decryption_failed {
                                id,
                                reason: reason_bounded,
                            },
                        )
                    };

                    for (id, block_number, author, blob) in drained.into_iter() {
                        log::debug!(
                            target: "mev-shield",
                            "revealer: candidate id=0x{} submitted_in={} (reveal_block={}) author={} blob_len={}",
                            hex::encode(id.as_bytes()),
                            block_number,
                            curr_block,
                            author,
                            blob.len()
                        );

                        // Safely parse blob: [u16 kem_len][kem_ct][nonce24][aead_ct]
                        if blob.len() < 2 {
                            let error_message = "blob too short to contain kem_len";
                            log::debug!(
                                target: "mev-shield",
                                "  id=0x{}: {}",
                                hex::encode(id.as_bytes()),
                                error_message
                            );
                            failed_calls.push((
                                id,
                                create_failed_call(id, error_message),
                            ));
                            continue;
                        }

                        let mut cursor: usize = 0;

                        // 1) kem_len (u16 LE)
                        let kem_len_end = match cursor.checked_add(2usize) {
                            Some(e) => e,
                            None => {
                                let error_message = "kem_len range overflow";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let kem_len_slice = match blob.get(cursor..kem_len_end) {
                            Some(s) => s,
                            None => {
                                let error_message = "blob too short for kem_len bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (cursor={} end={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    cursor,
                                    kem_len_end
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let kem_len_bytes: [u8; 2] = match kem_len_slice.try_into() {
                            Ok(arr) => arr,
                            Err(_) => {
                                let error_message = "kem_len slice not 2 bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let kem_len = u16::from_le_bytes(kem_len_bytes) as usize;
                        cursor = kem_len_end;

                        // 2) KEM ciphertext
                        let kem_ct_end = match cursor.checked_add(kem_len) {
                            Some(e) => e,
                            None => {
                                let error_message = "kem_ct range overflow";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (cursor={} kem_len={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    cursor,
                                    kem_len
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let kem_ct_bytes = match blob.get(cursor..kem_ct_end) {
                            Some(s) => s,
                            None => {
                                let error_message = "blob too short for kem_ct";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (cursor={} end={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    cursor,
                                    kem_ct_end
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };
                        cursor = kem_ct_end;

                        // 3) Nonce (24 bytes)
                        const NONCE_LEN: usize = 24;
                        let nonce_end = match cursor.checked_add(NONCE_LEN) {
                            Some(e) => e,
                            None => {
                                let error_message = "nonce range overflow";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (cursor={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    cursor
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let nonce_bytes = match blob.get(cursor..nonce_end) {
                            Some(s) => s,
                            None => {
                                let error_message = "blob too short for nonce24";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (cursor={} end={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    cursor,
                                    nonce_end
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };
                        cursor = nonce_end;

                        // 4) AEAD body (rest)
                        let aead_body = match blob.get(cursor..) {
                            Some(s) => s,
                            None => {
                                let error_message = "blob too short for aead_body";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (cursor={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    cursor
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let kem_ct_hash = sp_core::hashing::blake2_256(kem_ct_bytes);
                        let aead_body_hash = sp_core::hashing::blake2_256(aead_body);

                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: kem_len={} kem_ct_hash=0x{} nonce=0x{} aead_body_len={} aead_body_hash=0x{}",
                            hex::encode(id.as_bytes()),
                            kem_len,
                            hex::encode(kem_ct_hash),
                            hex::encode(nonce_bytes),
                            aead_body.len(),
                            hex::encode(aead_body_hash),
                        );

                        // Rebuild DecapsulationKey and decapsulate.
                        let enc_sk =
                            match Encoded::<DecapsulationKey<MlKem768Params>>::try_from(
                                &curr_sk_bytes[..],
                            ) {
                                Ok(e) => e,
                                Err(e) => {
                                    let error_message = "DecapsulationKey::try_from failed";
                                    log::debug!(
                                        target: "mev-shield",
                                        "  id=0x{}: {} (len={}, err={:?})",
                                        hex::encode(id.as_bytes()),
                                        error_message,
                                        curr_sk_bytes.len(),
                                        e
                                    );
                                    failed_calls.push((
                                        id,
                                        create_failed_call(id, error_message),
                                    ));
                                    continue;
                                }
                            };
                        let sk = DecapsulationKey::<MlKem768Params>::from_bytes(&enc_sk);

                        let ct = match Ciphertext::<MlKem768>::try_from(kem_ct_bytes) {
                            Ok(c) => c,
                            Err(e) => {
                                let error_message = "Ciphertext::try_from failed";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}: {:?}",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    e
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let ss = match sk.decapsulate(&ct) {
                            Ok(s) => s,
                            Err(_) => {
                                let error_message = "ML-KEM decapsulate failed";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let ss_bytes: &[u8] = ss.as_ref();
                        if ss_bytes.len() != 32 {
                            let error_message = "shared secret length != 32";
                            log::debug!(
                                target: "mev-shield",
                                "  id=0x{}: {} (len={})",
                                hex::encode(id.as_bytes()),
                                error_message,
                                ss_bytes.len()
                            );
                            failed_calls.push((
                                id,
                                create_failed_call(id, error_message),
                            ));
                            continue;
                        }
                        let mut ss32 = [0u8; 32];
                        ss32.copy_from_slice(ss_bytes);

                        let ss_hash = sp_core::hashing::blake2_256(&ss32);
                        let aead_key =
                            crate::mev_shield::author::derive_aead_key(&ss32);
                        let key_hash_dbg = sp_core::hashing::blake2_256(&aead_key);

                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: decapsulated shared_secret_len=32 shared_secret_hash=0x{}",
                            hex::encode(id.as_bytes()),
                            hex::encode(ss_hash)
                        );
                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: derived AEAD key hash=0x{} (direct-from-ss)",
                            hex::encode(id.as_bytes()),
                            hex::encode(key_hash_dbg)
                        );

                        let mut nonce24 = [0u8; 24];
                        nonce24.copy_from_slice(nonce_bytes);

                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: attempting AEAD decrypt nonce=0x{} ct_len={}",
                            hex::encode(id.as_bytes()),
                            hex::encode(nonce24),
                            aead_body.len()
                        );

                        let plaintext = match crate::mev_shield::author::aead_decrypt(
                            aead_key,
                            nonce24,
                            aead_body,
                            &[],
                        ) {
                            Some(pt) => pt,
                            None => {
                                let error_message = "AEAD decrypt failed";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}; ct_hash=0x{}",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    hex::encode(aead_body_hash),
                                );
                                continue;
                            }
                        };

                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: AEAD decrypt OK, plaintext_len={}",
                            hex::encode(id.as_bytes()),
                            plaintext.len()
                        );

                        // Safely parse plaintext layout without panics.
                        //
                        // Layout:
                        //   signer   (32)
                        //   key_hash (32)  == Hashing::hash(NextKey_bytes) at submit time
                        //   call     (..)
                        //   sig_kind (1)
                        //   sig      (64)
                        let min_plain_len: usize = 32usize
                            .saturating_add(KEY_FP_LEN)
                            .saturating_add(1usize)
                            .saturating_add(64usize);

                        if plaintext.len() < min_plain_len {
                            let error_message = "plaintext too short";
                            log::debug!(
                                target: "mev-shield",
                                "  id=0x{}: {} (len={}, min={})",
                                hex::encode(id.as_bytes()),
                                error_message,
                                plaintext.len(),
                                min_plain_len
                            );
                            failed_calls.push((
                                id,
                                create_failed_call(id, error_message),
                            ));
                            continue;
                        }

                        let signer_raw = match plaintext.get(0..32) {
                            Some(s) => s,
                            None => {
                                let error_message = "missing signer bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let key_hash_raw = match plaintext.get(32..32usize.saturating_add(KEY_FP_LEN))
                        {
                            Some(s) if s.len() == KEY_FP_LEN => s,
                            _ => {
                                let error_message = "missing or malformed key_hash bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        // sig_off = len - 65 (sig_kind + 64-byte sig)
                        let sig_min_offset: usize =
                            32usize.saturating_add(KEY_FP_LEN);

                        let sig_off = match plaintext.len().checked_sub(65usize) {
                            Some(off) if off >= sig_min_offset => off,
                            _ => {
                                let error_message = "invalid plaintext length for signature split";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (len={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    plaintext.len()
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let call_start: usize = sig_min_offset;
                        let call_bytes = match plaintext.get(call_start..sig_off) {
                            Some(s) if !s.is_empty() => s,
                            _ => {
                                let error_message = "missing call bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let sig_kind = match plaintext.get(sig_off) {
                            Some(b) => *b,
                            None => {
                                let error_message = "missing signature kind byte";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let sig_bytes_start = sig_off.saturating_add(1usize);
                        let sig_bytes = match plaintext.get(sig_bytes_start..) {
                            Some(s) if s.len() == 64 => s,
                            _ => {
                                let error_message = "signature bytes not 64 bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };

                        let signer_array: [u8; 32] = match signer_raw.try_into() {
                            Ok(a) => a,
                            Err(_) => {
                                let error_message = "signer_raw not 32 bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            }
                        };
                        let signer = sp_runtime::AccountId32::new(signer_array);

                        let mut fp_array = [0u8; KEY_FP_LEN];
                        fp_array.copy_from_slice(key_hash_raw);
                        let key_hash_h256 = H256(fp_array);

                        let inner_call: node_subtensor_runtime::RuntimeCall =
                            match Decode::decode(&mut &call_bytes[..]) {
                                Ok(c) => c,
                                Err(e) => {
                                    let error_message = "failed to decode RuntimeCall";
                                    log::debug!(
                                        target: "mev-shield",
                                        "  id=0x{}: {} (len={}): {:?}",
                                        hex::encode(id.as_bytes()),
                                        error_message,
                                        call_bytes.len(),
                                        e
                                    );
                                    failed_calls.push((
                                        id,
                                        create_failed_call(id, error_message),
                                    ));
                                    continue;
                                }
                            };

                        let signature: MultiSignature =
                            if sig_kind == 0x01 {
                                let mut raw_sig = [0u8; 64];
                                raw_sig.copy_from_slice(sig_bytes);
                                MultiSignature::from(sr25519::Signature::from_raw(raw_sig))
                            } else {
                                let error_message = "unsupported signature format";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {} (kind=0x{:02x}, len={})",
                                    hex::encode(id.as_bytes()),
                                    error_message,
                                    sig_kind,
                                    sig_bytes.len()
                                );
                                failed_calls.push((
                                    id,
                                    create_failed_call(id, error_message),
                                ));
                                continue;
                            };

                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: decrypted wrapper: signer={}, key_hash=0x{}, call={:?}",
                            hex::encode(id.as_bytes()),
                            signer,
                            hex::encode(key_hash_h256.as_bytes()),
                            inner_call
                        );

                        let reveal = node_subtensor_runtime::RuntimeCall::MevShield(
                            pallet_shield::Call::execute_revealed {
                                id,
                                signer: signer.clone(),
                                key_hash: key_hash_h256.into(),
                                call: Box::new(inner_call),
                                signature,
                            },
                        );

                        to_submit.push((id, reveal));
                    }

                    // Submit locally.
                    let at = client.info().best_hash;
                    log::debug!(
                        target: "mev-shield",
                        "revealer: submitting {} execute_revealed calls at best_hash={:?}",
                        to_submit.len(),
                        at
                    );

                    for (id, call) in to_submit.into_iter() {
                        let uxt: node_subtensor_runtime::UncheckedExtrinsic =
                            node_subtensor_runtime::UncheckedExtrinsic::new_bare(call);
                        let xt_bytes = uxt.encode();

                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: encoded UncheckedExtrinsic len={}",
                            hex::encode(id.as_bytes()),
                            xt_bytes.len()
                        );

                        match OpaqueExtrinsic::from_bytes(&xt_bytes) {
                            Ok(opaque) => {
                                match pool
                                    .submit_one(at, TransactionSource::Local, opaque)
                                    .await
                                {
                                    Ok(_) => {
                                        let xt_hash =
                                            sp_core::hashing::blake2_256(&xt_bytes);
                                        log::debug!(
                                            target: "mev-shield",
                                            "  id=0x{}: submit_one(execute_revealed) OK, xt_hash=0x{}",
                                            hex::encode(id.as_bytes()),
                                            hex::encode(xt_hash)
                                        );
                                    }
                                    Err(e) => {
                                        log::debug!(
                                            target: "mev-shield",
                                            "  id=0x{}: submit_one(execute_revealed) FAILED: {:?}",
                                            hex::encode(id.as_bytes()),
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: OpaqueExtrinsic::from_bytes failed: {:?}",
                                    hex::encode(id.as_bytes()),
                                    e
                                );
                            }
                        }
                    }

                    // Submit failed decryption calls
                    if !failed_calls.is_empty() {
                        log::debug!(
                            target: "mev-shield",
                            "revealer: submitting {} mark_decryption_failed calls at best_hash={:?}",
                            failed_calls.len(),
                            at
                        );

                        for (id, call) in failed_calls.into_iter() {
                            let uxt: node_subtensor_runtime::UncheckedExtrinsic =
                                node_subtensor_runtime::UncheckedExtrinsic::new_bare(call);
                            let xt_bytes = uxt.encode();

                            log::debug!(
                                target: "mev-shield",
                                "  id=0x{}: encoded mark_decryption_failed UncheckedExtrinsic len={}",
                                hex::encode(id.as_bytes()),
                                xt_bytes.len()
                            );

                            match OpaqueExtrinsic::from_bytes(&xt_bytes) {
                                Ok(opaque) => {
                                    match pool
                                        .submit_one(at, TransactionSource::Local, opaque)
                                        .await
                                    {
                                        Ok(_) => {
                                            let xt_hash =
                                                sp_core::hashing::blake2_256(&xt_bytes);
                                            log::debug!(
                                                target: "mev-shield",
                                                "  id=0x{}: submit_one(mark_decryption_failed) OK, xt_hash=0x{}",
                                                hex::encode(id.as_bytes()),
                                                hex::encode(xt_hash)
                                            );
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                target: "mev-shield",
                                                "  id=0x{}: submit_one(mark_decryption_failed) FAILED: {:?}",
                                                hex::encode(id.as_bytes()),
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        target: "mev-shield",
                                        "  id=0x{}: OpaqueExtrinsic::from_bytes(mark_decryption_failed) failed: {:?}",
                                        hex::encode(id.as_bytes()),
                                        e
                                    );
                                }
                            }
                        }
                    }

                    // Let the decrypt window elapse.
                    if decrypt_window_ms > 0 {
                        sleep(Duration::from_millis(decrypt_window_ms)).await;
                    }
                }
            },
        );
    }
}
