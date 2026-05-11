#![allow(clippy::unwrap_used, clippy::expect_used)]

use frame_support::{
    BoundedVec, assert_err_ignore_postinfo, assert_noop, assert_ok, traits::Hooks, weights::Weight,
};
use sp_core::U256;
use sp_runtime::DispatchError;

use crate::{
    Collective, CollectiveInfo, CollectiveInspect, Error, Event as CollectiveEvent, OnNewTerm,
    Pallet as MultiCollective, mock::*,
};

#[test]
fn add_member_happy_path() {
    TestState::build_and_execute(|| {
        let mid = U256::from(5);
        let head = U256::from(2);
        let tail = U256::from(8);
        let between = U256::from(4);

        // Exercises the four insertion positions that `binary_search` can
        // return: empty list, before the first element, after the last,
        // and into the middle. A regression replacing the sorted insert
        // with `push` would only be caught by the head and middle cases.
        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            mid,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![mid]
        );
        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &mid
        ));

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            head,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![head, mid]
        );

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            tail,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![head, mid, tail]
        );

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            between,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![head, between, mid, tail]
        );

        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            4
        );

        assert_eq!(
            multi_collective_events(),
            vec![
                CollectiveEvent::MemberAdded {
                    collective_id: CollectiveId::Alpha,
                    who: mid,
                },
                CollectiveEvent::MemberAdded {
                    collective_id: CollectiveId::Alpha,
                    who: head,
                },
                CollectiveEvent::MemberAdded {
                    collective_id: CollectiveId::Alpha,
                    who: tail,
                },
                CollectiveEvent::MemberAdded {
                    collective_id: CollectiveId::Alpha,
                    who: between,
                },
            ]
        );
    });
}

#[test]
fn add_member_requires_origin() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let caller = U256::from(999);

        assert_noop!(
            MultiCollective::<Test>::add_member(
                RuntimeOrigin::signed(caller),
                CollectiveId::Alpha,
                alice,
            ),
            DispatchError::BadOrigin
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Alpha).is_empty());
        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn add_member_fails_for_unknown_collective() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);

        assert_noop!(
            MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Unknown,
                alice,
            ),
            Error::<Test>::CollectiveNotFound
        );

        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn add_member_rejects_duplicate() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));

        assert_noop!(
            MultiCollective::<Test>::add_member(RuntimeOrigin::root(), CollectiveId::Alpha, alice,),
            Error::<Test>::AlreadyMember
        );

        // Only one MemberAdded event; the failing call produced nothing.
        assert_eq!(
            multi_collective_events(),
            vec![CollectiveEvent::MemberAdded {
                collective_id: CollectiveId::Alpha,
                who: alice,
            }]
        );
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            1
        );
    });
}

#[test]
fn add_member_respects_info_max() {
    TestState::build_and_execute(|| {
        // Alpha declares max_members = Some(5). Fill it exactly to capacity.
        for i in 1..=5u32 {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                U256::from(i),
            ));
        }
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            5
        );

        assert_noop!(
            MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                U256::from(6),
            ),
            Error::<Test>::TooManyMembers
        );

        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            5
        );
        // Exactly five events; nothing from the failing 6th.
        assert_eq!(multi_collective_events().len(), 5);
    });
}

#[test]
fn add_member_respects_storage_max_when_info_max_none() {
    TestState::build_and_execute(|| {
        // Gamma's `info.max_members` is None; only `T::MaxMembers = 32` applies.
        for i in 1..=32u32 {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Gamma,
                U256::from(i),
            ));
        }
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Gamma),
            32
        );

        // 33rd add fails via `try_insert` (BoundedVec bound) rather than the info cap.
        assert_noop!(
            MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Gamma,
                U256::from(33),
            ),
            Error::<Test>::TooManyMembers
        );

        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Gamma),
            32
        );
        assert_eq!(multi_collective_events().len(), 32);
    });
}

#[test]
fn remove_member_happy_path() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        let charlie = U256::from(3);

        for who in [alice, bob, charlie] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        // Remove from the middle.
        assert_ok!(MultiCollective::<Test>::remove_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            bob,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![alice, charlie]
        );
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &bob
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            2
        );

        // Remove from the head. A swap-remove would leave the list
        // unsorted (`[charlie, ...]` shifting via swap), so asserting
        // that the remaining tail stays in order discriminates against
        // that regression.
        assert_ok!(MultiCollective::<Test>::remove_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![charlie]
        );
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &alice
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            1
        );

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberRemoved {
                collective_id: CollectiveId::Alpha,
                who: alice,
            })
        );
    });
}

#[test]
fn remove_member_requires_origin() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));

        assert_noop!(
            MultiCollective::<Test>::remove_member(
                RuntimeOrigin::signed(U256::from(999)),
                CollectiveId::Alpha,
                alice,
            ),
            DispatchError::BadOrigin
        );

        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &alice
        ));
    });
}

#[test]
fn remove_member_fails_for_unknown_collective() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::remove_member(
                RuntimeOrigin::root(),
                CollectiveId::Unknown,
                U256::from(1),
            ),
            Error::<Test>::CollectiveNotFound
        );

        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn remove_member_rejects_non_member() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::remove_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                U256::from(1),
            ),
            Error::<Test>::NotMember
        );

        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn remove_member_respects_min() {
    TestState::build_and_execute(|| {
        // Beta declares min_members = 2. Seed exactly to the floor.
        let alice = U256::from(1);
        let bob = U256::from(2);
        for who in [alice, bob] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Beta,
                who,
            ));
        }

        assert_noop!(
            MultiCollective::<Test>::remove_member(
                RuntimeOrigin::root(),
                CollectiveId::Beta,
                alice,
            ),
            Error::<Test>::TooFewMembers
        );

        assert_eq!(MultiCollective::<Test>::member_count(CollectiveId::Beta), 2);
    });
}

