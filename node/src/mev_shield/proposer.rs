use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use futures::StreamExt;
use sc_service::SpawnTaskHandle;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_core::H256;
use sp_runtime::{
    generic::Era,
    MultiSignature,
    OpaqueExtrinsic,
    AccountId32,
};
use tokio::time::sleep;
use super::author::ShieldContext;
use ml_kem::{Ciphertext, Encoded, EncodedSizeUser, MlKem768, MlKem768Params};
use ml_kem::kem::{Decapsulate, DecapsulationKey};

/// Buffer of wrappers per-slot.
#[derive(Default, Clone)]
struct WrapperBuffer {
    by_id: HashMap<
        H256,
        (
            Vec<u8>,      // ciphertext blob
            u64,          // key_epoch
            AccountId32,  // wrapper author
        ),
    >,
}

impl WrapperBuffer {
    fn upsert(
        &mut self,
        id: H256,
        key_epoch: u64,
        author: AccountId32,
        ciphertext: Vec<u8>,
    ) {
        self.by_id.insert(id, (ciphertext, key_epoch, author));
    }

    /// Drain only wrappers whose `key_epoch` matches the given `epoch`.
    ///   - Wrappers with `key_epoch > epoch` are kept for future decrypt windows.
    ///   - Wrappers with `key_epoch < epoch` are considered stale and dropped.
    fn drain_for_epoch(
        &mut self,
        epoch: u64,
    ) -> Vec<(H256, u64, sp_runtime::AccountId32, Vec<u8>)> {
        let mut ready = Vec::new();
        let mut kept_future: usize = 0;
        let mut dropped_past: usize = 0;

        self.by_id.retain(|id, (ct, key_epoch, who)| {
            if *key_epoch == epoch {
                // Ready to process now; remove from buffer.
                ready.push((*id, *key_epoch, who.clone(), ct.clone()));
                false
            } else if *key_epoch > epoch {
                // Not yet reveal time; keep for future epochs.
                kept_future = kept_future.saturating_add(1);
                true
            } else {
                // key_epoch < epoch => stale / missed reveal window; drop.
                dropped_past = dropped_past.saturating_add(1);
                log::info!(
                    target: "mev-shield",
                    "revealer: dropping stale wrapper id=0x{} key_epoch={} < curr_epoch={}",
                    hex::encode(id.as_bytes()),
                    *key_epoch,
                    epoch
                );
                false
            }
        });

        log::info!(
            target: "mev-shield",
            "revealer: drain_for_epoch(epoch={}): ready={}, kept_future={}, dropped_past={}",
            epoch,
            ready.len(),
            kept_future,
            dropped_past
        );

        ready
    }
}

