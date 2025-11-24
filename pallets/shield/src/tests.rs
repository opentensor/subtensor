#![cfg(test)]

use crate as pallet_mev_shield;
use crate::mock::*;

use codec::Encode;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_support::traits::ConstU32 as FrameConstU32;
use sp_core::sr25519;
use sp_runtime::{
    traits::{SaturatedConversion, Zero},
    transaction_validity::TransactionSource,
    AccountId32, MultiSignature,
};
use frame_support::pallet_prelude::ValidateUnsigned;
use sp_runtime::traits::Hash;
use sp_core::Pair;
use pallet_mev_shield::{
    Call as MevShieldCall,
    CurrentKey,
    Epoch,
    Event as MevShieldEvent,
    NextKey,
    Submissions,
};
use frame_support::traits::Hooks;

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

/// Deterministic sr25519 pair for tests (acts as "Alice").
fn test_sr25519_pair() -> sr25519::Pair {
    sr25519::Pair::from_seed(&[1u8; 32])
}

/// Reproduce the pallet's raw payload layout:
///   signer (32B) || nonce (u32 LE) || SCALE(call)
fn build_raw_payload_bytes_for_test(
    signer: &AccountId32,
    nonce: TestNonce,
    call: &RuntimeCall,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(signer.as_ref());

    let n_u32: u32 = nonce.saturated_into();
    out.extend_from_slice(&n_u32.to_le_bytes());

    out.extend(call.encode());
    out
}

#[test]
fn authority_can_announce_next_key_and_on_initialize_rolls_it() {
    new_test_ext().execute_with(|| {
        let epoch: u64 = 42;
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

        assert_eq!(Epoch::<Test>::get(), 0);
        assert!(CurrentKey::<Test>::get().is_none());
        assert!(NextKey::<Test>::get().is_none());

        // Signed by an Aura validator -> passes TestAuthorityOrigin::ensure_validator.
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::signed(validator_account.clone()),
            bounded_pk.clone(),
            epoch
        ));

        // NextKey storage updated
        let next = NextKey::<Test>::get().expect("NextKey should be set");
        assert_eq!(next.epoch, epoch);
        assert_eq!(next.public_key.to_vec(), pk_bytes);

        // Roll on new block
        MevShield::on_initialize(2);

        let curr = CurrentKey::<Test>::get().expect("CurrentKey should be set");
        assert_eq!(curr.epoch, epoch);
        assert_eq!(curr.public_key.to_vec(), pk_bytes);

        assert_eq!(Epoch::<Test>::get(), epoch);
        assert!(NextKey::<Test>::get().is_none());
    });
}


#[test]
fn announce_next_key_rejects_non_validator_origins() {
    new_test_ext().execute_with(|| {
        const KYBER_PK_LEN: usize = 1184;
        let epoch: u64 = 7;

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
                epoch,
            ),
            sp_runtime::DispatchError::BadOrigin
        );

        // 2) Unsigned origin must also fail with BadOrigin.
        assert_noop!(
            MevShield::announce_next_key(
                RuntimeOrigin::none(),
                bounded_pk.clone(),
                epoch,
            ),
            sp_runtime::DispatchError::BadOrigin
        );

        // 3) Signed validator origin succeeds (sanity check).
        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::signed(validator_account.clone()),
            bounded_pk.clone(),
            epoch
        ));

        let next = NextKey::<Test>::get().expect("NextKey must be set by validator");
        assert_eq!(next.epoch, epoch);
        assert_eq!(next.public_key.to_vec(), pk_bytes);
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

        let nonce: TestNonce = Zero::zero();
        assert_eq!(System::account_nonce(&signer), nonce);

        let payload_bytes = build_raw_payload_bytes_for_test(&signer, nonce, &inner_call);

        let commitment =
            <Test as frame_system::Config>::Hashing::hash(payload_bytes.as_ref());

        let ciphertext_bytes = vec![9u8, 9, 9, 9];
        let ciphertext: BoundedVec<u8, FrameConstU32<8192>> =
            BoundedVec::truncate_from(ciphertext_bytes.clone());

        System::set_block_number(1);

        // Wrapper author == signer for simplest path
        assert_ok!(MevShield::submit_encrypted(
            RuntimeOrigin::signed(signer.clone()),
            commitment,
            ciphertext.clone(),
        ));

        let id = <Test as frame_system::Config>::Hashing::hash_of(&(
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
            nonce,
            Box::new(inner_call.clone()),
            signature,
        );

        assert_ok!(result);

        // Submission consumed
        assert!(Submissions::<Test>::get(id).is_none());

        // Nonce bumped once
        let expected_nonce: TestNonce = (1u32).saturated_into();
        assert_eq!(System::account_nonce(&signer), expected_nonce);

        // Last event is DecryptedExecuted
        let events = System::events();
        let last = events.last().expect("an event should be emitted").event.clone();

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
fn validate_unsigned_accepts_local_source_for_execute_revealed() {
    new_test_ext().execute_with(|| {
        let pair = test_sr25519_pair();
        let signer: AccountId32 = pair.public().into();
        let nonce: TestNonce = Zero::zero();

        let inner_call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: b"noop-local".to_vec(),
        });

        let id = <Test as frame_system::Config>::Hashing::hash(b"mevshield-id-local");
        let signature: MultiSignature =
            sr25519::Signature::from_raw([0u8; 64]).into();

        let call = MevShieldCall::<Test>::execute_revealed {
            id,
            signer,
            nonce,
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
        let nonce: TestNonce = Zero::zero();

        let inner_call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: b"noop-inblock".to_vec(),
        });

        let id = <Test as frame_system::Config>::Hashing::hash(b"mevshield-id-inblock");
        let signature: MultiSignature =
            sr25519::Signature::from_raw([1u8; 64]).into();

        let call = MevShieldCall::<Test>::execute_revealed {
            id,
            signer,
            nonce,
            call: Box::new(inner_call),
            signature,
        };

        let validity = MevShield::validate_unsigned(TransactionSource::InBlock, &call);
        assert_ok!(validity);
    });
}