#[test]
fn remove_member_allows_down_to_min() {
    TestState::build_and_execute(|| {
        // Beta has min_members = 2; seed with one above.
        let alice = U256::from(1);
        let bob = U256::from(2);
        let charlie = U256::from(3);
        for who in [alice, bob, charlie] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Beta,
                who,
            ));
        }

        // Removing once leaves the collective at min_members; the check is
        // `len() > min_members` so post-removal len == min_members is allowed.
        assert_ok!(MultiCollective::<Test>::remove_member(
            RuntimeOrigin::root(),
            CollectiveId::Beta,
            charlie,
        ));

        assert_eq!(MultiCollective::<Test>::member_count(CollectiveId::Beta), 2);
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &charlie
        ));

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberRemoved {
                collective_id: CollectiveId::Beta,
                who: charlie,
            })
        );
    });
}

#[test]
fn swap_member_happy_path() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        let charlie = U256::from(3);
        let dave = U256::from(4);
        let zara = U256::from(10);

        for who in [alice, bob, charlie] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        // Swap the middle member for an account that sorts to the tail.
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            bob,
            dave,
        ));

        // Members are kept sorted: dave (4) goes after charlie (3).
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![alice, charlie, dave]
        );
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &bob
        ));
        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &dave
        ));

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberSwapped {
                collective_id: CollectiveId::Alpha,
                removed: bob,
                added: dave,
            })
        );

        // Swap the head member for an account that sorts to the tail.
        // A swap-remove regression on the remove side would leave the
        // resulting list unsorted, so this exercises both sides of the
        // sorted invariant.
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
            zara,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![charlie, dave, zara]
        );
    });
}

#[test]
fn swap_member_requires_origin() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));

        assert_noop!(
            MultiCollective::<Test>::swap_member(
                RuntimeOrigin::signed(U256::from(999)),
                CollectiveId::Alpha,
                alice,
                U256::from(2),
            ),
            DispatchError::BadOrigin
        );

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![alice]
        );
    });
}

#[test]
fn swap_member_fails_for_unknown_collective() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::swap_member(
                RuntimeOrigin::root(),
                CollectiveId::Unknown,
                U256::from(1),
                U256::from(2),
            ),
            Error::<Test>::CollectiveNotFound
        );

        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn swap_member_rejects_missing_remove() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::swap_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                U256::from(1),
                U256::from(2),
            ),
            Error::<Test>::NotMember
        );

        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn swap_member_rejects_existing_add() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);

        for who in [alice, bob] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        assert_noop!(
            MultiCollective::<Test>::swap_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                alice,
                bob,
            ),
            Error::<Test>::AlreadyMember
        );

        // Both still present, in their original order.
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![alice, bob]
        );
    });
}

#[test]
fn swap_member_rejects_self_swap() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));

        // `remove` matches a member, so `NotMember` doesn't fire; the next
        // check (`!contains(add)`) rejects because add is already present
        // (it is `remove` itself). "Swap with self" is a no-op the pallet
        // refuses.
        assert_noop!(
            MultiCollective::<Test>::swap_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                alice,
                alice,
            ),
            Error::<Test>::AlreadyMember
        );

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![alice]
        );
    });
}

/// Beta has `min_members = 2, max_members = 3`. Swap is count-invariant
/// and skips both bounds checks, so it must succeed at either end.
/// Setup walks the collective from min to max via `add_member`, then
/// swaps once at each bound.
#[test]
fn swap_member_works_at_bounds() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        let carol = U256::from(3);
        let dave = U256::from(4);
        let erin = U256::from(5);

        for who in [alice, bob] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Beta,
                who,
            ));
        }

        // At min: swap alice for carol.
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Beta,
            alice,
            carol,
        ));
        assert_eq!(MultiCollective::<Test>::member_count(CollectiveId::Beta), 2);
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &alice
        ));
        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &carol
        ));

        // Grow to max, then at max: swap carol for dave.
        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Beta,
            dave,
        ));
        assert_eq!(MultiCollective::<Test>::member_count(CollectiveId::Beta), 3);

        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Beta,
            carol,
            erin,
        ));
        assert_eq!(MultiCollective::<Test>::member_count(CollectiveId::Beta), 3);
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &carol
        ));
        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &erin
        ));
    });
}

#[test]
fn set_members_replaces_list() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);
        let c = U256::from(3);
        let d = U256::from(4);
        let e = U256::from(5);

        for who in [a, b] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        assert_ok!(MultiCollective::<Test>::set_members(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            vec![c, d, e],
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![c, d, e]
        );
        assert!(!MultiCollective::<Test>::is_member(CollectiveId::Alpha, &a));
        assert!(!MultiCollective::<Test>::is_member(CollectiveId::Alpha, &b));

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MembersSet {
                collective_id: CollectiveId::Alpha,
                outgoing: vec![a, b],
                incoming: vec![c, d, e],
            })
        );
    });
}

#[test]
fn set_members_handles_overlap() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);
        let c = U256::from(3);
        let d = U256::from(4);

        for who in [a, b, c] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        // [b, c, d] overlaps with the old [a, b, c]: b and c stay, a goes out,
        // d comes in. Final storage reflects the new list verbatim.
        assert_ok!(MultiCollective::<Test>::set_members(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            vec![b, c, d],
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![b, c, d]
        );

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MembersSet {
                collective_id: CollectiveId::Alpha,
                outgoing: vec![a],
                incoming: vec![d],
            })
        );
    });
}