/// Start a background worker that:
///   • watches imported blocks and captures `MevShield::submit_encrypted`
///   • buffers those wrappers,
///   • ~last `decrypt_window_ms` of the slot: decrypt & submit unsigned `execute_revealed`
pub fn spawn_revealer<B, C, Pool>(
    task_spawner: &SpawnTaskHandle,
    client: Arc<C>,
    pool:   Arc<Pool>,
    ctx:    ShieldContext,
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
    use sp_runtime::traits::SaturatedConversion;

    type Address    = sp_runtime::MultiAddress<sp_runtime::AccountId32, ()>;
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
                log::info!(target: "mev-shield", "buffer-wrappers task started");
                let mut import_stream = client.import_notification_stream();

                while let Some(notif) = import_stream.next().await {
                    let at_hash = notif.hash;

                    log::info!(
                        target: "mev-shield",
                        "imported block hash={:?} origin={:?}",
                        at_hash, notif.origin
                    );

                    match client.block_body(at_hash) {
                        Ok(Some(body)) => {
                            log::info!(
                                target: "mev-shield",
                                "  block has {} extrinsics",
                                body.len()
                            );

                            for (idx, opaque_xt) in body.into_iter().enumerate() {
                                let encoded = opaque_xt.encode();
                                log::info!(
                                    target: "mev-shield",
                                    "    [xt #{idx}] opaque len={} bytes",
                                    encoded.len()
                                );

                                let uxt: RUnchecked = match RUnchecked::decode(&mut &encoded[..]) {
                                    Ok(u) => u,
                                    Err(e) => {
                                        log::info!(
                                            target: "mev-shield",
                                            "    [xt #{idx}] failed to decode UncheckedExtrinsic: {:?}",
                                            e
                                        );
                                        continue;
                                    }
                                };

                                log::info!(
                                    target: "mev-shield",
                                    "    [xt #{idx}] decoded call: {:?}",
                                    &uxt.0.function
                                );

                                let author_opt: Option<sp_runtime::AccountId32> =
                                    match &uxt.0.preamble {
                                        sp_runtime::generic::Preamble::Signed(addr, _sig, _ext) => {
                                            match addr.clone() {
                                                Address::Id(acc) => Some(acc),
                                                Address::Address32(bytes) =>
                                                    Some(sp_runtime::AccountId32::new(bytes)),
                                                _ => None,
                                            }
                                        }
                                        _ => None,
                                    };

                                let Some(author) = author_opt else {
                                    log::info!(
                                        target: "mev-shield",
                                        "    [xt #{idx}] not a Signed(AccountId32) extrinsic; skipping"
                                    );
                                    continue;
                                };

                                if let node_subtensor_runtime::RuntimeCall::MevShield(
                                    pallet_shield::Call::submit_encrypted {
                                        key_epoch,
                                        commitment,
                                        ciphertext,
                                        ..
                                    }
                                ) = &uxt.0.function
                                {
                                    let payload = (author.clone(), *commitment, ciphertext).encode();
                                    let id = H256(sp_core::hashing::blake2_256(&payload));

                                    log::info!(
                                        target: "mev-shield",
                                        "    [xt #{idx}] buffered submit_encrypted: id=0x{}, key_epoch={}, author={}, ct_len={}, commitment={:?}",
                                        hex::encode(id.as_bytes()),
                                        key_epoch,
                                        author,
                                        ciphertext.len(),
                                        commitment
                                    );

                                    if let Ok(mut buf) = buffer.lock() {
                                        buf.upsert(
                                            id,
                                            *key_epoch,
                                            author,
                                            ciphertext.to_vec(),
                                        );
                                    } else {
                                        log::warn!(
                                            target: "mev-shield",
                                            "    [xt #{idx}] failed to lock WrapperBuffer; dropping wrapper"
                                        );
                                    }
                                }
                            }
                        }
                        Ok(None) => log::info!(
                            target: "mev-shield",
                            "  block_body returned None for hash={:?}",
                            at_hash
                        ),
                        Err(e) => log::info!(
                            target: "mev-shield",
                            "  block_body error for hash={:?}: {:?}",
                            at_hash,
                            e
                        ),
                    }
                }
            },
        );
    }

    // ── 2) last-3s revealer ─────────────────────────────────────
    {
        let client = Arc::clone(&client);
        let pool   = Arc::clone(&pool);
        let buffer = Arc::clone(&buffer);
        let ctx    = ctx.clone();

        task_spawner.spawn(
            "mev-shield-last-3s-revealer",
            None,
            async move {
                log::info!(target: "mev-shield", "last-3s-revealer task started");

                loop {
                    let tail = ctx.timing.slot_ms.saturating_sub(ctx.timing.decrypt_window_ms);
                    log::info!(
                        target: "mev-shield",
                        "revealer: sleeping {} ms before decrypt window (slot_ms={}, decrypt_window_ms={})",
                        tail,
                        ctx.timing.slot_ms,
                        ctx.timing.decrypt_window_ms
                    );
                    sleep(Duration::from_millis(tail)).await;

                    // Snapshot the current ML‑KEM secret and epoch
                    let snapshot_opt = {
                        match ctx.keys.lock() {
                            Ok(k) => {
                                let sk_hash = sp_core::hashing::blake2_256(&k.current_sk);
                                Some((
                                    k.current_sk.clone(),
                                    k.epoch,
                                    k.current_pk.len(),
                                    k.next_pk.len(),
                                    sk_hash,
                                ))
                            }
                            Err(e) => {
                                log::warn!(
                                    target: "mev-shield",
                                    "revealer: failed to lock ShieldKeys (poisoned?): {:?}",
                                    e
                                );
                                None
                            }
                        }
                    };

                    let (curr_sk_bytes, curr_epoch, curr_pk_len, next_pk_len, sk_hash) =
                        match snapshot_opt {
                            Some(v) => v,
                            None => {
                                // Skip this decrypt window entirely, without holding any guard.
                                sleep(Duration::from_millis(ctx.timing.decrypt_window_ms)).await;
                                continue;
                            }
                        };

                    log::info!(
                        target: "mev-shield",
                        "revealer: decrypt window start. epoch={} sk_len={} sk_hash=0x{} curr_pk_len={} next_pk_len={}",
                        curr_epoch,
                        curr_sk_bytes.len(),
                        hex::encode(sk_hash),
                        curr_pk_len,
                        next_pk_len
                    );

                    // Only process wrappers whose key_epoch == curr_epoch.
                    let drained: Vec<(H256, u64, sp_runtime::AccountId32, Vec<u8>)> = {
                        match buffer.lock() {
                            Ok(mut buf) => buf.drain_for_epoch(curr_epoch),
                            Err(e) => {
                                log::warn!(
                                    target: "mev-shield",
                                    "revealer: failed to lock WrapperBuffer for drain_for_epoch: {:?}",
                                    e
                                );
                                Vec::new()
                            }
                        }
                    };

                    log::info!(
                        target: "mev-shield",
                        "revealer: drained {} buffered wrappers for current epoch={}",
                        drained.len(),
                        curr_epoch
                    );

                    let mut to_submit: Vec<(H256, node_subtensor_runtime::RuntimeCall)> = Vec::new();

                    for (id, key_epoch, author, blob) in drained.into_iter() {
                        log::info!(
                            target: "mev-shield",
                            "revealer: candidate id=0x{} key_epoch={} (curr_epoch={}) author={} blob_len={}",
                            hex::encode(id.as_bytes()),
                            key_epoch,
                            curr_epoch,
                            author,
                            blob.len()
                        );

                        // Safely parse blob: [u16 kem_len][kem_ct][nonce24][aead_ct]
                        let kem_len: usize = match blob
                            .get(0..2)
                            .and_then(|two| <[u8; 2]>::try_from(two).ok())
                        {
                            Some(arr) => u16::from_le_bytes(arr) as usize,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: blob too short or invalid length prefix",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let kem_end = match 2usize.checked_add(kem_len) {
                            Some(v) => v,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: kem_len overflow",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let nonce_end = match kem_end.checked_add(24usize) {
                            Some(v) => v,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: nonce range overflow",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let kem_ct_bytes = match blob.get(2..kem_end) {
                            Some(s) => s,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: blob too short for kem_ct (kem_len={}, total={})",
                                    hex::encode(id.as_bytes()),
                                    kem_len,
                                    blob.len()
                                );
                                continue;
                            }
                        };

                        let nonce_bytes = match blob.get(kem_end..nonce_end) {
                            Some(s) if s.len() == 24 => s,
                            _ => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: blob too short for 24-byte nonce (kem_len={}, total={})",
                                    hex::encode(id.as_bytes()),
                                    kem_len,
                                    blob.len()
                                );
                                continue;
                            }
                        };

                        let aead_body = match blob.get(nonce_end..) {
                            Some(s) => s,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: blob has no AEAD body",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let kem_ct_hash = sp_core::hashing::blake2_256(kem_ct_bytes);
                        let aead_body_hash = sp_core::hashing::blake2_256(aead_body);
                        log::info!(
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
                        let enc_sk = match Encoded::<DecapsulationKey<MlKem768Params>>::try_from(&curr_sk_bytes[..]) {
                            Ok(e) => e,
                            Err(e) => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: DecapsulationKey::try_from(sk_bytes) failed (len={}, err={:?})",
                                    hex::encode(id.as_bytes()),
                                    curr_sk_bytes.len(),
                                    e
                                );
                                continue;
                            }
                        };
                        let sk = DecapsulationKey::<MlKem768Params>::from_bytes(&enc_sk);

                        let ct = match Ciphertext::<MlKem768>::try_from(kem_ct_bytes) {
                            Ok(c) => c,
                            Err(e) => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: Ciphertext::try_from failed: {:?}",
                                    hex::encode(id.as_bytes()),
                                    e
                                );
                                continue;
                            }
                        };

                        let ss = match sk.decapsulate(&ct) {
                            Ok(s) => s,
                            Err(_) => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: ML‑KEM decapsulate() failed",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let ss_bytes: &[u8] = ss.as_ref();
                        if ss_bytes.len() != 32 {
                            log::info!(
                                target: "mev-shield",
                                "  id=0x{}: shared secret len={} != 32; skipping",
                                hex::encode(id.as_bytes()),
                                ss_bytes.len()
                            );
                            continue;
                        }
                        let mut ss32 = [0u8; 32];
                        ss32.copy_from_slice(ss_bytes);

                        let ss_hash = sp_core::hashing::blake2_256(&ss32);
                        let aead_key = crate::mev_shield::author::derive_aead_key(&ss32);
                        let key_hash = sp_core::hashing::blake2_256(&aead_key);

                        log::info!(
                            target: "mev-shield",
                            "  id=0x{}: decapsulated shared_secret_len=32 shared_secret_hash=0x{}",
                            hex::encode(id.as_bytes()),
                            hex::encode(ss_hash)
                        );
                        log::info!(
                            target: "mev-shield",
                            "  id=0x{}: derived AEAD key hash=0x{} (direct-from-ss)",
                            hex::encode(id.as_bytes()),
                            hex::encode(key_hash)
                        );

                        let mut nonce24 = [0u8; 24];
                        nonce24.copy_from_slice(nonce_bytes);

                        log::info!(
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
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: AEAD decrypt FAILED with direct-from-ss key; ct_hash=0x{}",
                                    hex::encode(id.as_bytes()),
                                    hex::encode(aead_body_hash),
                                );
                                continue;
                            }
                        };

                        log::info!(
                            target: "mev-shield",
                            "  id=0x{}: AEAD decrypt OK, plaintext_len={}",
                            hex::encode(id.as_bytes()),
                            plaintext.len()
                        );

                        type RuntimeNonce = <node_subtensor_runtime::Runtime as frame_system::Config>::Nonce;

                        // Safely parse plaintext layout without panics.
                        // Layout: signer (32) || nonce (4) || mortality (1) || call (..)
                        //         || sig_kind (1) || sig (64)
                        let min_plain_len: usize = 32usize
                            .saturating_add(4)
                            .saturating_add(1)
                            .saturating_add(1)
                            .saturating_add(64);
                        if plaintext.len() < min_plain_len {
                            log::info!(
                                target: "mev-shield",
                                "  id=0x{}: plaintext too short ({}) for expected layout",
                                hex::encode(id.as_bytes()),
                                plaintext.len()
                            );
                            continue;
                        }

                        let signer_raw = match plaintext.get(0..32) {
                            Some(s) => s,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: missing signer bytes",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let nonce_le = match plaintext.get(32..36) {
                            Some(s) => s,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: missing nonce bytes",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let mortality_byte = match plaintext.get(36) {
                            Some(b) => *b,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: missing mortality byte",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let sig_off = match plaintext.len().checked_sub(65) {
                            Some(off) if off >= 37 => off,
                            _ => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: invalid plaintext length for signature split",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let call_bytes = match plaintext.get(37..sig_off) {
                            Some(s) => s,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: missing call bytes",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let sig_kind = match plaintext.get(sig_off) {
                            Some(b) => *b,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: missing signature kind byte",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let sig_start = match sig_off.checked_add(1) {
                            Some(v) => v,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: sig_start overflow",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let sig_raw = match plaintext.get(sig_start..) {
                            Some(s) => s,
                            None => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: missing signature bytes",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };

                        let signer_array: [u8; 32] = match signer_raw.try_into() {
                            Ok(a) => a,
                            Err(_) => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: signer_raw not 32 bytes",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };
                        let signer = sp_runtime::AccountId32::new(signer_array);

                        let nonce_array: [u8; 4] = match nonce_le.try_into() {
                            Ok(a) => a,
                            Err(_) => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: nonce bytes not 4 bytes",
                                    hex::encode(id.as_bytes())
                                );
                                continue;
                            }
                        };
                        let raw_nonce_u32 = u32::from_le_bytes(nonce_array);
                        let account_nonce: RuntimeNonce = raw_nonce_u32.saturated_into();

                        // Mortality currently only supports immortal; we still
                        // parse the byte to keep layout consistent.
                        let _mortality = match mortality_byte {
                            0 => Era::Immortal,
                            _ => Era::Immortal,
                        };

                        let inner_call: node_subtensor_runtime::RuntimeCall =
                            match Decode::decode(&mut &call_bytes[..]) {
                                Ok(c) => c,
                                Err(e) => {
                                    log::info!(
                                        target: "mev-shield",
                                        "  id=0x{}: failed to decode RuntimeCall (len={}): {:?}",
                                        hex::encode(id.as_bytes()),
                                        call_bytes.len(),
                                        e
                                    );
                                    continue;
                                }
                            };

                        let signature: MultiSignature =
                            if sig_kind == 0x01 && sig_raw.len() == 64 {
                                let mut raw = [0u8; 64];
                                raw.copy_from_slice(sig_raw);
                                MultiSignature::from(sp_core::sr25519::Signature::from_raw(raw))
                            } else {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: unsupported signature format kind=0x{:02x}, len={}",
                                    hex::encode(id.as_bytes()),
                                    sig_kind,
                                    sig_raw.len()
                                );
                                continue;
                            };

                        log::info!(
                            target: "mev-shield",
                            "  id=0x{}: decrypted wrapper: signer={}, nonce={}, call={:?}",
                            hex::encode(id.as_bytes()),
                            signer,
                            raw_nonce_u32,
                            inner_call
                        );

                        let reveal = node_subtensor_runtime::RuntimeCall::MevShield(
                            pallet_shield::Call::execute_revealed {
                                id,
                                signer: signer.clone(),
                                nonce: account_nonce,
                                mortality: Era::Immortal,
                                call: Box::new(inner_call),
                                signature,
                            }
                        );

                        to_submit.push((id, reveal));
                    }

                    // Submit locally.
                    let at = client.info().best_hash;
                    log::info!(
                        target: "mev-shield",
                        "revealer: submitting {} execute_revealed calls at best_hash={:?}",
                        to_submit.len(),
                        at
                    );

                    for (id, call) in to_submit.into_iter() {
                        let uxt: node_subtensor_runtime::UncheckedExtrinsic =
                            node_subtensor_runtime::UncheckedExtrinsic::new_bare(call);
                        let xt_bytes = uxt.encode();

                        log::info!(
                            target: "mev-shield",
                            "  id=0x{}: encoded UncheckedExtrinsic len={}",
                            hex::encode(id.as_bytes()),
                            xt_bytes.len()
                        );

                        match OpaqueExtrinsic::from_bytes(&xt_bytes) {
                            Ok(opaque) => {
                                match pool.submit_one(at, TransactionSource::Local, opaque).await {
                                    Ok(_) => {
                                        let xt_hash = sp_core::hashing::blake2_256(&xt_bytes);
                                        log::info!(
                                            target: "mev-shield",
                                            "  id=0x{}: submit_one(execute_revealed) OK, xt_hash=0x{}",
                                            hex::encode(id.as_bytes()),
                                            hex::encode(xt_hash)
                                        );
                                    }
                                    Err(e) => {
                                        log::info!(
                                            target: "mev-shield",
                                            "  id=0x{}: submit_one(execute_revealed) FAILED: {:?}",
                                            hex::encode(id.as_bytes()),
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                log::info!(
                                    target: "mev-shield",
                                    "  id=0x{}: OpaqueExtrinsic::from_bytes failed: {:?}",
                                    hex::encode(id.as_bytes()),
                                    e
                                );
                            }
                        }
                    }

                    // Let the decrypt window elapse.
                    sleep(Duration::from_millis(ctx.timing.decrypt_window_ms)).await;
                }
            },
        );
    }
}
