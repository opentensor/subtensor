use codec::Encode;
use sp_std::prelude::*;

#[cfg(test)]
use crate::{
    CommitmentInfo, CommitmentOf, Config, Data, Error, Event, MaxSpace, Pallet, RateLimit,
    Registration, RevealedCommitments, TimelockedIndex,
    mock::{
        Balances, DRAND_QUICKNET_SIG_HEX, RuntimeEvent, RuntimeOrigin, Test, insert_drand_pulse,
        new_test_ext, produce_ciphertext,
    },
};
use frame_support::pallet_prelude::Hooks;
use frame_support::{
    BoundedVec, assert_noop, assert_ok,
    traits::{Currency, Get, ReservableCurrency},
};
use frame_system::Pallet as System;

#[allow(clippy::indexing_slicing)]
#[test]
fn manual_data_type_info() {
    let mut registry = scale_info::Registry::new();
    let type_id = registry.register_type(&scale_info::meta_type::<Data>());
    let registry: scale_info::PortableRegistry = registry.into();
    let type_info = registry.resolve(type_id.id).expect("Expected not to panic");

    let check_type_info = |data: &Data| {
        let variant_name = match data {
            Data::None => "None".to_string(),
            Data::BlakeTwo256(_) => "BlakeTwo256".to_string(),
            Data::Sha256(_) => "Sha256".to_string(),
            Data::Keccak256(_) => "Keccak256".to_string(),
            Data::ShaThree256(_) => "ShaThree256".to_string(),
            Data::Raw(bytes) => format!("Raw{}", bytes.len()),
            Data::TimelockEncrypted { .. } => "TimelockEncrypted".to_string(),
        };
        if let scale_info::TypeDef::Variant(variant) = &type_info.type_def {
            let variant = variant
                .variants
                .iter()
                .find(|v| v.name == variant_name)
                .unwrap_or_else(|| panic!("Expected to find variant {}", variant_name));

            let encoded = data.encode();
            assert_eq!(encoded[0], variant.index);

            // For variants with fields, check the encoded length matches expected field lengths
            if !variant.fields.is_empty() {
                let expected_len = match data {
                    Data::None => 0,
                    Data::Raw(bytes) => bytes.len() as u32,
                    Data::BlakeTwo256(_)
                    | Data::Sha256(_)
                    | Data::Keccak256(_)
                    | Data::ShaThree256(_) => 32,
                    Data::TimelockEncrypted {
                        encrypted,
                        reveal_round,
                    } => {
                        // Calculate length: encrypted (length prefixed) + reveal_round (u64)
                        let encrypted_len = encrypted.encode().len() as u32; // Includes length prefix
                        let reveal_round_len = reveal_round.encode().len() as u32; // Typically 8 bytes
                        encrypted_len + reveal_round_len
                    }
                };
                assert_eq!(
                    encoded.len() as u32 - 1, // Subtract variant byte
                    expected_len,
                    "Encoded length mismatch for variant {}",
                    variant_name
                );
            } else {
                assert_eq!(
                    encoded.len() as u32 - 1,
                    0,
                    "Expected no fields for {}",
                    variant_name
                );
            }
        } else {
            panic!("Should be a variant type");
        }
    };

    let mut data = vec![
        Data::None,
        Data::BlakeTwo256(Default::default()),
        Data::Sha256(Default::default()),
        Data::Keccak256(Default::default()),
        Data::ShaThree256(Default::default()),
    ];

    // Add Raw instances for all possible sizes
    for n in 0..128 {
        data.push(Data::Raw(
            vec![0u8; n as usize]
                .try_into()
                .expect("Expected not to panic"),
        ));
    }

    // Add a TimelockEncrypted instance
    data.push(Data::TimelockEncrypted {
        encrypted: vec![0u8; 64].try_into().expect("Expected not to panic"),
        reveal_round: 12345,
    });

    for d in data.iter() {
        check_type_info(d);
    }
}

#[test]
fn set_commitment_works() {
    new_test_ext().execute_with(|| {
        System::<Test>::set_block_number(1);
        let info = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![]).expect("Expected not to panic"),
        });

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(1),
            1,
            info.clone()
        ));

        let commitment = Pallet::<Test>::commitment_of(1, 1).expect("Expected not to panic");
        let initial_deposit: u64 = <Test as Config>::InitialDeposit::get();
        assert_eq!(commitment.deposit, initial_deposit);
        assert_eq!(commitment.block, 1);
        assert_eq!(Pallet::<Test>::last_commitment(1, 1), Some(1));
    });
}

