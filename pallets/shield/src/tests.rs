use crate::mock::*;
use crate::{AuthorKeys, CurrentKey, Error, NextKey};

use codec::Encode;
use frame_support::{BoundedVec, assert_noop, assert_ok};
use sp_runtime::BuildStorage;
use sp_runtime::testing::TestSignature;
use sp_runtime::traits::{Block as BlockT, Hash};
use stp_shield::{ShieldKeystore, ShieldKeystoreExt, ShieldPublicKey, ShieldedTransaction};

use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use ml_kem::{
    EncodedSizeUser, MlKem768Params,
    kem::{Encapsulate, EncapsulationKey},
};
use rand::rngs::OsRng;
use stc_shield::MemoryShieldKeystore;
use std::sync::Arc;

#[test]
fn announce_rejects_signed_origin() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), None);
        assert_noop!(
            MevShield::announce_next_key(RuntimeOrigin::signed(1), Some(valid_pk())),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn announce_shifts_next_into_current() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), Some(2));

        let old_next = valid_pk_b();
        NextKey::<Test>::put(old_next.clone());

        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(valid_pk()),
        ));

        assert_eq!(CurrentKey::<Test>::get(), Some(old_next));
    });
}

#[test]
fn announce_stores_key_in_author_keys() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), None);
        let pk = valid_pk();

        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(pk.clone()),
        ));

        assert_eq!(AuthorKeys::<Test>::get(1u64), Some(pk));
    });
}

#[test]
fn announce_sets_next_key_from_next_author() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), Some(2));

        let pk_b = valid_pk_b();
        AuthorKeys::<Test>::insert(2u64, pk_b.clone());

        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(valid_pk()),
        ));

        assert_eq!(NextKey::<Test>::get(), Some(pk_b));
    });
}

#[test]
fn announce_next_key_none_when_next_author_has_no_key() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), Some(2));

        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(valid_pk()),
        ));

        assert!(NextKey::<Test>::get().is_none());
    });
}

#[test]
fn announce_next_key_none_when_no_next_author() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), None);

        assert_ok!(MevShield::announce_next_key(
            RuntimeOrigin::none(),
            Some(valid_pk()),
        ));

        assert!(NextKey::<Test>::get().is_none());
    });
}

#[test]
fn announce_rejects_bad_pk_length() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), None);
        let bad_pk: ShieldPublicKey = BoundedVec::truncate_from(vec![0x01; 100]);

        assert_noop!(
            MevShield::announce_next_key(RuntimeOrigin::none(), Some(bad_pk)),
            Error::<Test>::BadPublicKeyLen
        );
    });
}

#[test]
fn announce_none_pk_removes_author_key() {
    new_test_ext().execute_with(|| {
        set_authors(Some(1), None);
        AuthorKeys::<Test>::insert(1u64, valid_pk());

        assert_ok!(MevShield::announce_next_key(RuntimeOrigin::none(), None));

        assert!(AuthorKeys::<Test>::get(1u64).is_none());
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
    let keystore = Arc::new(MemoryShieldKeystore::new());

    // Client side: read the announced public key and encapsulate.
    let pk_bytes = keystore.next_public_key().unwrap();
    let enc_key =
        EncapsulationKey::<MlKem768Params>::from_bytes(pk_bytes.as_slice().try_into().unwrap());
    let (kem_ct, shared_secret) = enc_key.encapsulate(&mut OsRng).unwrap();

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

    let shielded_tx = ShieldedTransaction {
        key_hash: [0u8; 16],
        kem_ct: kem_ct.as_slice().to_vec(),
        nonce,
        aead_ct,
    };

    // Build externalities with ShieldKeystoreExt registered.
    let storage = RuntimeGenesisConfig::default()
        .build_storage()
        .expect("valid genesis");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.register_extension(ShieldKeystoreExt::from(
        keystore as Arc<dyn stp_shield::ShieldKeystore>,
    ));

    ext.execute_with(|| {
        let result = crate::Pallet::<Test>::try_unshield_tx::<Block>(shielded_tx);
        assert!(result.is_some());

        let decoded = result.unwrap();
        assert_eq!(decoded.encode(), inner_uxt.encode());
    });
}