#[test]
fn set_members_requires_origin() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::set_members(
                RuntimeOrigin::signed(U256::from(999)),
                CollectiveId::Alpha,
                vec![U256::from(1)],
            ),
            DispatchError::BadOrigin
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Alpha).is_empty());
        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn set_members_fails_for_unknown_collective() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::set_members(
                RuntimeOrigin::root(),
                CollectiveId::Unknown,
                vec![U256::from(1)],
            ),
            Error::<Test>::CollectiveNotFound
        );

        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn set_members_rejects_too_few() {
    TestState::build_and_execute(|| {
        // Beta declares min_members = 2.
        assert_noop!(
            MultiCollective::<Test>::set_members(
                RuntimeOrigin::root(),
                CollectiveId::Beta,
                vec![U256::from(1)],
            ),
            Error::<Test>::TooFewMembers
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Beta).is_empty());
        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn set_members_rejects_too_many_via_info() {
    TestState::build_and_execute(|| {
        // Beta declares max_members = Some(3); four accounts is one over.
        let list: Vec<U256> = (1..=4u32).map(U256::from).collect();
        assert_noop!(
            MultiCollective::<Test>::set_members(RuntimeOrigin::root(), CollectiveId::Beta, list,),
            Error::<Test>::TooManyMembers
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Beta).is_empty());
        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn set_members_rejects_too_many_via_storage() {
    TestState::build_and_execute(|| {
        // Gamma's info.max_members is None; only T::MaxMembers = 32 applies.
        // 33 accounts exceed the BoundedVec bound, caught by try_from.
        let list: Vec<U256> = (1..=33u32).map(U256::from).collect();
        assert_noop!(
            MultiCollective::<Test>::set_members(RuntimeOrigin::root(), CollectiveId::Gamma, list,),
            Error::<Test>::TooManyMembers
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Gamma).is_empty());
    });
}

#[test]
fn set_members_rejects_duplicates() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);

        assert_noop!(
            MultiCollective::<Test>::set_members(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                vec![a, b, a],
            ),
            Error::<Test>::DuplicateAccounts
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Alpha).is_empty());
    });
}

/// Setting a list identical to the current membership still emits a
/// `MembersSet` event; the pallet doesn't short-circuit no-op sets.
/// Pinned so downstream consumers know they must tolerate empty-diff calls.
#[test]
fn set_members_noop_still_fires_event() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);

        for who in [a, b] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        assert_ok!(MultiCollective::<Test>::set_members(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            vec![a, b],
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![a, b]
        );

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MembersSet {
                collective_id: CollectiveId::Alpha,
                incoming: vec![],
                outgoing: vec![],
            })
        );
    });
}

#[test]
fn on_initialize_no_rotation_when_term_duration_none() {
    TestState::build_and_execute(|| {
        // Alpha (td=None) and Gamma (td=None) must never appear in the log
        // regardless of how many blocks pass.
        run_to_block(300);

        let log = take_new_term_log();
        assert!(
            !log.contains(&CollectiveId::Alpha),
            "Alpha has term_duration = None; should never rotate"
        );
        assert!(
            !log.contains(&CollectiveId::Gamma),
            "Gamma has term_duration = None; should never rotate"
        );
    });
}

#[test]
fn on_initialize_no_rotation_between_boundaries() {
    TestState::build_and_execute(|| {
        // Earliest boundary is Delta's at block 50. Before that, nothing fires.
        run_to_block(49);
        assert!(take_new_term_log().is_empty());
    });
}

#[test]
fn on_initialize_fires_rotation_at_modulo_boundary() {
    TestState::build_and_execute(|| {
        // Delta (td=50) first fires at block 50. The "no rotation between
        // boundaries" property is covered by
        // `on_initialize_no_rotation_between_boundaries`.
        run_to_block(50);
        assert_eq!(take_new_term_log(), vec![CollectiveId::Delta]);
    });
}

#[test]
fn on_initialize_fires_all_matching_collectives() {
    TestState::build_and_execute(|| {
        // Advance through the first shared boundary at block 100. Delta fires
        // at 50, then both Beta and Delta fire at 100. Iteration order in
        // `TestCollectives` is [Alpha, Beta, Gamma, Delta], so within block
        // 100 the log gets Beta before Delta.
        run_to_block(100);

        assert_eq!(
            take_new_term_log(),
            vec![
                CollectiveId::Delta, // block 50
                CollectiveId::Beta,  // block 100
                CollectiveId::Delta, // block 100
            ]
        );

        // Next cadence: only Delta at 150, both again at 200.
        run_to_block(150);
        assert_eq!(take_new_term_log(), vec![CollectiveId::Delta]);

        run_to_block(200);
        assert_eq!(
            take_new_term_log(),
            vec![CollectiveId::Beta, CollectiveId::Delta]
        );
    });
}

#[test]
fn force_rotate_routes_through_on_new_term() {
    TestState::build_and_execute(|| {
        // Beta has term_duration = Some(100), so it's eligible.
        assert_ok!(MultiCollective::<Test>::force_rotate(
            RuntimeOrigin::root(),
            CollectiveId::Beta,
        ));
        assert_eq!(take_new_term_log(), vec![CollectiveId::Beta]);
    });
}

#[test]
fn force_rotate_requires_origin() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::force_rotate(
                RuntimeOrigin::signed(U256::from(1)),
                CollectiveId::Beta,
            ),
            DispatchError::BadOrigin,
        );
        assert!(take_new_term_log().is_empty());
    });
}

#[test]
fn force_rotate_rejects_non_rotating_collective() {
    TestState::build_and_execute(|| {
        // Alpha has `term_duration: None`.
        assert_noop!(
            MultiCollective::<Test>::force_rotate(RuntimeOrigin::root(), CollectiveId::Alpha,),
            Error::<Test>::CollectiveDoesNotRotate,
        );
        assert!(take_new_term_log().is_empty());
    });
}