#[test]
#[should_panic(expected = "BoundedVec::try_from failed")]
fn set_commitment_too_many_fields_panics() {
    new_test_ext().execute_with(|| {
        let max_fields: u32 = <Test as Config>::MaxFields::get();
        let fields = vec![Data::None; (max_fields + 1) as usize];

        // This line will panic when 'BoundedVec::try_from(...)' sees too many items.
        let info = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(fields).expect("BoundedVec::try_from failed"),
        });

        // We never get here, because the constructor panics above.
        let _ = Pallet::<Test>::set_commitment(frame_system::RawOrigin::Signed(1).into(), 1, info);
    });
}

// DEPRECATED
// #[test]
// fn set_commitment_rate_limit_exceeded() {
//     new_test_ext().execute_with(|| {
//         let rate_limit = <Test as Config>::DefaultRateLimit::get();
//         System::<Test>::set_block_number(1);
//         let info = Box::new(CommitmentInfo {
//             fields: BoundedVec::try_from(vec![]).expect("Expected not to panic"),
//         });

//         assert_ok!(Pallet::<Test>::set_commitment(
//             RuntimeOrigin::signed(1),
//             1,
//             info.clone()
//         ));

//         // Set block number to just before rate limit expires
//         System::<Test>::set_block_number(rate_limit);
//         assert_noop!(
//             Pallet::<Test>::set_commitment(RuntimeOrigin::signed(1), 1, info.clone()),
//             Error::<Test>::CommitmentSetRateLimitExceeded
//         );

//         // Set block number to after rate limit
//         System::<Test>::set_block_number(rate_limit + 1);
//         assert_ok!(Pallet::<Test>::set_commitment(
//             RuntimeOrigin::signed(1),
//             1,
//             info
//         ));
//     });
// }

#[test]
fn set_commitment_updates_deposit() {
    new_test_ext().execute_with(|| {
        System::<Test>::set_block_number(1);
        let info1 = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Default::default(); 2])
                .expect("Expected not to panic"),
        });
        let info2 = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Default::default(); 3])
                .expect("Expected not to panic"),
        });

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(1),
            1,
            info1
        ));
        let initial_deposit: u64 = <Test as Config>::InitialDeposit::get();
        let field_deposit: u64 = <Test as Config>::FieldDeposit::get();
        let expected_deposit1: u64 = initial_deposit + 2u64 * field_deposit;
        assert_eq!(
            Pallet::<Test>::commitment_of(1, 1)
                .expect("Expected not to panic")
                .deposit,
            expected_deposit1
        );

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(1),
            1,
            info2
        ));
        let expected_deposit2: u64 = initial_deposit + 3u64 * field_deposit;
        assert_eq!(
            Pallet::<Test>::commitment_of(1, 1)
                .expect("Expected not to panic")
                .deposit,
            expected_deposit2
        );
    });
}

#[test]
fn set_rate_limit_works() {
    new_test_ext().execute_with(|| {
        let default_rate_limit: u64 = <Test as Config>::DefaultRateLimit::get();
        assert_eq!(RateLimit::<Test>::get(), default_rate_limit);

        assert_ok!(Pallet::<Test>::set_rate_limit(RuntimeOrigin::root(), 200));
        assert_eq!(RateLimit::<Test>::get(), 200);

        assert_noop!(
            Pallet::<Test>::set_rate_limit(RuntimeOrigin::signed(1), 300),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn event_emission_works() {
    new_test_ext().execute_with(|| {
        System::<Test>::set_block_number(1);
        let info = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![]).expect("Expected not to panic"),
        });

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(1),
            1,
            info
        ));

        let events = System::<Test>::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Commitments(Event::Commitment { netuid: 1, who: 1 })
        )));
    });
}

#[allow(clippy::indexing_slicing)]
#[test]
fn happy_path_timelock_commitments() {
    new_test_ext().execute_with(|| {
        let message_text = b"Hello timelock only!";
        let data_raw = Data::Raw(
            message_text
                .to_vec()
                .try_into()
                .expect("<= 128 bytes for Raw variant"),
        );
        let fields_vec = vec![data_raw];
        let fields_bounded: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(fields_vec).expect("Too many fields");

        let inner_info: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo {
            fields: fields_bounded,
        };

        let plaintext = inner_info.encode();

        let reveal_round = 1000;
        let encrypted = produce_ciphertext(&plaintext, reveal_round);

        let data = Data::TimelockEncrypted {
            encrypted: encrypted.clone(),
            reveal_round,
        };

        let fields_outer: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![data]).expect("Too many fields");
        let info_outer = CommitmentInfo {
            fields: fields_outer,
        };

        let who = 123;
        let netuid = 42;
        System::<Test>::set_block_number(1);

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            Box::new(info_outer)
        ));

        let drand_signature_bytes =
            hex::decode(DRAND_QUICKNET_SIG_HEX).expect("Expected not to panic");
        insert_drand_pulse(reveal_round, &drand_signature_bytes);

        System::<Test>::set_block_number(9999);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        let revealed =
            RevealedCommitments::<Test>::get(netuid, who).expect("Should have revealed data");

        let revealed_inner = &revealed.info;
        assert_eq!(revealed_inner.fields.len(), 1);
        match &revealed_inner.fields[0] {
            Data::Raw(bounded_bytes) => {
                assert_eq!(
                    bounded_bytes.as_slice(),
                    message_text,
                    "Decrypted text from on-chain storage must match the original message"
                );
            }
            other => panic!("Expected Data::Raw(...) in revealed, got {:?}", other),
        }
    });
}

