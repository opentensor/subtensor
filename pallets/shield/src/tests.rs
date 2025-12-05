use crate as pallet_mev_shield;
use crate::mock::*;

use frame_support::{
    BoundedVec, assert_noop, assert_ok,
    traits::{ConstU32 as FrameConstU32, Hooks},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_mev_shield::{
    Call as MevShieldCall, CurrentKey, Event as MevShieldEvent, KeyHashByBlock, NextKey,
    Submissions,
};
use sp_core::{Pair, sr25519};
use sp_runtime::{
    AccountId32,
    traits::{Hash, SaturatedConversion},
};

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
fn submissions_pruned_after_ttl_window() {
    new_test_ext().execute_with(|| {
        // This must match KEY_EPOCH_HISTORY in the pallet.
        const KEEP: u64 = 100;
        const TOTAL: u64 = KEEP + 5;

        let pair = test_sr25519_pair();
        let who: AccountId32 = pair.public().into();

        // Helper: create a submission at a specific block with a tagged commitment.
        let make_submission = |block: u64, tag: &[u8]| -> TestHash {
            System::set_block_number(block);
            let commitment: TestHash = <Test as frame_system::Config>::Hashing::hash(tag);
            let ciphertext_bytes = vec![block as u8; 4];
            let ciphertext: BoundedVec<u8, FrameConstU32<8192>> =
                BoundedVec::truncate_from(ciphertext_bytes);

            assert_ok!(MevShield::submit_encrypted(
                RuntimeOrigin::signed(who.clone()),
                commitment,
                ciphertext.clone(),
            ));

            <Test as frame_system::Config>::Hashing::hash_of(&(
                who.clone(),
                commitment,
                &ciphertext,
            ))
        };

        // With n = TOTAL and depth = KEEP, prune_before = n - KEEP = 5.
        let stale_block1: u64 = 1; // < 5, should be pruned
        let stale_block2: u64 = 4; // < 5, should be pruned
        let keep_block1: u64 = 5; // == prune_before, should be kept
        let keep_block2: u64 = TOTAL; // latest, should be kept

        let id_stale1 = make_submission(stale_block1, b"stale-1");
        let id_stale2 = make_submission(stale_block2, b"stale-2");
        let id_keep1 = make_submission(keep_block1, b"keep-1");
        let id_keep2 = make_submission(keep_block2, b"keep-2");

        // Sanity: all are present before pruning.
        assert!(Submissions::<Test>::get(id_stale1).is_some());
        assert!(Submissions::<Test>::get(id_stale2).is_some());
        assert!(Submissions::<Test>::get(id_keep1).is_some());
        assert!(Submissions::<Test>::get(id_keep2).is_some());

        // Run on_initialize at block TOTAL, triggering TTL pruning over Submissions.
        let n_final: TestBlockNumber = TOTAL.saturated_into();
        MevShield::on_initialize(n_final);

        // Submissions with submitted_in < prune_before (5) should be gone.
        assert!(Submissions::<Test>::get(id_stale1).is_none());
        assert!(Submissions::<Test>::get(id_stale2).is_none());

        // Submissions at or after prune_before should remain.
        assert!(Submissions::<Test>::get(id_keep1).is_some());
        assert!(Submissions::<Test>::get(id_keep2).is_some());
    });
}

#[test]
fn mark_decryption_failed_removes_submission_and_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(42);
        let pair = test_sr25519_pair();
        let who: AccountId32 = pair.public().into();

        let commitment: TestHash =
            <Test as frame_system::Config>::Hashing::hash(b"failed-decryption-commitment");
        let ciphertext_bytes = vec![5u8; 8];
        let ciphertext: BoundedVec<u8, FrameConstU32<8192>> =
            BoundedVec::truncate_from(ciphertext_bytes.clone());

        assert_ok!(MevShield::submit_encrypted(
            RuntimeOrigin::signed(who.clone()),
            commitment,
            ciphertext.clone(),
        ));

        let id: TestHash = <Test as frame_system::Config>::Hashing::hash_of(&(
            who.clone(),
            commitment,
            &ciphertext,
        ));

        // Sanity: submission exists.
        assert!(Submissions::<Test>::get(id).is_some());

        // Reason we will pass into mark_decryption_failed.
        let reason_bytes = b"AEAD decrypt failed".to_vec();
        let reason: BoundedVec<u8, FrameConstU32<256>> =
            BoundedVec::truncate_from(reason_bytes.clone());

        // Call mark_decryption_failed as unsigned (RuntimeOrigin::none()).
        assert_ok!(MevShield::mark_decryption_failed(
            RuntimeOrigin::none(),
            id,
            reason.clone(),
        ));

        // Submission should be removed.
        assert!(Submissions::<Test>::get(id).is_none());

        // Last event should be DecryptionFailed with the correct id and reason.
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
                    MevShieldEvent::<Test>::DecryptionFailed { id: ev_id, reason: ev_reason }
                )
                if ev_id == id && ev_reason.to_vec() == reason_bytes
            ),
            "expected DecryptionFailed event with correct id & reason"
        );

        // A second call with the same id should now fail with MissingSubmission.
        let res = MevShield::mark_decryption_failed(RuntimeOrigin::none(), id, reason);
        assert_noop!(res, pallet_mev_shield::Error::<Test>::MissingSubmission);
    });
}

