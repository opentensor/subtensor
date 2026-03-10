use crate::mock::*;
use crate::{
    AuthorKeys, CurrentKey, Error, HasMigrationRun, NextKey, NextKeyExpiresAt, PendingKey,
    PendingKeyExpiresAt,
};

use codec::Encode;
use frame_support::{BoundedVec, assert_noop, assert_ok};
use sp_runtime::testing::TestSignature;
use sp_runtime::traits::{Block as BlockT, Hash};
use stp_shield::{MLKEM768_ENC_KEY_LEN, ShieldEncKey, ShieldKeystore, ShieldedTransaction};

use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use ml_kem::{
    EncodedSizeUser, MlKem768Params,
    kem::{Encapsulate, EncapsulationKey},
};
use rand_chacha::{ChaChaRng, rand_core::SeedableRng};
use stc_shield::MemoryShieldKeystore;

/// Simulates a 3-validator round-robin (authors 1, 2, 3) over 5 blocks.
/// Each block calls `announce_next_key` and verifies the full pipeline:
/// CurrentKey, PendingKey, NextKey, AuthorKeys, expirations, and
/// `is_shielded_using_current_key`.
#[test]
fn key_rotation_round_robin() {
    new_test_ext().execute_with(|| {
        let key_of =
            |n: u8| -> ShieldEncKey { BoundedVec::truncate_from(vec![n; MLKEM768_ENC_KEY_LEN]) };
        let hash_of = |pk: &ShieldEncKey| sp_io::hashing::twox_128(&pk[..]);

        // 3 validators in round-robin: 1, 2, 3, 1, 2.
        let authors = [1u8, 2, 3, 1, 2];
        let next_next = |block: usize| -> Option<u8> { authors.get(block + 2).copied() };

        // ── Block 1: author=1, next_next=3 ──────────────────────────────
        // Pipeline is empty; author(3) has no AuthorKeys yet.
        System::set_block_number(1);
        set_authors(Some(author(1)), next_next(0).map(author));
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(key_of(1)),
        ));

        assert!(CurrentKey::<Test>::get().is_none());
        assert!(PendingKey::<Test>::get().is_none());
        assert!(NextKey::<Test>::get().is_none());
        assert_eq!(AuthorKeys::<Test>::get(author(1)), Some(key_of(1)));
        assert!(PendingKeyExpiresAt::<Test>::get().is_none());
        assert!(NextKeyExpiresAt::<Test>::get().is_none());
        // Nothing in PendingKey → is_shielded always false.
        assert!(!MevShield::is_shielded_using_current_key(&[0xFF; 16]));

        // ── Block 2: author=2, next_next=1 ──────────────────────────────
        // author(1) registered in block 1 → NextKey picks up key_of(1).
        System::set_block_number(2);
        set_authors(Some(author(2)), next_next(1).map(author));
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(key_of(2)),
        ));

        assert!(CurrentKey::<Test>::get().is_none());
        assert!(PendingKey::<Test>::get().is_none());
        assert_eq!(NextKey::<Test>::get(), Some(key_of(1)));
        assert_eq!(AuthorKeys::<Test>::get(author(2)), Some(key_of(2)));
        assert!(PendingKeyExpiresAt::<Test>::get().is_none());
        assert_eq!(NextKeyExpiresAt::<Test>::get(), Some(5)); // 2 + 3

        // ── Block 3: author=3, next_next=2 ──────────────────────────────
        // NextKey(key_of(1)) → PendingKey; next_next=author(2) has key_of(2) → NextKey.
        System::set_block_number(3);
        set_authors(Some(author(3)), next_next(2).map(author));
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(key_of(3)),
        ));

        assert!(CurrentKey::<Test>::get().is_none());
        assert_eq!(PendingKey::<Test>::get(), Some(key_of(1)));
        assert_eq!(NextKey::<Test>::get(), Some(key_of(2)));
        assert_eq!(AuthorKeys::<Test>::get(author(3)), Some(key_of(3)));
        assert_eq!(PendingKeyExpiresAt::<Test>::get(), Some(5)); // 3 + 2
        assert_eq!(NextKeyExpiresAt::<Test>::get(), Some(6)); // 3 + 3
        // PendingKey = key_of(1) → is_shielded matches its hash.
        assert!(MevShield::is_shielded_using_current_key(&hash_of(&key_of(
            1
        ))));
        assert!(!MevShield::is_shielded_using_current_key(&hash_of(
            &key_of(2)
        )));
        assert!(!MevShield::is_shielded_using_current_key(&[0xFF; 16]));

        // ── Block 4: author=1, next_next=out of bounds ──────────────────
        // Full pipeline: PendingKey(key_of(1)) → CurrentKey, NextKey(key_of(2)) → PendingKey.
        System::set_block_number(4);
        set_authors(Some(author(1)), next_next(3).map(author));
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(key_of(1)),
        ));

        assert_eq!(CurrentKey::<Test>::get(), Some(key_of(1)));
        assert_eq!(PendingKey::<Test>::get(), Some(key_of(2)));
        assert!(NextKey::<Test>::get().is_none());
        assert_eq!(AuthorKeys::<Test>::get(author(1)), Some(key_of(1)));
        assert_eq!(PendingKeyExpiresAt::<Test>::get(), Some(6)); // 4 + 2
        assert!(NextKeyExpiresAt::<Test>::get().is_none());
        // PendingKey = key_of(2).
        assert!(MevShield::is_shielded_using_current_key(&hash_of(&key_of(
            2
        ))));
        assert!(!MevShield::is_shielded_using_current_key(&hash_of(
            &key_of(1)
        )));

        // ── Block 5: author=2, next_next=none ───────────────────────────
        // PendingKey(key_of(2)) → CurrentKey; pipeline drains.
        System::set_block_number(5);
        set_authors(Some(author(2)), None);
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(key_of(2)),
        ));

        assert_eq!(CurrentKey::<Test>::get(), Some(key_of(2)));
        assert!(PendingKey::<Test>::get().is_none());
        assert!(NextKey::<Test>::get().is_none());
        assert!(PendingKeyExpiresAt::<Test>::get().is_none());
        assert!(NextKeyExpiresAt::<Test>::get().is_none());
    });
}