#[test]
fn reveal_timelocked_commitment_missing_round_does_nothing() {
    new_test_ext().execute_with(|| {
        let who = 1;
        let netuid = 2;
        System::<Test>::set_block_number(5);
        let ciphertext = produce_ciphertext(b"My plaintext", 1000);
        let data = Data::TimelockEncrypted {
            encrypted: ciphertext,
            reveal_round: 1000,
        };
        let fields: BoundedVec<_, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![data]).expect("Expected not to panic");
        let info = CommitmentInfo { fields };
        let origin = RuntimeOrigin::signed(who);
        assert_ok!(Pallet::<Test>::set_commitment(
            origin,
            netuid,
            Box::new(info)
        ));
        System::<Test>::set_block_number(100_000);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());
        assert!(RevealedCommitments::<Test>::get(netuid, who).is_none());
    });
}

#[allow(clippy::indexing_slicing)]
#[test]
fn reveal_timelocked_commitment_cant_deserialize_ciphertext() {
    new_test_ext().execute_with(|| {
        let who = 42;
        let netuid = 9;
        System::<Test>::set_block_number(10);
        let good_ct = produce_ciphertext(b"Some data", 1000);
        let mut corrupted = good_ct.into_inner();
        if !corrupted.is_empty() {
            corrupted[0] = 0xFF;
        }
        let corrupted_ct = BoundedVec::try_from(corrupted).expect("Expected not to panic");
        let data = Data::TimelockEncrypted {
            encrypted: corrupted_ct,
            reveal_round: 1000,
        };
        let fields = BoundedVec::try_from(vec![data]).expect("Expected not to panic");
        let info = CommitmentInfo { fields };
        let origin = RuntimeOrigin::signed(who);
        assert_ok!(Pallet::<Test>::set_commitment(
            origin,
            netuid,
            Box::new(info)
        ));
        let sig_bytes = hex::decode(DRAND_QUICKNET_SIG_HEX).expect("Expected not to panic");
        insert_drand_pulse(1000, &sig_bytes);
        System::<Test>::set_block_number(99999);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());
        assert!(RevealedCommitments::<Test>::get(netuid, who).is_none());
    });
}

#[test]
fn reveal_timelocked_commitment_bad_signature_skips_decryption() {
    new_test_ext().execute_with(|| {
        let who = 10;
        let netuid = 11;
        System::<Test>::set_block_number(15);
        let real_ct = produce_ciphertext(b"A valid plaintext", 1000);
        let data = Data::TimelockEncrypted {
            encrypted: real_ct,
            reveal_round: 1000,
        };
        let fields: BoundedVec<_, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![data]).expect("Expected not to panic");
        let info = CommitmentInfo { fields };
        let origin = RuntimeOrigin::signed(who);
        assert_ok!(Pallet::<Test>::set_commitment(
            origin,
            netuid,
            Box::new(info)
        ));
        let bad_signature = [0x33u8; 10];
        insert_drand_pulse(1000, &bad_signature);
        System::<Test>::set_block_number(10_000);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());
        assert!(RevealedCommitments::<Test>::get(netuid, who).is_none());
    });
}

#[test]
fn reveal_timelocked_commitment_empty_decrypted_data_is_skipped() {
    new_test_ext().execute_with(|| {
        let who = 2;
        let netuid = 3;
        let commit_block = 100u64;
        System::<Test>::set_block_number(commit_block);
        let reveal_round = 1000;
        let empty_ct = produce_ciphertext(&[], reveal_round);
        let data = Data::TimelockEncrypted {
            encrypted: empty_ct,
            reveal_round,
        };
        let fields = BoundedVec::try_from(vec![data]).expect("Expected not to panic");
        let info = CommitmentInfo { fields };
        let origin = RuntimeOrigin::signed(who);
        assert_ok!(Pallet::<Test>::set_commitment(
            origin,
            netuid,
            Box::new(info)
        ));
        let sig_bytes = hex::decode(DRAND_QUICKNET_SIG_HEX).expect("Expected not to panic");
        insert_drand_pulse(reveal_round, &sig_bytes);
        System::<Test>::set_block_number(10_000);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());
        assert!(RevealedCommitments::<Test>::get(netuid, who).is_none());
    });
}

