use crate as pallet_mev_shield;
use crate::mock::*;

use codec::Encode;
use frame_support::pallet_prelude::ValidateUnsigned;
use frame_support::traits::ConstU32 as FrameConstU32;
use frame_support::traits::Hooks;
use frame_support::{BoundedVec, assert_noop, assert_ok};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_mev_shield::{
    Call as MevShieldCall, CurrentKey, Event as MevShieldEvent, KeyHashByBlock, NextKey,
    Submissions,
};
use sp_core::Pair;
use sp_core::sr25519;
use sp_runtime::traits::{Hash, SaturatedConversion};
use sp_runtime::{AccountId32, MultiSignature, transaction_validity::TransactionSource};

// Type aliases for convenience in tests.
type TestHash = <Test as frame_system::Config>::Hash;
type TestBlockNumber = BlockNumberFor<Test>;

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

/// Deterministic sr25519 pair for tests (acts as "Alice").
fn test_sr25519_pair() -> sr25519::Pair {
    sr25519::Pair::from_seed(&[1u8; 32])
}

/// Reproduce the pallet's raw payload layout:
///   signer (32B) || key_hash (Hash bytes) || SCALE(call)
fn build_raw_payload_bytes_for_test(
    signer: &AccountId32,
    key_hash: &TestHash,
    call: &RuntimeCall,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(signer.as_ref());
    out.extend_from_slice(key_hash.as_ref());
    out.extend(call.encode());
    out
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[test]
fn authority_can_announce_next_key_and_on_initialize_rolls_it_and_records_epoch_hash() {
    new_test_ext().execute_with(|| {
        const KYBER_PK_LEN: usize = 1184;
        let pk_bytes = vec![7u8; KYBER_PK_LEN];
        let bounded_pk: BoundedVec<u8, FrameConstU32<2048>> =
            BoundedVec::truncate_from(pk_bytes.clone());

        // Seed Aura authorities with a single validator and derive the matching account.
        let validator_pair = test_sr25519_pair();
        let validator_account: AccountId32 = validator_pair.public().into();
        let validator_aura_id: <Test as pallet_aura::Config>::AuthorityId =
            validator_pair.public().into();

        // Authorities storage expects a BoundedVec<AuthorityId, MaxAuthorities>.
        let authorities: BoundedVec<
            <Test as pallet_aura::Config>::AuthorityId,
            <Test as pallet_aura::Config>::MaxAuthorities,
        > = BoundedVec::truncate_from(vec![validator_aura_id.clone()]);
        pallet_aura::Authorities::<Test>::put(authorities);

        assert!(CurrentKey::<Test>::get().is_none());
        assert!(NextKey::<Test>::get().is_none());

        // Signed by an Aura validator -> passes TestAuthorityOrigin::ensure_validator.
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::signed(validator_account.clone()),
            bounded_pk.clone(),
        ));

        // NextKey storage updated
        let next = NextKey::<Test>::get().expect("NextKey should be set");
        assert_eq!(next, pk_bytes);

        // Simulate beginning of block #2.
        let block_two: TestBlockNumber = 2u64.saturated_into();
        MevShield::on_initialize(block_two);

        // CurrentKey should now equal the previously announced NextKey.
        let curr = CurrentKey::<Test>::get().expect("CurrentKey should be set");
        assert_eq!(curr, pk_bytes);

        // And NextKey cleared.
        assert!(NextKey::<Test>::get().is_none());

        // Key hash for this block should be recorded and equal hash(CurrentKey_bytes).
        let expected_hash: TestHash = <Test as frame_system::Config>::Hashing::hash(curr.as_ref());
        let recorded =
            KeyHashByBlock::<Test>::get(block_two).expect("epoch key hash must be recorded");
        assert_eq!(recorded, expected_hash);
    });
}