#[test]
fn force_rotate_rejects_unknown_collective() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::force_rotate(RuntimeOrigin::root(), CollectiveId::Unknown,),
            Error::<Test>::CollectiveNotFound,
        );
        assert!(take_new_term_log().is_empty());
    });
}

#[test]
fn inspect_is_member_basic() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let mallory = U256::from(999);

        // Empty collective: no membership.
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &alice
        ));

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));

        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &alice
        ));
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &mallory
        ));
        // Membership is per-collective; alice isn't in Beta.
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &alice
        ));
    });
}

#[test]
fn inspect_member_count_matches_mutations() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);
        let c = U256::from(3);
        let d = U256::from(4);

        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            0
        );

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            a,
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            1
        );

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            b,
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            2
        );

        // Swap is count-invariant.
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            a,
            c,
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            2
        );

        // Remove decrements by one.
        assert_ok!(MultiCollective::<Test>::remove_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            b,
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            1
        );

        // `set_members` replaces wholesale; count reflects the new list length.
        assert_ok!(MultiCollective::<Test>::set_members(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            vec![a, b, c, d],
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            4
        );
    });
}

#[test]
fn inspect_of_unknown_collective_returns_empty() {
    TestState::build_and_execute(|| {
        // `Unknown` is not registered in TestCollectives::collectives().
        // `Members` storage uses ValueQuery and returns an empty BoundedVec by
        // default, so all three reads succeed without error or panic.
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Unknown),
            Vec::<U256>::new()
        );
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Unknown,
            &U256::from(1)
        ));
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Unknown),
            0
        );
    });
}

// `integrity_test_passes_on_valid_config` is implicit: the mock's
// auto-generated `__construct_runtime_integrity_test::runtime_integrity_tests`
// runs `integrity_test()` against the default `TestCollectives` on every
// `cargo test`. Listed in test output as `mock::...runtime_integrity_tests`.

fn bad_min_exceeds_storage() -> Vec<Collective<CollectiveId, u64, [u8; 32]>> {
    vec![Collective {
        id: CollectiveId::Alpha,
        info: CollectiveInfo {
            name: name_bytes(b"bad"),
            // T::MaxMembers = 32 in the mock; 100 exceeds storage capacity.
            min_members: 100,
            max_members: Some(200),
            term_duration: None,
        },
    }]
}

fn bad_max_exceeds_storage() -> Vec<Collective<CollectiveId, u64, [u8; 32]>> {
    vec![Collective {
        id: CollectiveId::Alpha,
        info: CollectiveInfo {
            name: name_bytes(b"bad"),
            min_members: 0,
            // T::MaxMembers = 32; max_members = 100 is declaratively larger.
            max_members: Some(100),
            term_duration: None,
        },
    }]
}

fn bad_min_exceeds_info_max() -> Vec<Collective<CollectiveId, u64, [u8; 32]>> {
    vec![Collective {
        id: CollectiveId::Alpha,
        info: CollectiveInfo {
            name: name_bytes(b"bad"),
            // min > max: the collective can never satisfy both.
            min_members: 5,
            max_members: Some(3),
            term_duration: None,
        },
    }]
}

fn bad_term_duration_zero() -> Vec<Collective<CollectiveId, u64, [u8; 32]>> {
    vec![Collective {
        id: CollectiveId::Alpha,
        info: CollectiveInfo {
            name: name_bytes(b"bad"),
            min_members: 0,
            max_members: Some(5),
            // Some(0) silently disables rotations; integrity_test rejects it.
            term_duration: Some(0),
        },
    }]
}

#[test]
#[should_panic(expected = "min_members (100) exceeds T::MaxMembers (32)")]
fn integrity_test_panics_on_min_exceeds_storage_max() {
    with_collectives_override(bad_min_exceeds_storage, || {
        <MultiCollective<Test> as Hooks<u64>>::integrity_test();
    });
}

#[test]
#[should_panic(expected = "max_members (100) exceeds T::MaxMembers (32)")]
fn integrity_test_panics_on_max_exceeds_storage_max() {
    with_collectives_override(bad_max_exceeds_storage, || {
        <MultiCollective<Test> as Hooks<u64>>::integrity_test();
    });
}

#[test]
#[should_panic(expected = "min_members (5) exceeds max_members (3)")]
fn integrity_test_panics_on_min_exceeds_info_max() {
    with_collectives_override(bad_min_exceeds_info_max, || {
        <MultiCollective<Test> as Hooks<u64>>::integrity_test();
    });
}

#[test]
#[should_panic(expected = "silently disables rotations")]
fn integrity_test_panics_on_term_duration_zero() {
    with_collectives_override(bad_term_duration_zero, || {
        <MultiCollective<Test> as Hooks<u64>>::integrity_test();
    });
}

// `OnMembersChanged` payload tests. The pallet's events show what changed
// in storage but not what was passed to the hook, so an argument-order
// regression (e.g. swapping `incoming` and `outgoing`) would not be
// caught by the event assertions alone.

#[test]
fn on_members_changed_payload_for_add_member() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));
        assert_eq!(
            take_members_changed_log(),
            vec![MembersChangedCall {
                collective_id: CollectiveId::Alpha,
                incoming: vec![alice],
                outgoing: vec![],
            }]
        );
    });
}

#[test]
fn on_members_changed_payload_for_remove_member() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        for who in [alice, bob] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }
        let _ = take_members_changed_log();

        assert_ok!(MultiCollective::<Test>::remove_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            bob,
        ));
        assert_eq!(
            take_members_changed_log(),
            vec![MembersChangedCall {
                collective_id: CollectiveId::Alpha,
                incoming: vec![],
                outgoing: vec![bob],
            }]
        );
    });
}