#[test]
fn reveal_timelocked_commitment_decode_failure_is_skipped() {
    new_test_ext().execute_with(|| {
        let who = 999;
        let netuid = 8;
        let commit_block = 42u64;
        System::<Test>::set_block_number(commit_block);
        let plaintext = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
        let reveal_round = 1000;
        let real_ct = produce_ciphertext(&plaintext, reveal_round);
        let data = Data::TimelockEncrypted {
            encrypted: real_ct,
            reveal_round,
        };
        let fields = BoundedVec::try_from(vec![data]).expect("Expected not to panic");
        let info = CommitmentInfo { fields };
        let origin = RuntimeOrigin::signed(who);
        assert_ok!(Pallet::<Test>::set_commitment(
            origin,
            netuid,
            Box::new(info)
        ));
        let sig_bytes =
            hex::decode(DRAND_QUICKNET_SIG_HEX.as_bytes()).expect("Expected not to panic");
        insert_drand_pulse(reveal_round, &sig_bytes);
        System::<Test>::set_block_number(9999);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());
        assert!(RevealedCommitments::<Test>::get(netuid, who).is_none());
    });
}

#[test]
fn reveal_timelocked_commitment_single_field_entry_is_removed_after_reveal() {
    new_test_ext().execute_with(|| {
        let message_text = b"Single field timelock test!";
        let data_raw = Data::Raw(
            message_text
                .to_vec()
                .try_into()
                .expect("Message must be <=128 bytes for Raw variant"),
        );

        let fields_bounded: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![data_raw]).expect("BoundedVec creation must not fail");

        let inner_info: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo {
            fields: fields_bounded,
        };

        let plaintext = inner_info.encode();
        let reveal_round = 1000;
        let encrypted = produce_ciphertext(&plaintext, reveal_round);

        let timelock_data = Data::TimelockEncrypted {
            encrypted,
            reveal_round,
        };
        let fields_outer: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![timelock_data]).expect("Too many fields");
        let info_outer: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo {
            fields: fields_outer,
        };

        let who = 555;
        let netuid = 777;
        System::<Test>::set_block_number(1);
        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            Box::new(info_outer)
        ));

        let drand_signature_bytes = hex::decode(DRAND_QUICKNET_SIG_HEX)
            .expect("Must decode DRAND_QUICKNET_SIG_HEX successfully");
        insert_drand_pulse(reveal_round, &drand_signature_bytes);

        System::<Test>::set_block_number(9999);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        let revealed =
            RevealedCommitments::<Test>::get(netuid, who).expect("Expected to find revealed data");
        assert_eq!(
            revealed.info.fields.len(),
            1,
            "Should have exactly 1 revealed field"
        );

        assert!(
            crate::CommitmentOf::<Test>::get(netuid, who).is_none(),
            "Expected CommitmentOf<T> entry to be removed after reveal"
        );
    });
}

