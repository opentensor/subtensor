#![cfg(test)]
use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use std::iter::repeat;

#[test]
fn environment_works() {
    TestState::default().build_and_execute(|| {
        assert_eq!(
            AllowedProposers::<Test>::get(),
            vec![U256::from(1), U256::from(2), U256::from(3)]
        );
        assert_eq!(
            Triumvirate::<Test>::get(),
            vec![U256::from(1001), U256::from(1002), U256::from(1003)]
        );
    });
}

#[test]
fn environment_members_are_sorted() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(2), U256::from(3), U256::from(1)])
        .with_triumvirate(vec![U256::from(1002), U256::from(1001), U256::from(1003)])
        .build_and_execute(|| {
            assert_eq!(
                AllowedProposers::<Test>::get(),
                vec![U256::from(1), U256::from(2), U256::from(3)]
            );
            assert_eq!(
                Triumvirate::<Test>::get(),
                vec![U256::from(1001), U256::from(1002), U256::from(1003)]
            );
        });
}

#[test]
#[should_panic(expected = "Allowed proposers cannot contain duplicate accounts.")]
fn environment_with_duplicate_allowed_proposers_panics() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(2)])
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Allowed proposers length cannot exceed MaxAllowedProposers.")]
fn environment_with_too_many_allowed_proposers_panics() {
    let max_allowed_proposers = <Test as pallet::Config>::MaxAllowedProposers::get() as usize;
    let allowed_proposers = (0..=max_allowed_proposers).map(|i| U256::from(i)).collect();
    TestState::default()
        .with_allowed_proposers(allowed_proposers)
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Triumvirate cannot contain duplicate accounts.")]
fn environment_with_duplicate_triumvirate_panics() {
    TestState::default()
        .with_triumvirate(vec![U256::from(1001), U256::from(1002), U256::from(1002)])
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Triumvirate length cannot exceed 3.")]
fn environment_with_too_many_triumvirate_panics() {
    let triumvirate = (1..=4).map(|i| U256::from(i)).collect();
    TestState::default()
        .with_triumvirate(triumvirate)
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Allowed proposers and triumvirate must be disjoint.")]
fn environment_with_overlapping_allowed_proposers_and_triumvirate_panics() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(3)])
        .with_triumvirate(vec![U256::from(1001), U256::from(1002), U256::from(1)])
        .build_and_execute(|| {});
}

#[test]
fn set_allowed_proposers_works() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .build_and_execute(|| {
            let allowed_proposers = BoundedVec::truncate_from(vec![
                U256::from(5),
                U256::from(1),
                U256::from(4),
                U256::from(3),
                U256::from(2),
            ]);
            assert_eq!(AllowedProposers::<Test>::get(), vec![]);

            assert_ok!(Pallet::<Test>::set_allowed_proposers(
                // SetAllowedProposersOrigin is EnsureRoot<Self::AccountId>
                RuntimeOrigin::root(),
                allowed_proposers.clone()
            ));

            assert_eq!(
                AllowedProposers::<Test>::get().to_vec(),
                // Sorted allowed proposers
                vec![
                    U256::from(1),
                    U256::from(2),
                    U256::from(3),
                    U256::from(4),
                    U256::from(5)
                ]
            );
            assert_eq!(
                last_event(),
                RuntimeEvent::Governance(Event::<Test>::AllowedProposersSet)
            );
        });
}

#[test]
fn set_allowed_proposers_with_bad_origin_fails() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .build_and_execute(|| {
            let allowed_proposers =
                BoundedVec::truncate_from((1..=5).map(|i| U256::from(i)).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(
                    RuntimeOrigin::signed(U256::from(42)),
                    allowed_proposers.clone()
                ),
                DispatchError::BadOrigin
            );

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(RuntimeOrigin::none(), allowed_proposers),
                DispatchError::BadOrigin
            );
        });
}

#[test]
fn set_allowed_proposers_with_duplicate_accounts_fails() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .build_and_execute(|| {
            let allowed_proposers =
                BoundedVec::truncate_from(repeat(U256::from(1)).take(2).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(RuntimeOrigin::root(), allowed_proposers),
                Error::<Test>::DuplicateAccounts
            );
        });
}

#[test]
fn set_allowed_proposers_with_triumvirate_intersection_fails() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .with_triumvirate(vec![U256::from(1), U256::from(2), U256::from(3)])
        .build_and_execute(|| {
            let allowed_proposers =
                BoundedVec::truncate_from((3..=8).map(|i| U256::from(i)).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(RuntimeOrigin::root(), allowed_proposers),
                Error::<Test>::AllowedProposersAndTriumvirateMustBeDisjoint
            );
        });
}

#[test]
fn set_triumvirate_works() {
    TestState::default()
        .with_triumvirate(vec![])
        .build_and_execute(|| {
            let triumvirate = BoundedVec::truncate_from(vec![
                U256::from(1003),
                U256::from(1001),
                U256::from(1002),
            ]);
            assert_eq!(Triumvirate::<Test>::get(), vec![]);

            assert_ok!(Pallet::<Test>::set_triumvirate(
                // SetTriumvirateOrigin is EnsureRoot<Self::AccountId>
                RuntimeOrigin::root(),
                triumvirate.clone()
            ));

            assert_eq!(
                Triumvirate::<Test>::get(),
                // Sorted triumvirate
                vec![U256::from(1001), U256::from(1002), U256::from(1003)]
            );
            assert_eq!(
                last_event(),
                RuntimeEvent::Governance(Event::<Test>::TriumvirateSet)
            );
        });
}

#[test]
fn set_triumvirate_with_bad_origin_fails() {
    TestState::default()
        .with_triumvirate(vec![])
        .build_and_execute(|| {
            let triumvirate = BoundedVec::truncate_from(
                (1..=3).map(|i| U256::from(1000 + i)).collect::<Vec<_>>(),
            );

            assert_noop!(
                Pallet::<Test>::set_triumvirate(
                    RuntimeOrigin::signed(U256::from(42)),
                    triumvirate.clone()
                ),
                DispatchError::BadOrigin
            );

            assert_noop!(
                Pallet::<Test>::set_triumvirate(RuntimeOrigin::none(), triumvirate),
                DispatchError::BadOrigin
            );
        });
}

#[test]
fn set_triumvirate_with_duplicate_accounts_fails() {
    TestState::default()
        .with_triumvirate(vec![])
        .build_and_execute(|| {
            let triumvirate =
                BoundedVec::truncate_from(repeat(U256::from(1001)).take(2).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_triumvirate(RuntimeOrigin::root(), triumvirate),
                Error::<Test>::DuplicateAccounts
            );
        });
}

#[test]
fn set_triumvirate_with_allowed_proposers_intersection_fails() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(3)])
        .build_and_execute(|| {
            let triumvirate =
                BoundedVec::truncate_from((3..=8).map(|i| U256::from(i)).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_triumvirate(RuntimeOrigin::root(), triumvirate),
                Error::<Test>::AllowedProposersAndTriumvirateMustBeDisjoint
            );
        });
}
