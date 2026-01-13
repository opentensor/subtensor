use super::author::ShieldContext;
use futures::StreamExt;
use ml_kem::kem::{Decapsulate, DecapsulationKey};
use ml_kem::{Ciphertext, Encoded, EncodedSizeUser, MlKem768, MlKem768Params};
use sc_service::SpawnTaskHandle;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_consensus::BlockOrigin;
use sp_core::H256;
use sp_runtime::traits::{Header, SaturatedConversion};
use sp_runtime::{AccountId32, OpaqueExtrinsic};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Truncate a UTF-8 string to at most `max_bytes` bytes without splitting codepoints.
fn truncate_utf8_to_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let mut end = max_bytes.min(s.len());

    // Decrement until we find a valid UTF-8 boundary.
    while end > 0 {
        if let Some(prefix) = s.get(..end) {
            return prefix.to_string();
        }
        end = end.saturating_sub(1);
    }

    // If max_bytes was 0 or we couldn't find a boundary (extremely defensive), return empty.
    String::new()
}

/// Helper to build a `mark_decryption_failed` runtime call with a bounded reason string.
fn create_failed_call(id: H256, reason: &str) -> node_subtensor_runtime::RuntimeCall {
    use sp_runtime::BoundedVec;

    let reason_bytes = reason.as_bytes();
    let reason_bounded = BoundedVec::try_from(reason_bytes.to_vec()).unwrap_or_else(|_| {
        // Fallback if the reason is too long for the bounded vector.
        BoundedVec::try_from(b"Decryption failed".to_vec()).unwrap_or_default()
    });

    node_subtensor_runtime::RuntimeCall::MevShield(pallet_shield::Call::mark_decryption_failed {
        id,
        reason: reason_bounded,
    })
}

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
    ///   - Wrappers with `block_number > block` are kept for future decrypt passes.
    ///   - Wrappers with `block_number < block` are considered stale and dropped, and
    ///     we emit `mark_decryption_failed` calls for them so they are visible on-chain.
    fn drain_for_block(
        &mut self,
        block: u64,
        failed_calls: &mut Vec<(H256, node_subtensor_runtime::RuntimeCall)>,
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
                // block_number < block => stale / missed decrypt opportunity; drop and mark failed.
                dropped_past = dropped_past.saturating_add(1);
                log::debug!(
                    target: "mev-shield",
                    "revealer: dropping stale wrapper id=0x{} block_number={} < block={}",
                    hex::encode(id.as_bytes()),
                    *block_number,
                    block
                );

                // Mark decryption failed on-chain so clients can observe the missed wrapper.
                failed_calls.push((
                    *id,
                    create_failed_call(
                        *id,
                        "missed decrypt window (wrapper submitted in an earlier block)",
                    ),
                ));

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
///   • on each **locally authored** block: decrypt & submit wrappers for that block.
///
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

    {
        let client = Arc::clone(&client);
        let pool = Arc::clone(&pool);
        let buffer = Arc::clone(&buffer);
        let ctx = ctx.clone();

        task_spawner.spawn(
            "mev-shield-block-revealer",
            None,
            async move {
                log::debug!(target: "mev-shield", "Revealer task started");
                let mut import_stream = client.import_notification_stream();

                while let Some(notif) = import_stream.next().await {
                    if notif.origin != BlockOrigin::Own {
                        continue;
                    }

                    let at_hash = notif.hash;
                    let block_number_u64: u64 = (*notif.header.number()).saturated_into();

                    log::debug!(
                        target: "mev-shield",
                        "imported block hash={:?} number={} origin={:?}",
                        at_hash,
                        block_number_u64,
                        notif.origin
                    );

                    // ── 1) buffer wrappers from this (locally authored) block ───────────
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

                    // ── 2) snapshot current ML‑KEM secret for this block ────────────────
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

                    let (curr_sk_bytes, curr_pk_len, next_pk_len, sk_hash) = match snapshot_opt {
                        Some(v) => v,
                        None => {
                            log::debug!(
                                target: "mev-shield",
                                "revealer: Cannot snapshot key for this block",
                            );
                            continue;
                        }
                    };

                    // Use this block as the reveal block.
                    let curr_block: u64 = block_number_u64;

                    log::debug!(
                        target: "mev-shield",
                        "revealer: decrypt for block {}. sk_len={} sk_hash=0x{} curr_pk_len={} next_pk_len={}",
                        curr_block,
                        curr_sk_bytes.len(),
                        hex::encode(sk_hash),
                        curr_pk_len,
                        next_pk_len
                    );

                    // ── 3) drain & decrypt wrappers for this block ─────────────────────
                    let mut to_submit: Vec<(H256, node_subtensor_runtime::UncheckedExtrinsic)> =
                        Vec::new();
                    let mut failed_calls: Vec<(H256, node_subtensor_runtime::RuntimeCall)> =
                        Vec::new();

                    // Only process wrappers whose originating block matches this block.
                    let drained: Vec<(H256, u64, sp_runtime::AccountId32, Vec<u8>)> =
                        match buffer.lock() {
                            Ok(mut buf) => buf.drain_for_block(curr_block, &mut failed_calls),
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
                        "revealer: drained {} buffered wrappers for block={}",
                        drained.len(),
                        curr_block
                    );

                    for (id, block_number, author, blob) in drained.into_iter() {
                        log::debug!(
                            target: "mev-shield",
                            "revealer: candidate id=0x{} submitted_in={} (block={}) author={} blob_len={}",
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
                            failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                    failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
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
                            failed_calls.push((id, create_failed_call(id, error_message)));
                            continue;
                        }
                        let mut ss32 = [0u8; 32];
                        ss32.copy_from_slice(ss_bytes);

                        let ss_hash = sp_core::hashing::blake2_256(&ss32);
                        let aead_key = crate::mev_shield::author::derive_aead_key(&ss32);
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
                                failed_calls.push((id, create_failed_call(id, error_message)));
                                continue;
                            }
                        };

                        log::debug!(
                            target: "mev-shield",
                            "  id=0x{}: AEAD decrypt OK, plaintext_len={}",
                            hex::encode(id.as_bytes()),
                            plaintext.len()
                        );

                        if plaintext.is_empty() {
                            let error_message = "plaintext too short";
                            log::debug!(
                                target: "mev-shield",
                                "  id=0x{}: {} (len={}, min={})",
                                hex::encode(id.as_bytes()),
                                error_message,
                                plaintext.len(),
                                1
                            );
                            failed_calls.push((id, create_failed_call(id, error_message)));
                            continue;
                        }

                        let signed_extrinsic_bytes = match plaintext.get(0..plaintext.len()) {
                            Some(s) if !s.is_empty() => s,
                            _ => {
                                let error_message = "missing signed extrinsic bytes";
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: {}",
                                    hex::encode(id.as_bytes()),
                                    error_message
                                );
                                failed_calls.push((id, create_failed_call(id, error_message)));
                                continue;
                            }
                        };

                        let signed_extrinsic: node_subtensor_runtime::UncheckedExtrinsic =
                            match Decode::decode(&mut &signed_extrinsic_bytes[..]) {
                                Ok(c) => c,
                                Err(e) => {
                                    let error_message = "failed to decode UncheckedExtrinsic";
                                    log::debug!(
                                        target: "mev-shield",
                                        "  id=0x{}: {} (len={}): {:?}",
                                        hex::encode(id.as_bytes()),
                                        error_message,
                                        signed_extrinsic_bytes.len(),
                                        e
                                    );
                                    failed_calls.push((id, create_failed_call(id, error_message)));
                                    continue;
                                }
                            };

                        to_submit.push((id, signed_extrinsic));
                    }

                    // ── 4) submit decrypted extrinsics to pool ──────────────────────────
                    let at = client.info().best_hash;
                    log::debug!(
                        target: "mev-shield",
                        "revealer: submitting {} extrinsics to pool at best_hash={:?}",
                        to_submit.len(),
                        at
                    );

                    for (id, uxt) in to_submit.into_iter() {
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
                                    .submit_one(at, TransactionSource::External, opaque)
                                    .await
                                {
                                    Ok(_) => {
                                        let xt_hash =
                                            sp_core::hashing::blake2_256(&xt_bytes);
                                        log::debug!(
                                            target: "mev-shield",
                                            "  id=0x{}: submit_one(...) OK, xt_hash=0x{}",
                                            hex::encode(id.as_bytes()),
                                            hex::encode(xt_hash)
                                        );
                                    }
                                    Err(e) => {
                                        // Emit an on-chain failure event even when the *inner*
                                        // transaction fails pre-dispatch validation in the pool.
                                        let err_dbg = format!("{e:?}");
                                        let reason = truncate_utf8_to_bytes(
                                            &format!(
                                                "inner extrinsic rejected by tx-pool (pre-dispatch): {err_dbg}"
                                            ),
                                            240,
                                        );
                                        log::debug!(
                                            target: "mev-shield",
                                            "  id=0x{}: submit_one(...) FAILED (will mark_decryption_failed): {:?}",
                                            hex::encode(id.as_bytes()),
                                            e
                                        );
                                        failed_calls.push((id, create_failed_call(id, &reason)));
                                    }
                                }
                            }
                            Err(e) => {
                                let err_dbg = format!("{e:?}");
                                let reason = truncate_utf8_to_bytes(
                                    &format!(
                                        "invalid decrypted extrinsic bytes (OpaqueExtrinsic::from_bytes): {err_dbg}"
                                    ),
                                    240,
                                );
                                log::debug!(
                                    target: "mev-shield",
                                    "  id=0x{}: OpaqueExtrinsic::from_bytes failed (will mark_decryption_failed): {:?}",
                                    hex::encode(id.as_bytes()),
                                    e
                                );
                                failed_calls.push((id, create_failed_call(id, &reason)));
                            }
                        }
                    }

                    // ── 5) submit decryption-failed markers ─────────────────────────────
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
                }
            },
        );
    }
}