/// AuthorKeys is read *before* being updated, so when current == next_next
/// the NextKey picks up the old key, not the newly announced one.
#[test]
fn announce_rotations_use_pre_update_author_keys() {
    new_test_ext().execute_with(|| {
        set_authors(Some(author(1)), Some(author(1)));

        let old_pk = valid_pk();
        let new_pk = valid_pk_b();
        AuthorKeys::<Test>::insert(author(1), old_pk.clone());

        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(new_pk.clone()),
        ));

        assert_eq!(NextKey::<Test>::get(), Some(old_pk));
        assert_eq!(AuthorKeys::<Test>::get(author(1)), Some(new_pk));
    });
}

#[test]
fn announce_rejects_signed_origin() {
    new_test_ext().execute_with(|| {
        set_authors(Some(author(1)), None);
        assert_noop!(
            MevShield::announce_next_key(RuntimeOrigin::signed(1), Some(valid_pk())),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn announce_rejects_bad_pk_length() {
    new_test_ext().execute_with(|| {
        set_authors(Some(author(1)), None);
        let bad_pk: ShieldEncKey = BoundedVec::truncate_from(vec![0x01; 100]);

        assert_noop!(
            MevShield::announce_next_key(RuntimeOrigin::none(), Some(bad_pk)),
            Error::<Test>::BadEncKeyLen
        );
    });
}

#[test]
fn announce_none_pk_removes_author_key() {
    new_test_ext().execute_with(|| {
        set_authors(Some(author(1)), None);
        AuthorKeys::<Test>::insert(author(1), valid_pk());

        assert_ok!(MevShield::announce_next_key(RuntimeOrigin::none(), None));

        assert!(AuthorKeys::<Test>::get(author(1)).is_none());
    });
}

#[test]
fn announce_fails_when_no_current_author() {
    new_test_ext().execute_with(|| {
        set_authors(None, None);

        assert_noop!(
            MevShield::announce_next_key(RuntimeOrigin::none(), Some(valid_pk())),
            Error::<Test>::Unreachable
        );
    });
}

#[test]
fn submit_encrypted_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let ciphertext = BoundedVec::truncate_from(vec![0xAA; 64]);
        let who: u64 = 1;

        assert_ok!(MevShield::submit_encrypted(
            RuntimeOrigin::signed(who),
            ciphertext.clone(),
        ));

        let expected_id = <Test as frame_system::Config>::Hashing::hash_of(&(who, &ciphertext));

        System::assert_last_event(
            crate::Event::<Test>::EncryptedSubmitted {
                id: expected_id,
                who,
            }
            .into(),
        );
    });
}

#[test]
fn submit_encrypted_rejects_unsigned() {
    new_test_ext().execute_with(|| {
        let ciphertext = BoundedVec::truncate_from(vec![0xAA; 64]);

        assert_noop!(
            MevShield::submit_encrypted(RuntimeOrigin::none(), ciphertext),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn try_decode_shielded_tx_parses_bare_submit_encrypted() {
    new_test_ext().execute_with(|| {
        let key_hash = [0xAB; 16];
        let kem_ct = vec![0xCC; 32];
        let nonce = [0xDD; 24];
        let aead_ct = vec![0xEE; 64];

        let ciphertext = build_wire_ciphertext(&key_hash, &kem_ct, &nonce, &aead_ct);
        let call = RuntimeCall::MevShield(crate::Call::submit_encrypted {
            ciphertext: BoundedVec::truncate_from(ciphertext),
        });
        let uxt = DecodableExtrinsic::new_bare(call);

        let result = crate::Pallet::<Test>::try_decode_shielded_tx::<
            DecodableBlock,
            frame_system::ChainContext<Test>,
        >(uxt);
        assert!(result.is_some());

        let shielded = result.unwrap();
        assert_eq!(shielded.key_hash, key_hash);
        assert_eq!(shielded.kem_ct, kem_ct);
        assert_eq!(shielded.nonce, nonce);
        assert_eq!(shielded.aead_ct, aead_ct);
    });
}

#[test]
fn try_decode_shielded_tx_returns_none_for_non_shield_call() {
    new_test_ext().execute_with(|| {
        let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
        let uxt = DecodableExtrinsic::new_bare(call);

        let result = crate::Pallet::<Test>::try_decode_shielded_tx::<
            DecodableBlock,
            frame_system::ChainContext<Test>,
        >(uxt);
        assert!(result.is_none());
    });
}

#[test]
fn try_decode_shielded_tx_returns_none_for_bad_signature() {
    new_test_ext().execute_with(|| {
        let ciphertext = build_wire_ciphertext(&[0xAB; 16], &[0xCC; 32], &[0xDD; 24], &[0xEE; 64]);
        let call = RuntimeCall::MevShield(crate::Call::submit_encrypted {
            ciphertext: BoundedVec::truncate_from(ciphertext),
        });
        let bad_sig = TestSignature(1, vec![0xFF; 32]);
        let uxt = DecodableExtrinsic::new_signed(call, 1u64, bad_sig, ());

        let result = crate::Pallet::<Test>::try_decode_shielded_tx::<
            DecodableBlock,
            frame_system::ChainContext<Test>,
        >(uxt);
        assert!(result.is_none());
    });
}

#[test]
fn try_decode_shielded_tx_returns_none_for_malformed_ciphertext() {
    new_test_ext().execute_with(|| {
        let call = RuntimeCall::MevShield(crate::Call::submit_encrypted {
            ciphertext: BoundedVec::truncate_from(vec![0u8; 5]),
        });
        let uxt = DecodableExtrinsic::new_bare(call);

        let result = crate::Pallet::<Test>::try_decode_shielded_tx::<
            DecodableBlock,
            frame_system::ChainContext<Test>,
        >(uxt);
        assert!(result.is_none());
    });
}

#[test]
fn try_decode_shielded_tx_returns_none_when_depth_exceeded() {
    new_test_ext().execute_with(|| {
        let ciphertext = build_wire_ciphertext(&[0xAB; 16], &[0xCC; 32], &[0xDD; 24], &[0xEE; 64]);
        let inner = RuntimeCall::MevShield(crate::Call::submit_encrypted {
            ciphertext: BoundedVec::truncate_from(ciphertext),
        });
        let call = nest_call(inner, 8);
        let uxt = DecodableExtrinsic::new_bare(call);

        let result = crate::Pallet::<Test>::try_decode_shielded_tx::<
            DecodableBlock,
            frame_system::ChainContext<Test>,
        >(uxt);
        assert!(result.is_none());
    });
}

#[test]
fn try_unshield_tx_decrypts_extrinsic() {
    let mut rng = ChaChaRng::from_seed([42u8; 32]);
    let keystore = MemoryShieldKeystore::new();

    // Client side: read the announced encapsulation key and encapsulate.
    let pk_bytes = keystore.next_enc_key().unwrap();
    let enc_key =
        EncapsulationKey::<MlKem768Params>::from_bytes(pk_bytes.as_slice().try_into().unwrap());
    let (kem_ct, shared_secret) = enc_key.encapsulate(&mut rng).unwrap();

    // Build the inner extrinsic that we'll encrypt.
    let inner_call = RuntimeCall::System(frame_system::Call::remark {
        remark: vec![1, 2, 3],
    });
    let inner_uxt = <Block as BlockT>::Extrinsic::new_bare(inner_call);
    let plaintext = inner_uxt.encode();

    // AEAD encrypt the extrinsic bytes.
    let nonce = [42u8; 24];
    let cipher = XChaCha20Poly1305::new(shared_secret.as_slice().into());
    let aead_ct = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &plaintext,
                aad: &[],
            },
        )
        .unwrap();

    // Roll keystore so next -> current (author side).
    keystore.roll_for_next_slot().unwrap();
    let dec_key_bytes = keystore.current_dec_key().unwrap();

    let shielded_tx = ShieldedTransaction {
        key_hash: [0u8; 16],
        kem_ct: kem_ct.as_slice().to_vec(),
        nonce,
        aead_ct,
    };

    let result = crate::Pallet::<Test>::try_unshield_tx::<Block>(dec_key_bytes, shielded_tx);
    assert!(result.is_some());

    let decoded = result.unwrap();
    assert_eq!(decoded.encode(), inner_uxt.encode());
}

// ---------------------------------------------------------------------------
// Migration tests
// ---------------------------------------------------------------------------

mod migration_tests {
    use super::*;
    use crate::migrations::migrate_clear_v1_storage::migrate_clear_v1_storage;
    use sp_io::hashing::twox_128;

    #[test]
    fn migrate_clear_v1_storage_works() {
        new_test_ext().execute_with(|| {
            // Seed legacy storage that should be cleared.
            seed_legacy_map("Submissions", 5);
            seed_legacy_map("KeyHashByBlock", 3);
            CurrentKey::<Test>::put(valid_pk());

            // Current storage that must survive.
            NextKey::<Test>::put(valid_pk());
            AuthorKeys::<Test>::insert(author(1), valid_pk_b());

            // Sanity: legacy values exist.
            assert_eq!(count_keys("Submissions"), 5);
            assert_eq!(count_keys("KeyHashByBlock"), 3);
            assert!(CurrentKey::<Test>::get().is_some());

            migrate_clear_v1_storage::<Test>();

            // Legacy storage cleared.
            assert_eq!(count_keys("Submissions"), 0);
            assert_eq!(count_keys("KeyHashByBlock"), 0);
            assert!(CurrentKey::<Test>::get().is_none());

            // Current storage untouched.
            assert_eq!(NextKey::<Test>::get(), Some(valid_pk()));
            assert_eq!(AuthorKeys::<Test>::get(author(1)), Some(valid_pk_b()));

            // Migration was recorded.
            let mig_key = BoundedVec::truncate_from(b"migrate_clear_v1_storage".to_vec());
            assert!(HasMigrationRun::<Test>::get(&mig_key));

            // Idempotent: re-run doesn't touch new data.
            CurrentKey::<Test>::put(valid_pk_b());
            migrate_clear_v1_storage::<Test>();
            assert_eq!(CurrentKey::<Test>::get(), Some(valid_pk_b()));
        });
    }

    fn seed_legacy_map(storage_name: &str, count: u32) {
        let mut prefix = Vec::new();
        prefix.extend_from_slice(&twox_128(b"MevShield"));
        prefix.extend_from_slice(&twox_128(storage_name.as_bytes()));

        for i in 0..count {
            let mut key = prefix.clone();
            key.extend_from_slice(&i.to_le_bytes());
            sp_io::storage::set(&key, &[1u8; 32]);
        }
    }

    fn count_keys(storage_name: &str) -> u32 {
        let mut prefix = Vec::new();
        prefix.extend_from_slice(&twox_128(b"MevShield"));
        prefix.extend_from_slice(&twox_128(storage_name.as_bytes()));

        let mut count = 0u32;
        let mut next_key = sp_io::storage::next_key(&prefix);
        while let Some(key) = next_key {
            if !key.starts_with(&prefix) {
                break;
            }
            count += 1;
            next_key = sp_io::storage::next_key(&key);
        }
        count
    }
}