#[test]
fn on_members_changed_payload_for_swap_member() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        for who in [alice, bob] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }
        let _ = take_members_changed_log();

        let carol = U256::from(3);
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
            carol,
        ));
        assert_eq!(
            take_members_changed_log(),
            vec![MembersChangedCall {
                collective_id: CollectiveId::Alpha,
                incoming: vec![carol],
                outgoing: vec![alice],
            }]
        );
    });
}

#[test]
fn on_members_changed_payload_for_set_members() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);
        let c = U256::from(3);
        let d = U256::from(4);
        for who in [a, b, c] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }
        let _ = take_members_changed_log();

        assert_ok!(MultiCollective::<Test>::set_members(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            vec![b, c, d],
        ));
        assert_eq!(
            take_members_changed_log(),
            vec![MembersChangedCall {
                collective_id: CollectiveId::Alpha,
                incoming: vec![d],
                outgoing: vec![a],
            }]
        );
    });
}

// `do_try_state` direct tests. The extrinsics maintain the invariants by
// construction, so corrupting `Members` storage manually is the only way
// to exercise each failure branch.

fn write_raw_members(id: CollectiveId, members: Vec<U256>) {
    let bounded = BoundedVec::try_from(members).expect("test fixture must fit MaxMembers");
    crate::pallet::Members::<Test>::insert(id, bounded);
}

#[test]
fn try_state_passes_on_valid_storage() {
    TestState::build_and_execute(|| {
        for who in [U256::from(1), U256::from(2)] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }
        assert!(MultiCollective::<Test>::do_try_state().is_ok());
    });
}

#[test]
fn try_state_rejects_unsorted_storage() {
    TestState::build_and_execute(|| {
        write_raw_members(CollectiveId::Alpha, vec![U256::from(2), U256::from(1)]);
        assert!(MultiCollective::<Test>::do_try_state().is_err());
    });
}

#[test]
fn try_state_rejects_orphan_collective_row() {
    TestState::build_and_execute(|| {
        // `Unknown` is reachable via the storage map's `Blake2_128Concat`
        // hash but is not registered in `TestCollectives::collectives()`.
        write_raw_members(CollectiveId::Unknown, vec![U256::from(1)]);
        assert!(MultiCollective::<Test>::do_try_state().is_err());
    });
}

#[test]
fn try_state_rejects_count_exceeding_info_max() {
    TestState::build_and_execute(|| {
        // Beta declares max_members = 3; four entries fit the BoundedVec
        // bound (T::MaxMembers = 32) but violate the per-collective cap.
        let four: Vec<U256> = (1..=4u32).map(U256::from).collect();
        write_raw_members(CollectiveId::Beta, four);
        assert!(MultiCollective::<Test>::do_try_state().is_err());
    });
}

/// `set_members` sorts its input before writing. Without this step,
/// downstream `binary_search` and `compute_members_diff_sorted` calls
/// would silently observe an unsorted storage entry; pinning the sort
/// here guards against a regression that drops the `sorted.sort()` call.
#[test]
fn set_members_sorts_input() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);
        let c = U256::from(3);

        assert_ok!(MultiCollective::<Test>::set_members(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            vec![c, a, b],
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![a, b, c]
        );
    });
}

/// `force_rotate` returns `Some(actual_weight)` equal to
/// `WeightInfo::force_rotate() + OnNewTerm::on_new_term(...)`. The mock's
/// `WeightInfo` is `()` (zero), so the post-info weight should equal the
/// hook's reported cost, which we set explicitly here.
#[test]
fn force_rotate_returns_post_info_weight() {
    TestState::build_and_execute(|| {
        let hook_weight = Weight::from_parts(123_456, 0);
        set_new_term_weight(hook_weight);

        let post = MultiCollective::<Test>::force_rotate(RuntimeOrigin::root(), CollectiveId::Beta)
            .expect("force_rotate succeeds for Beta");

        assert_eq!(post.actual_weight, Some(hook_weight));
    });
}

/// The pallet ships a tuple impl of `OnNewTerm` so a runtime can fan a
/// rotation out to multiple handlers. The mock wires a single impl, so
/// without this test the tuple expansion is not exercised by `cargo test`.
#[test]
fn on_new_term_tuple_impl_dispatches_to_each_member() {
    TestState::build_and_execute(|| {
        set_new_term_weight(Weight::from_parts(7, 0));

        let combined = <(TestOnNewTerm, TestOnNewTerm) as OnNewTerm<CollectiveId>>::on_new_term(
            CollectiveId::Beta,
        );

        assert_eq!(combined, Weight::from_parts(14, 0));
        assert_eq!(
            take_new_term_log(),
            vec![CollectiveId::Beta, CollectiveId::Beta]
        );

        let weight = <(TestOnNewTerm, TestOnNewTerm) as OnNewTerm<CollectiveId>>::weight();
        assert_eq!(weight, Weight::from_parts(14, 0));
    });
}

#[test]
fn try_join_admits_into_empty_collective() {
    TestState::build_and_execute(|| {
        let candidate = U256::from(7);
        set_eligible(CollectiveId::Alpha, candidate, true);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Alpha,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![candidate]
        );
        assert_eq!(
            take_members_changed_log(),
            vec![MembersChangedCall {
                collective_id: CollectiveId::Alpha,
                incoming: vec![candidate],
                outgoing: vec![],
            }]
        );
        assert_eq!(
            multi_collective_events(),
            vec![CollectiveEvent::MemberJoined {
                collective_id: CollectiveId::Alpha,
                who: candidate,
                evicted: vec![],
            }]
        );
    });
}