#[allow(clippy::indexing_slicing)]
#[test]
fn reveal_timelocked_multiple_fields_only_correct_ones_removed() {
    new_test_ext().execute_with(|| {
        let round_1000 = 1000;

        // 2) Build two CommitmentInfos, one for each timelock
        let msg_1 = b"Hello from TLE #1";
        let inner_1_fields: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![Data::Raw(
                msg_1.to_vec().try_into().expect("expected not to panic"),
            )])
            .expect("BoundedVec of size 1");
        let inner_info_1 = CommitmentInfo {
            fields: inner_1_fields,
        };
        let encoded_1 = inner_info_1.encode();
        let ciphertext_1 = produce_ciphertext(&encoded_1, round_1000);
        let timelock_1 = Data::TimelockEncrypted {
            encrypted: ciphertext_1,
            reveal_round: round_1000,
        };

        let msg_2 = b"Hello from TLE #2";
        let inner_2_fields: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![Data::Raw(
                msg_2.to_vec().try_into().expect("expected not to panic"),
            )])
            .expect("BoundedVec of size 1");
        let inner_info_2 = CommitmentInfo {
            fields: inner_2_fields,
        };
        let encoded_2 = inner_info_2.encode();
        let ciphertext_2 = produce_ciphertext(&encoded_2, round_1000);
        let timelock_2 = Data::TimelockEncrypted {
            encrypted: ciphertext_2,
            reveal_round: round_1000,
        };

        // 3) One plain Data::Raw field (non-timelocked)
        let raw_bytes = b"Plain non-timelocked data";
        let data_raw = Data::Raw(
            raw_bytes
                .to_vec()
                .try_into()
                .expect("expected not to panic"),
        );

        // 4) Outer commitment: 3 fields total => [Raw, TLE #1, TLE #2]
        let outer_fields = BoundedVec::try_from(vec![
            data_raw.clone(),
            timelock_1.clone(),
            timelock_2.clone(),
        ])
        .expect("T::MaxFields >= 3 in the test config, or at least 3 here");
        let outer_info = CommitmentInfo {
            fields: outer_fields,
        };

        // 5) Insert the commitment
        let who = 123;
        let netuid = 999;
        System::<Test>::set_block_number(1);
        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            Box::new(outer_info)
        ));
        let initial = Pallet::<Test>::commitment_of(netuid, who).expect("Must exist");
        assert_eq!(initial.info.fields.len(), 3, "3 fields inserted");

        // 6) Insert Drand signature for round=1000
        let drand_sig_1000 = hex::decode(DRAND_QUICKNET_SIG_HEX).expect("decode DRAND sig");
        insert_drand_pulse(round_1000, &drand_sig_1000);

        // 7) Reveal once
        System::<Test>::set_block_number(50);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        // => The pallet code has removed *both* TLE #1 and TLE #2 in this single call!
        let after_reveal = Pallet::<Test>::commitment_of(netuid, who)
            .expect("Should still exist with leftover fields");
        // Only the raw, non-timelocked field remains
        assert_eq!(
            after_reveal.info.fields.len(),
            1,
            "Both timelocks referencing round=1000 got removed at once"
        );
        assert_eq!(
            after_reveal.info.fields[0], data_raw,
            "Only the raw field is left"
        );

        // 8) Check revealed data
        let revealed_data = RevealedCommitments::<Test>::get(netuid, who)
            .expect("Expected revealed data for TLE #1 and #2");

        assert_eq!(
            revealed_data.info.fields.len(),
            2,
            "We revealed both TLE #1 and TLE #2 in the same pass"
        );
        let mut found_msg1 = false;
        let mut found_msg2 = false;
        for item in &revealed_data.info.fields {
            if let Data::Raw(bytes) = item {
                if bytes.as_slice() == msg_1 {
                    found_msg1 = true;
                } else if bytes.as_slice() == msg_2 {
                    found_msg2 = true;
                }
            }
        }
        assert!(
            found_msg1 && found_msg2,
            "Should see both TLE #1 and TLE #2 in the revealed data"
        );

        // 9) A second reveal call now does nothing, because no timelocks remain
        System::<Test>::set_block_number(51);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        let after_second = Pallet::<Test>::commitment_of(netuid, who).expect("Still must exist");
        assert_eq!(
            after_second.info.fields.len(),
            1,
            "No new fields were removed, because no timelocks remain"
        );
    });
}

#[test]
fn test_index_lifecycle_no_timelocks_updates_in_out() {
    new_test_ext().execute_with(|| {
        let netuid = 100;
        let who = 999;

        //
        // A) Create a commitment with **no** timelocks => shouldn't be in index
        //
        let no_tl_fields: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![]).expect("Empty is ok");
        let info_no_tl = CommitmentInfo {
            fields: no_tl_fields,
        };
        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            Box::new(info_no_tl)
        ));
        assert!(
            !TimelockedIndex::<Test>::get().contains(&(netuid, who)),
            "User with no timelocks must not appear in index"
        );

        //
        // B) Update the commitment to have a timelock => enters index
        //
        let tl_fields: BoundedVec<_, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![Data::TimelockEncrypted {
                encrypted: Default::default(),
                reveal_round: 1234,
            }])
            .expect("Expected success");
        let info_with_tl = CommitmentInfo { fields: tl_fields };
        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            Box::new(info_with_tl)
        ));
        assert!(
            TimelockedIndex::<Test>::get().contains(&(netuid, who)),
            "User must appear in index after adding a timelock"
        );

        //
        // C) Remove the timelock => leaves index
        //
        let back_to_no_tl: BoundedVec<_, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![]).expect("Expected success");
        let info_remove_tl = CommitmentInfo {
            fields: back_to_no_tl,
        };
        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            Box::new(info_remove_tl)
        ));

        assert!(
            !TimelockedIndex::<Test>::get().contains(&(netuid, who)),
            "User must be removed from index after losing all timelocks"
        );
    });
}