#[test]
fn announce_next_key_rejects_non_validator_origins() {
    new_test_ext().execute_with(|| {
        const KYBER_PK_LEN: usize = 1184;

        // Validator account: bytes match the Aura authority we put into storage.
        let validator_pair = test_sr25519_pair();
        let validator_account: AccountId32 = validator_pair.public().into();
        let validator_aura_id: <Test as pallet_aura::Config>::AuthorityId =
            validator_pair.public().into();

        // Non‑validator is some other key (not in Aura::Authorities<Test>).
        let non_validator_pair = sr25519::Pair::from_seed(&[2u8; 32]);
        let non_validator: AccountId32 = non_validator_pair.public().into();

        // Only the validator is in the Aura validator set.
        let authorities: BoundedVec<
            <Test as pallet_aura::Config>::AuthorityId,
            <Test as pallet_aura::Config>::MaxAuthorities,
        > = BoundedVec::truncate_from(vec![validator_aura_id.clone()]);
        pallet_aura::Authorities::<Test>::put(authorities);

        let pk_bytes = vec![9u8; KYBER_PK_LEN];
        let bounded_pk: BoundedVec<u8, FrameConstU32<2048>> =
            BoundedVec::truncate_from(pk_bytes.clone());

        // 1) Signed non‑validator origin must fail with BadOrigin.
        assert_noop!(
            MevShield::announce_next_key(
                RuntimeOrigin::signed(non_validator.clone()),
                bounded_pk.clone(),
            ),
            sp_runtime::DispatchError::BadOrigin
        );

        // 2) Unsigned origin must also fail with BadOrigin.
        assert_noop!(
            MevShield::announce_next_key(RuntimeOrigin::none(), bounded_pk.clone(),),
            sp_runtime::DispatchError::BadOrigin
        );

        // 3) Signed validator origin succeeds (sanity check).
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::signed(validator_account.clone()),
            bounded_pk.clone(),
        ));

        let next = NextKey::<Test>::get().expect("NextKey must be set by validator");
        assert_eq!(next, pk_bytes);
    });
}

#[test]
fn submit_encrypted_stores_submission_and_emits_event() {
    new_test_ext().execute_with(|| {
        let pair = test_sr25519_pair();
        let who: AccountId32 = pair.public().into();

        System::set_block_number(10);

        let commitment =
            <Test as frame_system::Config>::Hashing::hash(b"test-mevshield-commitment");
        let ciphertext_bytes = vec![1u8, 2, 3, 4];
        let ciphertext: BoundedVec<u8, FrameConstU32<8192>> =
            BoundedVec::truncate_from(ciphertext_bytes.clone());

        assert_ok!(MevShield::submit_encrypted(
            RuntimeOrigin::signed(who.clone()),
            commitment,
            ciphertext.clone(),
        ));

        let id = <Test as frame_system::Config>::Hashing::hash_of(&(
            who.clone(),
            commitment,
            &ciphertext,
        ));

        let stored = Submissions::<Test>::get(id).expect("submission stored");
        assert_eq!(stored.author, who);
        assert_eq!(stored.commitment, commitment);
        assert_eq!(stored.ciphertext.to_vec(), ciphertext_bytes);
        assert_eq!(stored.submitted_in, 10);

        let events = System::events();
        let last = events.last().expect("at least one event").event.clone();

        assert!(
            matches!(
                last,
                RuntimeEvent::MevShield(
                    MevShieldEvent::<Test>::EncryptedSubmitted { id: ev_id, who: ev_who }
                )
                if ev_id == id && ev_who == who
            ),
            "expected EncryptedSubmitted event with correct id & who",
        );
    });
}

#[test]
fn execute_revealed_happy_path_verifies_and_executes_inner_call() {
    new_test_ext().execute_with(|| {
        let pair = test_sr25519_pair();
        let signer: AccountId32 = pair.public().into();

        // Inner call – System.remark; must dispatch successfully.
        let inner_call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: b"hello-mevshield".to_vec(),
        });

        // Choose a deterministic epoch key hash and wire it up for block #1.
        let key_hash: TestHash = <Test as frame_system::Config>::Hashing::hash(b"epoch-key");
        let payload_bytes = build_raw_payload_bytes_for_test(&signer, &key_hash, &inner_call);

        let commitment: TestHash =
            <Test as frame_system::Config>::Hashing::hash(payload_bytes.as_ref());

        let ciphertext_bytes = vec![9u8, 9, 9, 9];
        let ciphertext: BoundedVec<u8, FrameConstU32<8192>> =
            BoundedVec::truncate_from(ciphertext_bytes.clone());

        // All submissions in this test happen at block #1.
        System::set_block_number(1);
        let submitted_in = System::block_number();
        // Record epoch hash for that block, as on_initialize would do.
        KeyHashByBlock::<Test>::insert(submitted_in, key_hash);

        // Wrapper author == signer for simplest path
        assert_ok!(MevShield::submit_encrypted(
            RuntimeOrigin::signed(signer.clone()),
            commitment,
            ciphertext.clone(),
        ));

        let id: TestHash = <Test as frame_system::Config>::Hashing::hash_of(&(
            signer.clone(),
            commitment,
            &ciphertext,
        ));

        // Build message "mev-shield:v1" || genesis_hash || payload
        let genesis = System::block_hash(0);
        let mut msg = b"mev-shield:v1".to_vec();
        msg.extend_from_slice(genesis.as_ref());
        msg.extend_from_slice(&payload_bytes);

        let sig_sr25519 = pair.sign(&msg);
        let signature: MultiSignature = sig_sr25519.into();

        let result = MevShield::execute_revealed(
            RuntimeOrigin::none(),
            id,
            signer.clone(),
            key_hash,
            Box::new(inner_call.clone()),
            signature,
        );

        assert_ok!(result);

        // Submission consumed
        assert!(Submissions::<Test>::get(id).is_none());

        // Last event is DecryptedExecuted
        let events = System::events();
        let last = events
            .last()
            .expect("an event should be emitted")
            .event
            .clone();

        assert!(
            matches!(
                last,
                RuntimeEvent::MevShield(
                    MevShieldEvent::<Test>::DecryptedExecuted { id: ev_id, signer: ev_signer }
                )
                if ev_id == id && ev_signer == signer
            ),
            "expected DecryptedExecuted event"
        );
    });
}