#[test]
fn try_join_preserves_sort_invariant_for_all_insert_positions() {
    TestState::build_and_execute(|| {
        let head = U256::from(1);
        let mid = U256::from(5);
        let tail = U256::from(9);
        let between_low = U256::from(3);
        let between_high = U256::from(7);

        // Seed the middle; subsequent inserts must land at head, after the
        // middle, before the tail, and at the very end. Mark the seed as
        // eligible so the sweep doesn't evict it.
        seed_members(CollectiveId::Alpha, &[mid]);
        set_eligible(CollectiveId::Alpha, mid, true);

        for c in [head, tail, between_low, between_high] {
            set_eligible(CollectiveId::Alpha, c, true);
            assert_ok!(MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(c),
                CollectiveId::Alpha,
            ));
        }

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![head, between_low, mid, between_high, tail]
        );
    });
}

#[test]
fn try_join_requires_signed_origin() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::try_join(RuntimeOrigin::root(), CollectiveId::Alpha),
            DispatchError::BadOrigin,
        );
        assert_noop!(
            MultiCollective::<Test>::try_join(RuntimeOrigin::none(), CollectiveId::Alpha),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn try_join_fails_for_unknown_collective() {
    TestState::build_and_execute(|| {
        let candidate = U256::from(1);
        set_eligible(CollectiveId::Unknown, candidate, true);

        assert_noop!(
            MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(candidate),
                CollectiveId::Unknown,
            ),
            Error::<Test>::CollectiveNotFound
        );
    });
}

#[test]
fn try_join_rejects_already_member() {
    TestState::build_and_execute(|| {
        let candidate = U256::from(4);
        seed_members(CollectiveId::Alpha, &[candidate]);
        set_eligible(CollectiveId::Alpha, candidate, true);

        assert_noop!(
            MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(candidate),
                CollectiveId::Alpha,
            ),
            Error::<Test>::AlreadyMember
        );
    });
}

#[test]
fn try_join_rejects_ineligible_candidate() {
    TestState::build_and_execute(|| {
        let candidate = U256::from(4);
        assert_noop!(
            MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(candidate),
                CollectiveId::Alpha,
            ),
            Error::<Test>::NotEligible
        );

        // Marking the candidate eligible for a *different* collective does
        // not unlock admission into Alpha.
        set_eligible(CollectiveId::Beta, candidate, true);
        assert_noop!(
            MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(candidate),
                CollectiveId::Alpha,
            ),
            Error::<Test>::NotEligible
        );
    });
}

#[test]
fn try_join_evicts_lowest_ranked_when_full_and_candidate_outranks() {
    TestState::build_and_execute(|| {
        // Alpha caps at 5. Fill with five members of strictly ascending ranks.
        let m1 = U256::from(10);
        let m2 = U256::from(20);
        let m3 = U256::from(30);
        let m4 = U256::from(40);
        let m5 = U256::from(50);
        seed_members(CollectiveId::Alpha, &[m1, m2, m3, m4, m5]);
        for (m, r) in [(m1, 1u128), (m2, 2), (m3, 3), (m4, 4), (m5, 5)] {
            set_eligible(CollectiveId::Alpha, m, true);
            set_rank(CollectiveId::Alpha, m, r);
        }

        let candidate = U256::from(25);
        set_eligible(CollectiveId::Alpha, candidate, true);
        set_rank(CollectiveId::Alpha, candidate, 99);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Alpha,
        ));

        // m1 had the lowest rank; it gets evicted. Sorted insert places the
        // candidate between m2 (id=20) and m3 (id=30).
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![m2, candidate, m3, m4, m5]
        );
        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberJoined {
                collective_id: CollectiveId::Alpha,
                who: candidate,
                evicted: vec![m1],
            })
        );
        assert_eq!(
            take_members_changed_log().last(),
            Some(&MembersChangedCall {
                collective_id: CollectiveId::Alpha,
                incoming: vec![candidate],
                outgoing: vec![m1],
            })
        );
    });
}

#[test]
fn try_join_full_collective_evicts_correctly_when_lowest_id_is_above_candidate() {
    TestState::build_and_execute(|| {
        // Setup forces the lowest-rank member to live at an index greater
        // than the candidate's insertion position. The replacement-index
        // adjustment in `try_join` must not double-decrement.
        let m1 = U256::from(10);
        let m2 = U256::from(20);
        let m3 = U256::from(30);
        let m4 = U256::from(40);
        let m5 = U256::from(50);
        seed_members(CollectiveId::Alpha, &[m1, m2, m3, m4, m5]);
        // m5 (account id = 50) is the lowest-ranked member.
        for (m, r) in [(m1, 9u128), (m2, 8), (m3, 7), (m4, 6), (m5, 1)] {
            set_eligible(CollectiveId::Alpha, m, true);
            set_rank(CollectiveId::Alpha, m, r);
        }

        let candidate = U256::from(15);
        set_eligible(CollectiveId::Alpha, candidate, true);
        set_rank(CollectiveId::Alpha, candidate, 5);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Alpha,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![m1, candidate, m2, m3, m4]
        );
    });
}

#[test]
fn try_join_rejects_when_candidate_rank_equals_lowest() {
    TestState::build_and_execute(|| {
        // Tie at the bottom: `try_join`'s eviction rule is strict `>`, so
        // an equal-rank candidate must not displace the incumbent.
        let m1 = U256::from(10);
        let m2 = U256::from(20);
        let m3 = U256::from(30);
        let m4 = U256::from(40);
        let m5 = U256::from(50);
        seed_members(CollectiveId::Alpha, &[m1, m2, m3, m4, m5]);
        for m in [m1, m2, m3, m4, m5] {
            set_eligible(CollectiveId::Alpha, m, true);
            set_rank(CollectiveId::Alpha, m, 5);
        }

        let candidate = U256::from(7);
        set_eligible(CollectiveId::Alpha, candidate, true);
        set_rank(CollectiveId::Alpha, candidate, 5);

        let before = MultiCollective::<Test>::members_of(CollectiveId::Alpha);
        assert_err_ignore_postinfo!(
            MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(candidate),
                CollectiveId::Alpha,
            ),
            Error::<Test>::RankTooLow
        );
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            before
        );
    });
}