#[test]
fn two_timelocks_partial_then_full_reveal() {
    new_test_ext().execute_with(|| {
        let netuid_a = 1;
        let who_a = 10;
        let round_1000 = 1000;
        let round_2000 = 2000;

        let drand_sig_1000 = hex::decode(DRAND_QUICKNET_SIG_HEX).expect("Expected success");
        insert_drand_pulse(round_1000, &drand_sig_1000);

        let drand_sig_2000_hex =
            "b6cb8f482a0b15d45936a4c4ea08e98a087e71787caee3f4d07a8a9843b1bc5423c6b3c22f446488b3137eaca799c77e";

        //
        // First Timelock => round=1000
        //
        let msg_a1 = b"UserA timelock #1 (round=1000)";
        let inner_1_fields: BoundedVec<Data, <Test as Config>::MaxFields> = BoundedVec::try_from(
            vec![Data::Raw(msg_a1.to_vec().try_into().expect("Expected success"))],
        )
        .expect("MaxFields >= 1");
        let inner_info_1: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo {
            fields: inner_1_fields,
        };
        let encoded_1 = inner_info_1.encode();
        let ciphertext_1 = produce_ciphertext(&encoded_1, round_1000);
        let tle_a1 = Data::TimelockEncrypted {
            encrypted: ciphertext_1,
            reveal_round: round_1000,
        };

        //
        // Second Timelock => round=2000
        //
        let msg_a2 = b"UserA timelock #2 (round=2000)";
        let inner_2_fields: BoundedVec<Data, <Test as Config>::MaxFields> = BoundedVec::try_from(
            vec![Data::Raw(msg_a2.to_vec().try_into().expect("Expected success"))],
        )
        .expect("MaxFields >= 1");
        let inner_info_2: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo {
            fields: inner_2_fields,
        };
        let encoded_2 = inner_info_2.encode();
        let ciphertext_2 = produce_ciphertext(&encoded_2, round_2000);
        let tle_a2 = Data::TimelockEncrypted {
            encrypted: ciphertext_2,
            reveal_round: round_2000,
        };

        //
        // Insert outer commitment with both timelocks
        //
        let fields_a: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![tle_a1, tle_a2]).expect("2 fields, must be <= MaxFields");
        let info_a: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo { fields: fields_a };

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who_a),
            netuid_a,
            Box::new(info_a)
        ));
        assert!(
            TimelockedIndex::<Test>::get().contains(&(netuid_a, who_a)),
            "User A must be in index with 2 timelocks"
        );

        System::<Test>::set_block_number(10);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        let leftover_a1 = CommitmentOf::<Test>::get(netuid_a, who_a).expect("still there");
        assert_eq!(
            leftover_a1.info.fields.len(),
            1,
            "Only the round=1000 timelock removed; round=2000 remains"
        );
        assert!(
            TimelockedIndex::<Test>::get().contains(&(netuid_a, who_a)),
            "Still in index with leftover timelock"
        );

        //
        // Insert signature for round=2000 => final reveal => leftover=none => removed
        //
        let drand_sig_2000 = hex::decode(drand_sig_2000_hex).expect("Expected success");
        insert_drand_pulse(round_2000, &drand_sig_2000);

        System::<Test>::set_block_number(11);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        let leftover_a2 = CommitmentOf::<Test>::get(netuid_a, who_a);
        assert!(
            leftover_a2.is_none(),
            "All timelocks removed => none leftover"
        );
        assert!(
            !TimelockedIndex::<Test>::get().contains(&(netuid_a, who_a)),
            "User A removed from index after final reveal"
        );
    });
}

#[test]
fn single_timelock_reveal_later_round() {
    new_test_ext().execute_with(|| {
        let netuid_b = 2;
        let who_b = 20;
        let round_2000 = 2000;

        let drand_sig_2000_hex =
            "b6cb8f482a0b15d45936a4c4ea08e98a087e71787caee3f4d07a8a9843b1bc5423c6b3c22f446488b3137eaca799c77e";
        let drand_sig_2000 = hex::decode(drand_sig_2000_hex).expect("Expected success");
        insert_drand_pulse(round_2000, &drand_sig_2000);

        let msg_b = b"UserB single timelock (round=2000)";

        let inner_b_fields: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![Data::Raw(msg_b.to_vec().try_into().expect("Expected success"))])
                .expect("MaxFields >= 1");
        let inner_info_b: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo {
            fields: inner_b_fields,
        };
        let encoded_b = inner_info_b.encode();
        let ciphertext_b = produce_ciphertext(&encoded_b, round_2000);
        let tle_b = Data::TimelockEncrypted {
            encrypted: ciphertext_b,
            reveal_round: round_2000,
        };

        let fields_b: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![tle_b]).expect("1 field");
        let info_b: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo { fields: fields_b };

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who_b),
            netuid_b,
            Box::new(info_b)
        ));
        assert!(
            TimelockedIndex::<Test>::get().contains(&(netuid_b, who_b)),
            "User B in index"
        );

        // Remove the round=2000 signature so first reveal does nothing
        pallet_drand::Pulses::<Test>::remove(round_2000);

        System::<Test>::set_block_number(20);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        let leftover_b1 = CommitmentOf::<Test>::get(netuid_b, who_b).expect("still there");
        assert_eq!(
            leftover_b1.info.fields.len(),
            1,
            "No signature => timelock remains"
        );
        assert!(
            TimelockedIndex::<Test>::get().contains(&(netuid_b, who_b)),
            "Still in index with leftover timelock"
        );

        insert_drand_pulse(round_2000, &drand_sig_2000);

        System::<Test>::set_block_number(21);
        assert_ok!(Pallet::<Test>::reveal_timelocked_commitments());

        let leftover_b2 = CommitmentOf::<Test>::get(netuid_b, who_b);
        assert!(leftover_b2.is_none(), "Timelock removed => leftover=none");
        assert!(
            !TimelockedIndex::<Test>::get().contains(&(netuid_b, who_b)),
            "User B removed from index after final reveal"
        );
    });
}