#[test]
fn execute_revealed_fails_on_key_hash_mismatch() {
    new_test_ext().execute_with(|| {
        let pair = test_sr25519_pair();
        let signer: AccountId32 = pair.public().into();

        let inner_call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: b"bad-key-hash".to_vec(),
        });

        System::set_block_number(5);
        let submitted_in = System::block_number();

        // Epoch hash recorded for this block:
        let correct_key_hash: TestHash =
            <Test as frame_system::Config>::Hashing::hash(b"correct-epoch");
        KeyHashByBlock::<Test>::insert(submitted_in, correct_key_hash);

        // But we build payload & commitment with a *different* key_hash.
        let wrong_key_hash: TestHash =
            <Test as frame_system::Config>::Hashing::hash(b"wrong-epoch");

        let payload_bytes = build_raw_payload_bytes_for_test(&signer, &wrong_key_hash, &inner_call);
        let commitment: TestHash =
            <Test as frame_system::Config>::Hashing::hash(payload_bytes.as_ref());

        let ciphertext_bytes = vec![0u8; 4];
        let ciphertext: BoundedVec<u8, FrameConstU32<8192>> =
            BoundedVec::truncate_from(ciphertext_bytes);

        assert_ok!(MevShield::submit_encrypted(
            RuntimeOrigin::signed(signer.clone()),
            commitment,
            ciphertext.clone(),
        ));

        let id: TestHash = <Test as frame_system::Config>::Hashing::hash_of(&(
            signer.clone(),
            commitment,
            &ciphertext,
        ));

        let genesis = System::block_hash(0);
        let mut msg = b"mev-shield:v1".to_vec();
        msg.extend_from_slice(genesis.as_ref());
        msg.extend_from_slice(&payload_bytes);

        let sig_sr25519 = pair.sign(&msg);
        let signature: MultiSignature = sig_sr25519.into();

        // execute_revealed should fail with KeyHashMismatch.
        let res = MevShield::execute_revealed(
            RuntimeOrigin::none(),
            id,
            signer.clone(),
            wrong_key_hash,
            Box::new(inner_call.clone()),
            signature,
        );
        assert_noop!(res, pallet_mev_shield::Error::<Test>::KeyHashMismatch);
    });
}

#[test]
fn execute_revealed_rejects_replay_for_same_wrapper_id() {
    new_test_ext().execute_with(|| {
        let pair = test_sr25519_pair();
        let signer: AccountId32 = pair.public().into();

        let inner_call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: b"replay-test".to_vec(),
        });

        System::set_block_number(10);
        let submitted_in = System::block_number();

        let key_hash: TestHash = <Test as frame_system::Config>::Hashing::hash(b"replay-epoch");
        KeyHashByBlock::<Test>::insert(submitted_in, key_hash);

        let payload_bytes = build_raw_payload_bytes_for_test(&signer, &key_hash, &inner_call);
        let commitment: TestHash =
            <Test as frame_system::Config>::Hashing::hash(payload_bytes.as_ref());

        let ciphertext_bytes = vec![7u8; 16];
        let ciphertext: BoundedVec<u8, FrameConstU32<8192>> =
            BoundedVec::truncate_from(ciphertext_bytes.clone());

        assert_ok!(MevShield::submit_encrypted(
            RuntimeOrigin::signed(signer.clone()),
            commitment,
            ciphertext.clone(),
        ));

        let id: TestHash = <Test as frame_system::Config>::Hashing::hash_of(&(
            signer.clone(),
            commitment,
            &ciphertext,
        ));

        let genesis = System::block_hash(0);
        let mut msg = b"mev-shield:v1".to_vec();
        msg.extend_from_slice(genesis.as_ref());
        msg.extend_from_slice(&payload_bytes);

        let sig_sr25519 = pair.sign(&msg);
        let signature: MultiSignature = sig_sr25519.into();

        // First execution succeeds.
        assert_ok!(MevShield::execute_revealed(
            RuntimeOrigin::none(),
            id,
            signer.clone(),
            key_hash,
            Box::new(inner_call.clone()),
            signature.clone(),
        ));

        // Second execution with the same id must fail with MissingSubmission.
        let res = MevShield::execute_revealed(
            RuntimeOrigin::none(),
            id,
            signer.clone(),
            key_hash,
            Box::new(inner_call.clone()),
            signature,
        );
        assert_noop!(res, pallet_mev_shield::Error::<Test>::MissingSubmission);
    });
}