#[test]
fn try_join_rejects_when_candidate_rank_below_lowest() {
    TestState::build_and_execute(|| {
        let m1 = U256::from(10);
        let m2 = U256::from(20);
        let m3 = U256::from(30);
        let m4 = U256::from(40);
        let m5 = U256::from(50);
        seed_members(CollectiveId::Alpha, &[m1, m2, m3, m4, m5]);
        for (m, r) in [(m1, 5u128), (m2, 6), (m3, 7), (m4, 8), (m5, 9)] {
            set_eligible(CollectiveId::Alpha, m, true);
            set_rank(CollectiveId::Alpha, m, r);
        }

        let candidate = U256::from(99);
        set_eligible(CollectiveId::Alpha, candidate, true);
        set_rank(CollectiveId::Alpha, candidate, 1);

        let before = MultiCollective::<Test>::members_of(CollectiveId::Alpha);
        assert_err_ignore_postinfo!(
            MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(candidate),
                CollectiveId::Alpha,
            ),
            Error::<Test>::RankTooLow
        );
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            before
        );
    });
}

#[test]
fn try_join_does_not_consult_rank_when_max_members_is_unbounded() {
    TestState::build_and_execute(|| {
        // Gamma has `max_members = None`. Even with a very low rank for the
        // candidate, admission must succeed once eligibility is set.
        for who in [U256::from(1), U256::from(2), U256::from(3)] {
            set_eligible(CollectiveId::Gamma, who, true);
            assert_ok!(MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(who),
                CollectiveId::Gamma,
            ));
        }

        let candidate = U256::from(4);
        set_eligible(CollectiveId::Gamma, candidate, true);
        set_rank(CollectiveId::Gamma, candidate, 0);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Gamma,
        ));
        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Gamma,
            &candidate
        ));
    });
}

#[test]
fn try_join_sweep_evicts_ineligible_incumbents() {
    TestState::build_and_execute(|| {
        // Alpha's min_members is 0, so the sweep can drain freely.
        // Two ineligible incumbents must be evicted before the join.
        let inc1 = U256::from(10);
        let inc2 = U256::from(20);
        seed_members(CollectiveId::Alpha, &[inc1, inc2]);
        // Incumbents have no eligibility marker → ineligible by default.

        let candidate = U256::from(15);
        set_eligible(CollectiveId::Alpha, candidate, true);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Alpha,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![candidate]
        );
        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberJoined {
                collective_id: CollectiveId::Alpha,
                who: candidate,
                evicted: vec![inc1, inc2],
            })
        );
        // OnMembersChanged outgoing is sorted (computed by
        // `compute_members_diff_sorted`).
        assert_eq!(
            take_members_changed_log().last(),
            Some(&MembersChangedCall {
                collective_id: CollectiveId::Alpha,
                incoming: vec![candidate],
                outgoing: vec![inc1, inc2],
            })
        );
    });
}

#[test]
fn try_join_sweep_respects_min_members_floor() {
    TestState::build_and_execute(|| {
        // Beta's min_members is 2, max 3. Fill with 3 ineligible incumbents.
        // The sweep can drop the collective to its floor (2) but no further,
        // so exactly ONE incumbent is evicted. With one slot freed and the
        // collective now under cap, the candidate joins without invoking
        // ranking on the remaining (ineligible) incumbents.
        let inc1 = U256::from(10);
        let inc2 = U256::from(20);
        let inc3 = U256::from(30);
        seed_members(CollectiveId::Beta, &[inc1, inc2, inc3]);

        let candidate = U256::from(25);
        set_eligible(CollectiveId::Beta, candidate, true);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Beta,
        ));

        // The first incumbent (head of the list) is the one evicted.
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Beta),
            vec![inc2, candidate, inc3]
        );
        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberJoined {
                collective_id: CollectiveId::Beta,
                who: candidate,
                evicted: vec![inc1],
            })
        );
    });
}

#[test]
fn try_join_sweep_then_rank_when_floor_blocks_full_sweep() {
    TestState::build_and_execute(|| {
        // Beta: min=2, max=3. Two eligible incumbents at the floor and one
        // higher-ranked ineligible incumbent above the floor. The sweep
        // evicts the ineligible incumbent (budget=1), freeing one slot, so
        // the candidate joins without ranking.
        //
        // This is distinct from the all-eligible case below, where the
        // sweep removes nobody and ranking decides.
        let inc1 = U256::from(10); // eligible, will stay
        let inc2 = U256::from(20); // eligible, will stay
        let inc3 = U256::from(30); // ineligible, evicted
        seed_members(CollectiveId::Beta, &[inc1, inc2, inc3]);
        set_eligible(CollectiveId::Beta, inc1, true);
        set_eligible(CollectiveId::Beta, inc2, true);

        let candidate = U256::from(25);
        set_eligible(CollectiveId::Beta, candidate, true);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Beta,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Beta),
            vec![inc1, inc2, candidate]
        );
    });
}