#[test]
fn tempo_based_space_limit_accumulates_in_same_window() {
    new_test_ext().execute_with(|| {
        let netuid = 1;
        let who = 100;
        let space_limit = 50;
        MaxSpace::<Test>::set(space_limit);
        System::<Test>::set_block_number(0);

        // A single commitment that uses some space, e.g. 30 bytes:
        let data = vec![0u8; 30];
        let info = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Data::Raw(
                data.try_into().expect("Data up to 128 bytes OK"),
            )])
            .expect("1 field is <= MaxFields"),
        });

        // 2) First call => usage=0 => usage=30 after. OK.
        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            info.clone(),
        ));

        // 3) Second call => tries another 30 bytes in the SAME block => total=60 => exceeds 50 => should fail.
        assert_noop!(
            Pallet::<Test>::set_commitment(RuntimeOrigin::signed(who), netuid, info.clone()),
            Error::<Test>::SpaceLimitExceeded
        );
    });
}

#[test]
fn tempo_based_space_limit_resets_after_tempo() {
    new_test_ext().execute_with(|| {
        let netuid = 2;
        let who = 101;

        MaxSpace::<Test>::set(40);
        System::<Test>::set_block_number(1);

        let commit_small = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Data::Raw(
                vec![0u8; 20].try_into().expect("expected ok"),
            )])
            .expect("expected ok"),
        });

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            commit_small.clone()
        ));

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            commit_small.clone()
        ));

        assert_noop!(
            Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(who),
                netuid,
                commit_small.clone()
            ),
            Error::<Test>::SpaceLimitExceeded
        );

        System::<Test>::set_block_number(200);

        assert_noop!(
            Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(who),
                netuid,
                commit_small.clone()
            ),
            Error::<Test>::SpaceLimitExceeded
        );

        System::<Test>::set_block_number(360);

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            commit_small
        ));
    });
}

#[test]
fn tempo_based_space_limit_does_not_affect_different_netuid() {
    new_test_ext().execute_with(|| {
        let netuid_a = 10;
        let netuid_b = 20;
        let who = 111;
        let space_limit = 50;
        MaxSpace::<Test>::set(space_limit);

        let commit_large = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Data::Raw(
                vec![0u8; 40].try_into().expect("expected ok"),
            )])
            .expect("expected ok"),
        });
        let commit_small = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Data::Raw(
                vec![0u8; 20].try_into().expect("expected ok"),
            )])
            .expect("expected ok"),
        });

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid_a,
            commit_large.clone()
        ));

        assert_noop!(
            Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(who),
                netuid_a,
                commit_small.clone()
            ),
            Error::<Test>::SpaceLimitExceeded
        );

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid_b,
            commit_large
        ));

        assert_noop!(
            Pallet::<Test>::set_commitment(RuntimeOrigin::signed(who), netuid_b, commit_small),
            Error::<Test>::SpaceLimitExceeded
        );
    });
}

#[test]
fn tempo_based_space_limit_does_not_affect_different_user() {
    new_test_ext().execute_with(|| {
        let netuid = 10;
        let user1 = 123;
        let user2 = 456;
        let space_limit = 50;
        MaxSpace::<Test>::set(space_limit);

        let commit_large = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Data::Raw(
                vec![0u8; 40].try_into().expect("expected ok"),
            )])
            .expect("expected ok"),
        });
        let commit_small = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Data::Raw(
                vec![0u8; 20].try_into().expect("expected ok"),
            )])
            .expect("expected ok"),
        });

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(user1),
            netuid,
            commit_large.clone()
        ));

        assert_noop!(
            Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(user1),
                netuid,
                commit_small.clone()
            ),
            Error::<Test>::SpaceLimitExceeded
        );

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(user2),
            netuid,
            commit_large
        ));

        assert_noop!(
            Pallet::<Test>::set_commitment(RuntimeOrigin::signed(user2), netuid, commit_small),
            Error::<Test>::SpaceLimitExceeded
        );
    });
}