#[test]
fn key_hash_by_block_prunes_old_entries() {
    new_test_ext().execute_with(|| {
        // This must match the constant configured in the pallet.
        const KEEP: u64 = 100;
        const TOTAL: u64 = KEEP + 5;

        // For each block n, set a CurrentKey and call on_initialize(n),
        // which will record KeyHashByBlock[n] and prune old entries.
        for n in 1..=TOTAL {
            let key_bytes = vec![n as u8; 32];
            let bounded: BoundedVec<u8, FrameConstU32<2048>> =
                BoundedVec::truncate_from(key_bytes.clone());

            CurrentKey::<Test>::put(bounded.clone());

            let bn: TestBlockNumber = n.saturated_into();
            MevShield::on_initialize(bn);
        }

        // The oldest block that should still be kept after TOTAL blocks.
        let oldest_kept: u64 = if TOTAL > KEEP { TOTAL - KEEP + 1 } else { 1 };

        // Blocks strictly before oldest_kept must be pruned.
        for old in 0..oldest_kept {
            let bn: TestBlockNumber = old.saturated_into();
            assert!(
                KeyHashByBlock::<Test>::get(bn).is_none(),
                "block {bn:?} should have been pruned"
            );
        }

        // Blocks from oldest_kept..=TOTAL must still have entries.
        for recent in oldest_kept..=TOTAL {
            let bn: TestBlockNumber = recent.saturated_into();
            assert!(
                KeyHashByBlock::<Test>::get(bn).is_some(),
                "block {bn:?} should be retained"
            );
        }

        // Additionally, assert we never exceed the configured cap.
        let mut count: u64 = 0;
        for bn in 0..=TOTAL {
            let bn_t: TestBlockNumber = bn.saturated_into();
            if KeyHashByBlock::<Test>::get(bn_t).is_some() {
                count += 1;
            }
        }
        let expected = KEEP.min(TOTAL);
        assert_eq!(
            count, expected,
            "expected at most {expected} entries in KeyHashByBlock after pruning, got {count}"
        );
    });
}

#[test]
fn validate_unsigned_accepts_local_source_for_execute_revealed() {
    new_test_ext().execute_with(|| {
        let pair = test_sr25519_pair();
        let signer: AccountId32 = pair.public().into();

        let inner_call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: b"noop-local".to_vec(),
        });

        let id: TestHash = <Test as frame_system::Config>::Hashing::hash(b"mevshield-id-local");
        let key_hash: TestHash = <Test as frame_system::Config>::Hashing::hash(b"epoch-for-local");
        let signature: MultiSignature = sr25519::Signature::from_raw([0u8; 64]).into();

        let call = MevShieldCall::<Test>::execute_revealed {
            id,
            signer,
            key_hash,
            call: Box::new(inner_call),
            signature,
        };

        let validity = MevShield::validate_unsigned(TransactionSource::Local, &call);
        assert_ok!(validity);
    });
}

#[test]
fn validate_unsigned_accepts_inblock_source_for_execute_revealed() {
    new_test_ext().execute_with(|| {
        let pair = test_sr25519_pair();
        let signer: AccountId32 = pair.public().into();

        let inner_call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: b"noop-inblock".to_vec(),
        });

        let id: TestHash = <Test as frame_system::Config>::Hashing::hash(b"mevshield-id-inblock");
        let key_hash: TestHash =
            <Test as frame_system::Config>::Hashing::hash(b"epoch-for-inblock");
        let signature: MultiSignature = sr25519::Signature::from_raw([1u8; 64]).into();

        let call = MevShieldCall::<Test>::execute_revealed {
            id,
            signer,
            key_hash,
            call: Box::new(inner_call),
            signature,
        };

        let validity = MevShield::validate_unsigned(TransactionSource::InBlock, &call);
        assert_ok!(validity);
    });
}