#[test]
fn try_join_falls_through_to_ranking_when_all_incumbents_eligible() {
    TestState::build_and_execute(|| {
        // Beta full and every incumbent eligible: sweep frees nothing, and
        // ranking must displace the lowest if the candidate outranks.
        let m1 = U256::from(10);
        let m2 = U256::from(20);
        let m3 = U256::from(30);
        seed_members(CollectiveId::Beta, &[m1, m2, m3]);
        for (m, r) in [(m1, 1u128), (m2, 2), (m3, 3)] {
            set_eligible(CollectiveId::Beta, m, true);
            set_rank(CollectiveId::Beta, m, r);
        }

        let candidate = U256::from(25);
        set_eligible(CollectiveId::Beta, candidate, true);
        set_rank(CollectiveId::Beta, candidate, 10);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Beta,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Beta),
            vec![m2, candidate, m3]
        );
        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberJoined {
                collective_id: CollectiveId::Beta,
                who: candidate,
                evicted: vec![m1],
            })
        );
    });
}

#[test]
fn try_join_full_with_unbounded_min_can_evict_lowest_ranked_after_partial_sweep() {
    TestState::build_and_execute(|| {
        // Alpha's min_members is 0 so the sweep is allowed to drain everyone;
        // the candidate has lower rank than every incumbent but the sweep
        // empties the collective, so admission succeeds without any rank
        // comparison.
        let incs: Vec<U256> = (1u64..=5).map(U256::from).collect();
        seed_members(CollectiveId::Alpha, &incs);
        for (i, m) in incs.iter().enumerate() {
            // Mark each incumbent as eligible only intermittently to make
            // sure the sweep handles mixed eligibility correctly. Even ones
            // stay, odd ones go.
            if i % 2 == 0 {
                set_eligible(CollectiveId::Alpha, *m, true);
                set_rank(CollectiveId::Alpha, *m, 100);
            }
        }

        let candidate = U256::from(99);
        set_eligible(CollectiveId::Alpha, candidate, true);
        set_rank(CollectiveId::Alpha, candidate, 0);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Alpha,
        ));

        // Survivors: the even-indexed members (indices 0,2,4 → ids 1,3,5).
        let expected: Vec<U256> = [1u64, 3, 5, 99].into_iter().map(U256::from).collect();
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            expected
        );
    });
}

#[test]
fn try_join_no_storage_write_on_failed_admission() {
    // `assert_noop` already checks the storage hash; this test additionally
    // proves the explicit invariants: members list unchanged, no events
    // emitted, no OnMembersChanged call.
    TestState::build_and_execute(|| {
        let inc = U256::from(10);
        seed_members(CollectiveId::Alpha, &[inc]);

        let candidate = U256::from(20);
        // Not eligible → noop.
        assert_noop!(
            MultiCollective::<Test>::try_join(
                RuntimeOrigin::signed(candidate),
                CollectiveId::Alpha,
            ),
            Error::<Test>::NotEligible
        );

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![inc]
        );
        assert!(take_members_changed_log().is_empty());
        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn try_join_sweep_takes_precedence_over_rank_comparison() {
    TestState::build_and_execute(|| {
        // Full collective with one ineligible member and two high-ranked
        // eligible members. The candidate's rank is *lower* than every
        // eligible incumbent's, but the sweep evicts the ineligible
        // incumbent first, freeing a slot, and the candidate joins without
        // ranking being consulted at all.
        let bad = U256::from(10);
        let good1 = U256::from(20);
        let good2 = U256::from(30);
        seed_members(CollectiveId::Alpha, &[bad, good1, good2]);
        set_eligible(CollectiveId::Alpha, good1, true);
        set_eligible(CollectiveId::Alpha, good2, true);
        set_rank(CollectiveId::Alpha, good1, u128::MAX);
        set_rank(CollectiveId::Alpha, good2, u128::MAX);

        let candidate = U256::from(25);
        set_eligible(CollectiveId::Alpha, candidate, true);
        set_rank(CollectiveId::Alpha, candidate, 0);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Alpha,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![good1, candidate, good2]
        );
    });
}

#[test]
fn try_join_emits_join_event_with_evicted_field_sorted() {
    TestState::build_and_execute(|| {
        // Two ineligible incumbents at distinct positions; the evicted list
        // in the event is expected to be sorted (ChangeMembers diff yields
        // sorted slices).
        let high = U256::from(40);
        let low = U256::from(15);
        seed_members(CollectiveId::Alpha, &[high, low]);

        let candidate = U256::from(25);
        set_eligible(CollectiveId::Alpha, candidate, true);

        assert_ok!(MultiCollective::<Test>::try_join(
            RuntimeOrigin::signed(candidate),
            CollectiveId::Alpha,
        ));

        assert_eq!(
            multi_collective_events().last().expect("event emitted"),
            &CollectiveEvent::MemberJoined {
                collective_id: CollectiveId::Alpha,
                who: candidate,
                evicted: vec![low, high],
            },
        );
    });
}

/// The pallet ships a `()` impl of `AdmissionPolicy` used as the default
/// for runtimes that don't opt into `try_join`. Exercise the trait default
/// surface so behaviour is locked in.
#[test]
fn admission_policy_unit_impl_rejects_and_zero_ranks() {
    use crate::AdmissionPolicy as AP;
    let any = U256::from(123);

    let (eligible, w) = <() as AP<U256, CollectiveId>>::is_eligible(CollectiveId::Alpha, &any);
    assert!(!eligible);
    assert_eq!(w, Weight::zero());

    let (eligible, _) = <() as AP<U256, CollectiveId>>::is_eligible(CollectiveId::Beta, &any);
    assert!(!eligible);

    let (rank, w) = <() as AP<U256, CollectiveId>>::rank(CollectiveId::Alpha, &any);
    assert_eq!(rank, 0u128);
    assert_eq!(w, Weight::zero());

    assert_eq!(
        <() as AP<U256, CollectiveId>>::is_eligible_weight(100),
        Weight::zero()
    );
    assert_eq!(
        <() as AP<U256, CollectiveId>>::rank_weight(100),
        Weight::zero()
    );
}