#[test]
fn tempo_based_space_limit_sudo_set_max_space() {
    new_test_ext().execute_with(|| {
        let netuid = 3;
        let who = 15;
        MaxSpace::<Test>::set(30);

        System::<Test>::set_block_number(1);
        let commit_25 = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![Data::Raw(
                vec![0u8; 25].try_into().expect("expected ok"),
            )])
            .expect("expected ok"),
        });

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            commit_25.clone()
        ));
        assert_noop!(
            Pallet::<Test>::set_commitment(RuntimeOrigin::signed(who), netuid, commit_25.clone()),
            Error::<Test>::SpaceLimitExceeded
        );

        assert_ok!(Pallet::<Test>::set_max_space(RuntimeOrigin::root(), 100));

        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            commit_25
        ));
    });
}

#[allow(clippy::indexing_slicing)]
#[test]
fn on_initialize_reveals_matured_timelocks() {
    new_test_ext().execute_with(|| {
        let who = 42;
        let netuid = 7;
        let reveal_round = 1000;

        let message_text = b"Timelock test via on_initialize";

        let inner_fields: BoundedVec<Data, <Test as Config>::MaxFields> =
            BoundedVec::try_from(vec![Data::Raw(
                message_text
                    .to_vec()
                    .try_into()
                    .expect("<= 128 bytes is OK for Data::Raw"),
            )])
            .expect("Should not exceed MaxFields");

        let inner_info: CommitmentInfo<<Test as Config>::MaxFields> = CommitmentInfo {
            fields: inner_fields,
        };

        let plaintext = inner_info.encode();
        let encrypted = produce_ciphertext(&plaintext, reveal_round);

        let outer_fields = BoundedVec::try_from(vec![Data::TimelockEncrypted {
            encrypted,
            reveal_round,
        }])
        .expect("One field is well under MaxFields");
        let info_outer = CommitmentInfo {
            fields: outer_fields,
        };

        System::<Test>::set_block_number(1);
        assert_ok!(Pallet::<Test>::set_commitment(
            RuntimeOrigin::signed(who),
            netuid,
            Box::new(info_outer)
        ));

        assert!(CommitmentOf::<Test>::get(netuid, who).is_some());
        assert!(
            TimelockedIndex::<Test>::get().contains(&(netuid, who)),
            "Should appear in TimelockedIndex since it contains a timelock"
        );

        let drand_sig_hex = hex::decode(DRAND_QUICKNET_SIG_HEX)
            .expect("Decoding DRAND_QUICKNET_SIG_HEX must not fail");
        insert_drand_pulse(reveal_round, &drand_sig_hex);

        assert!(RevealedCommitments::<Test>::get(netuid, who).is_none());

        System::<Test>::set_block_number(2);
        <Pallet<Test> as Hooks<u64>>::on_initialize(2);

        let revealed_opt = RevealedCommitments::<Test>::get(netuid, who);
        assert!(
            revealed_opt.is_some(),
            "Expected that the timelock got revealed at block #2"
        );

        let leftover = CommitmentOf::<Test>::get(netuid, who);
        assert!(
            leftover.is_none(),
            "After revealing the only timelock, the entire commitment is removed."
        );

        assert!(
            !TimelockedIndex::<Test>::get().contains(&(netuid, who)),
            "No longer in TimelockedIndex after reveal."
        );

        let revealed_data = revealed_opt.expect("expected to not panic");
        assert_eq!(revealed_data.info.fields.len(), 1);
        if let Data::Raw(bound_bytes) = &revealed_data.info.fields[0] {
            assert_eq!(bound_bytes.as_slice(), message_text);
        } else {
            panic!("Expected a Data::Raw variant in revealed data.");
        }
    });
}

#[test]
fn set_commitment_unreserve_leftover_fails() {
    new_test_ext().execute_with(|| {
        use frame_system::RawOrigin;

        let netuid = 999;
        let who = 99;

        Balances::make_free_balance_be(&who, 10_000);

        let fake_deposit = 100;
        let dummy_info = CommitmentInfo {
            fields: BoundedVec::try_from(vec![]).expect("empty fields is fine"),
        };
        let registration = Registration {
            deposit: fake_deposit,
            info: dummy_info,
            block: 0u64.into(),
        };

        CommitmentOf::<Test>::insert(netuid, &who, registration);

        assert_ok!(Balances::reserve(&who, fake_deposit));
        assert_eq!(Balances::reserved_balance(who), 100);

        Balances::unreserve(&who, 10_000);
        assert_eq!(Balances::reserved_balance(who), 0);

        let commit_small = Box::new(CommitmentInfo {
            fields: BoundedVec::try_from(vec![]).expect("no fields is fine"),
        });

        assert_noop!(
            Pallet::<Test>::set_commitment(RawOrigin::Signed(who).into(), netuid, commit_small),
            Error::<Test>::UnexpectedUnreserveLeftover
        );
    });
}
