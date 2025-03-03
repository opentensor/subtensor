use crate::{CommitmentInfo, Data};
use codec::Encode;
use frame_support::traits::Get;
use sp_std::prelude::*;

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{
        Config, Error, Event, Pallet, RateLimit,
        mock::{RuntimeEvent, RuntimeOrigin, Test, new_test_ext},
    };
    use frame_support::{BoundedVec, assert_noop, assert_ok};
    use frame_system::Pallet as System;

    #[test]
    fn manual_data_type_info() {
        let mut registry = scale_info::Registry::new();
        let type_id = registry.register_type(&scale_info::meta_type::<Data>());
        let registry: scale_info::PortableRegistry = registry.into();
        let type_info = registry.resolve(type_id.id).unwrap();

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
            data.push(Data::Raw(vec![0u8; n as usize].try_into().unwrap()));
        }

        // Add a TimelockEncrypted instance
        data.push(Data::TimelockEncrypted {
            encrypted: vec![0u8; 64].try_into().unwrap(),
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
                fields: BoundedVec::try_from(vec![]).unwrap(),
                ..Default::default()
            });

            assert_ok!(Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(1),
                1,
                info.clone()
            ));

            let commitment = Pallet::<Test>::commitment_of(1, &1).unwrap();
            let initial_deposit: u64 = <Test as Config>::InitialDeposit::get();
            assert_eq!(commitment.deposit, initial_deposit);
            assert_eq!(commitment.block, 1);
            assert_eq!(Pallet::<Test>::last_commitment(1, &1), Some(1));
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
                ..Default::default()
            });

            // We never get here, because the constructor panics above.
            let _ =
                Pallet::<Test>::set_commitment(frame_system::RawOrigin::Signed(1).into(), 1, info);
        });
    }

    #[test]
    fn set_commitment_rate_limit_exceeded() {
        new_test_ext().execute_with(|| {
            let rate_limit = <Test as Config>::DefaultRateLimit::get();
            System::<Test>::set_block_number(1);
            let info = Box::new(CommitmentInfo {
                fields: BoundedVec::try_from(vec![]).unwrap(),
                ..Default::default()
            });

            assert_ok!(Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(1),
                1,
                info.clone()
            ));

            // Set block number to just before rate limit expires
            System::<Test>::set_block_number(rate_limit);
            assert_noop!(
                Pallet::<Test>::set_commitment(RuntimeOrigin::signed(1), 1, info.clone()),
                Error::<Test>::CommitmentSetRateLimitExceeded
            );

            // Set block number to after rate limit
            System::<Test>::set_block_number(rate_limit + 1);
            assert_ok!(Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(1),
                1,
                info
            ));
        });
    }

    #[test]
    fn set_commitment_updates_deposit() {
        new_test_ext().execute_with(|| {
            System::<Test>::set_block_number(1);
            let info1 = Box::new(CommitmentInfo {
                fields: BoundedVec::try_from(vec![Default::default(); 2]).unwrap(),
                ..Default::default()
            });
            let info2 = Box::new(CommitmentInfo {
                fields: BoundedVec::try_from(vec![Default::default(); 3]).unwrap(),
                ..Default::default()
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
                Pallet::<Test>::commitment_of(1, &1).unwrap().deposit,
                expected_deposit1
            );

            assert_ok!(Pallet::<Test>::set_commitment(
                RuntimeOrigin::signed(1),
                1,
                info2
            ));
            let expected_deposit2: u64 = initial_deposit + 3u64 * field_deposit;
            assert_eq!(
                Pallet::<Test>::commitment_of(1, &1).unwrap().deposit,
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
                fields: BoundedVec::try_from(vec![]).unwrap(),
                ..Default::default()
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
}