#[test]
fn announce_next_key_charges_then_refunds_fee() {
    new_test_ext().execute_with(|| {
        const KYBER_PK_LEN: usize = 1184;

        // ---------------------------------------------------------------------
        // 1. Seed Aura authorities with a single validator and derive account.
        // ---------------------------------------------------------------------
        let validator_pair = test_sr25519_pair();
        let validator_account: AccountId32 = validator_pair.public().into();
        let validator_aura_id: <Test as pallet_aura::Config>::AuthorityId =
            validator_pair.public().into();

        let authorities: BoundedVec<
            <Test as pallet_aura::Config>::AuthorityId,
            <Test as pallet_aura::Config>::MaxAuthorities,
        > = BoundedVec::truncate_from(vec![validator_aura_id]);
        pallet_aura::Authorities::<Test>::put(authorities);

        // ---------------------------------------------------------------------
        // 2. Build a valid Kyber public key and the corresponding RuntimeCall.
        // ---------------------------------------------------------------------
        let pk_bytes = vec![42u8; KYBER_PK_LEN];
        let bounded_pk: BoundedVec<u8, FrameConstU32<2048>> =
            BoundedVec::truncate_from(pk_bytes.clone());

        let runtime_call = RuntimeCall::MevShield(MevShieldCall::<Test>::announce_next_key {
            public_key: bounded_pk.clone(),
        });

        // ---------------------------------------------------------------------
        // 3. Pre-dispatch: DispatchInfo must say Pays::Yes.
        // ---------------------------------------------------------------------
        let pre_info = <RuntimeCall as frame_support::dispatch::GetDispatchInfo>::get_dispatch_info(
            &runtime_call,
        );

        assert_eq!(
            pre_info.pays_fee,
            frame_support::dispatch::Pays::Yes,
            "announce_next_key must be declared as fee-paying at pre-dispatch"
        );

        // ---------------------------------------------------------------------
        // 4. Dispatch via the pallet function.
        // ---------------------------------------------------------------------
        let post = MevShield::announce_next_key(
            RuntimeOrigin::signed(validator_account.clone()),
            bounded_pk.clone(),
        )
        .expect("announce_next_key should succeed for an Aura validator");

        // Post-dispatch info should switch pays_fee from Yes -> No (refund).
        assert_eq!(
            post.pays_fee,
            frame_support::dispatch::Pays::No,
            "announce_next_key must refund the previously chargeable fee"
        );

        // And we don't override the actual weight (None => use pre-dispatch weight).
        assert!(
            post.actual_weight.is_none(),
            "announce_next_key should not override actual_weight in PostDispatchInfo"
        );
        let next = NextKey::<Test>::get().expect("NextKey should be set by announce_next_key");
        assert_eq!(next, pk_bytes);
    });
}
