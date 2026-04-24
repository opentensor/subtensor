#![allow(clippy::unwrap_used, clippy::expect_used)]

use frame_support::{assert_noop, assert_ok, traits::Hooks};
use sp_core::U256;
use sp_runtime::DispatchError;

use crate::{
    Collective, CollectiveInfo, CollectiveInspect, CollectivesInfo, Error,
    Event as CollectiveEvent, Pallet as MultiCollective, mock::*,
};

// -------- Section 1: Environment --------

/// Verifies the mock runtime exposes the expected set of collectives, each
/// with the per-collective config the tests rely on, and that `Members`
/// storage starts empty for every collective.
#[test]
fn environment_works() {
    TestState::build_and_execute(|| {
        for id in [
            CollectiveId::Alpha,
            CollectiveId::Beta,
            CollectiveId::Gamma,
            CollectiveId::Delta,
        ] {
            assert!(
                MultiCollective::<Test>::members_of(id).is_empty(),
                "{:?} should start empty",
                id,
            );
            assert_eq!(MultiCollective::<Test>::member_count(id), 0);
        }

        let alpha = TestCollectives::info(CollectiveId::Alpha).expect("Alpha known");
        assert_eq!(alpha.min_members, 0);
        assert_eq!(alpha.max_members, Some(5));
        assert_eq!(alpha.term_duration, None);

        let beta = TestCollectives::info(CollectiveId::Beta).expect("Beta known");
        assert_eq!(beta.min_members, 2);
        assert_eq!(beta.max_members, Some(3));
        assert_eq!(beta.term_duration, Some(100));

        let gamma = TestCollectives::info(CollectiveId::Gamma).expect("Gamma known");
        assert_eq!(gamma.min_members, 0);
        assert_eq!(gamma.max_members, None);
        assert_eq!(gamma.term_duration, None);

        let delta = TestCollectives::info(CollectiveId::Delta).expect("Delta known");
        assert_eq!(delta.min_members, 1);
        assert_eq!(delta.max_members, Some(32));
        assert_eq!(delta.term_duration, Some(50));

        assert!(multi_collective_events().is_empty());
        assert!(take_new_term_log().is_empty());
    });
}

// -------- Section 2: add_member --------

#[test]
fn add_member_appends_to_empty_collective() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);

        assert_ok!(MultiCollective::<Test>::add_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            alice,
        ));

        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![alice]
        );
        assert_eq!(
            MultiCollective::<Test>::member_count(CollectiveId::Alpha),
            1
        );
        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Alpha,
            &alice
        ));

        assert_eq!(
            multi_collective_events(),
            vec![CollectiveEvent::MemberAdded {
                collective_id: CollectiveId::Alpha,
                who: alice,
            }]
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

        // Only one MemberAdded event — the failing call produced nothing.
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
        // Exactly five events — no event from the failing 6th.
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

        // 33rd add fails via `try_push` (BoundedVec bound) rather than the info cap.
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

// -------- Section 3: remove_member --------

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

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MemberRemoved {
                collective_id: CollectiveId::Alpha,
                who: bob,
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

// -------- Section 4: swap_member --------

#[test]
fn swap_member_happy_path() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        let charlie = U256::from(3);
        let dave = U256::from(4);

        for who in [alice, bob, charlie] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            bob,
            dave,
        ));

        // Dave takes bob's slot at index 1 — position preserved.
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![alice, dave, charlie]
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
    });
}

#[test]
fn swap_member_preserves_position_on_head_and_tail() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);
        let c = U256::from(3);
        let x = U256::from(10);
        let y = U256::from(11);

        for who in [a, b, c] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }

        // Swap head slot.
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            a,
            x,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![x, b, c]
        );

        // Swap tail slot.
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            c,
            y,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![x, b, y]
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
        // check (`!contains(add)`) rejects because add is already present —
        // as it is `remove` itself. Records current behavior; "swap with
        // self" is a no-op the pallet refuses.
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

#[test]
fn swap_member_works_at_min_bound() {
    TestState::build_and_execute(|| {
        // Beta has min_members = 2. Seed exactly at the floor.
        let alice = U256::from(1);
        let bob = U256::from(2);
        let carol = U256::from(3);

        for who in [alice, bob] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Beta,
                who,
            ));
        }

        // Count-invariant swap is allowed even at min — swap doesn't go
        // through the `TooFewMembers` check.
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
    });
}

#[test]
fn swap_member_works_at_max_bound() {
    TestState::build_and_execute(|| {
        // Beta has max_members = 3. Seed exactly at the ceiling.
        let alice = U256::from(1);
        let bob = U256::from(2);
        let carol = U256::from(3);
        let dave = U256::from(4);

        for who in [alice, bob, carol] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Beta,
                who,
            ));
        }

        // Same count-invariance: swap at max is allowed.
        assert_ok!(MultiCollective::<Test>::swap_member(
            RuntimeOrigin::root(),
            CollectiveId::Beta,
            alice,
            dave,
        ));

        assert_eq!(MultiCollective::<Test>::member_count(CollectiveId::Beta), 3);
        assert!(!MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &alice
        ));
        assert!(MultiCollective::<Test>::is_member(
            CollectiveId::Beta,
            &dave
        ));
    });
}

// -------- Section 5: reset_members --------

#[test]
fn reset_members_replaces_list() {
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

        assert_ok!(MultiCollective::<Test>::reset_members(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            vec![c, d, e],
        ));

        // Storage is the new list, in the passed order.
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![c, d, e]
        );
        assert!(!MultiCollective::<Test>::is_member(CollectiveId::Alpha, &a));
        assert!(!MultiCollective::<Test>::is_member(CollectiveId::Alpha, &b));

        assert_eq!(
            multi_collective_events().last(),
            Some(&CollectiveEvent::MembersReset {
                collective_id: CollectiveId::Alpha,
                members: vec![c, d, e],
            })
        );
    });
}

#[test]
fn reset_members_handles_overlap() {
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
        assert_ok!(MultiCollective::<Test>::reset_members(
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
            Some(&CollectiveEvent::MembersReset {
                collective_id: CollectiveId::Alpha,
                members: vec![b, c, d],
            })
        );
    });
}

#[test]
fn reset_members_requires_origin() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::reset_members(
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
fn reset_members_fails_for_unknown_collective() {
    TestState::build_and_execute(|| {
        assert_noop!(
            MultiCollective::<Test>::reset_members(
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
fn reset_members_rejects_too_few() {
    TestState::build_and_execute(|| {
        // Beta declares min_members = 2.
        assert_noop!(
            MultiCollective::<Test>::reset_members(
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
fn reset_members_rejects_too_many_via_info() {
    TestState::build_and_execute(|| {
        // Beta declares max_members = Some(3); four accounts is one over.
        let list: Vec<U256> = (1..=4u32).map(U256::from).collect();
        assert_noop!(
            MultiCollective::<Test>::reset_members(RuntimeOrigin::root(), CollectiveId::Beta, list,),
            Error::<Test>::TooManyMembers
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Beta).is_empty());
        assert!(multi_collective_events().is_empty());
    });
}

#[test]
fn reset_members_rejects_too_many_via_storage() {
    TestState::build_and_execute(|| {
        // Gamma's info.max_members is None; only T::MaxMembers = 32 applies.
        // 33 accounts exceed the BoundedVec bound, caught by try_from.
        let list: Vec<U256> = (1..=33u32).map(U256::from).collect();
        assert_noop!(
            MultiCollective::<Test>::reset_members(
                RuntimeOrigin::root(),
                CollectiveId::Gamma,
                list,
            ),
            Error::<Test>::TooManyMembers
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Gamma).is_empty());
    });
}

#[test]
fn reset_members_rejects_duplicates() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);

        assert_noop!(
            MultiCollective::<Test>::reset_members(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                vec![a, b, a],
            ),
            Error::<Test>::DuplicateAccounts
        );

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Alpha).is_empty());
    });
}

/// Reset with a list identical to the current membership still emits a
/// `MembersReset` event — the pallet doesn't short-circuit no-op resets.
/// Pinned so downstream consumers know they must tolerate empty-diff calls.
#[test]
fn reset_members_noop_still_fires_event() {
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

        assert_ok!(MultiCollective::<Test>::reset_members(
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
            Some(&CollectiveEvent::MembersReset {
                collective_id: CollectiveId::Alpha,
                members: vec![a, b],
            })
        );
    });
}

// -------- Section 6: on_initialize / term rotation --------

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
        // Delta (td=50) first fires at block 50.
        run_to_block(50);
        assert_eq!(take_new_term_log(), vec![CollectiveId::Delta]);

        // 51..=99: no boundary for Delta (next at 100) or Beta (first at 100).
        run_to_block(99);
        assert!(take_new_term_log().is_empty());
    });
}

#[test]
fn on_initialize_fires_all_matching_collectives() {
    TestState::build_and_execute(|| {
        // Advance through the first shared boundary at block 100. Delta fires
        // at 50, then both Beta and Delta fire at 100. Iteration order in
        // `TestCollectives` is [Alpha, Beta, Gamma, Delta] — so within block
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

// -------- Section 7: CollectiveInspect --------

#[test]
fn inspect_members_of_returns_current_list() {
    TestState::build_and_execute(|| {
        let a = U256::from(1);
        let b = U256::from(2);
        let c = U256::from(3);

        assert!(MultiCollective::<Test>::members_of(CollectiveId::Alpha).is_empty());

        for who in [a, b, c] {
            assert_ok!(MultiCollective::<Test>::add_member(
                RuntimeOrigin::root(),
                CollectiveId::Alpha,
                who,
            ));
        }
        // Insertion order preserved on add.
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![a, b, c]
        );

        // `retain` keeps relative order on remove.
        assert_ok!(MultiCollective::<Test>::remove_member(
            RuntimeOrigin::root(),
            CollectiveId::Alpha,
            b,
        ));
        assert_eq!(
            MultiCollective::<Test>::members_of(CollectiveId::Alpha),
            vec![a, c]
        );
    });
}

#[test]
fn inspect_is_member_basic() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let mallory = U256::from(999);

        // Empty collective — no membership.
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

        // Reset replaces wholesale — count reflects the new list length.
        assert_ok!(MultiCollective::<Test>::reset_members(
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

// -------- Section 8: integrity_test --------
//
// Test 42 (`integrity_test_passes_on_valid_config`) is implicit — the main
// mock's auto-generated `mock::__construct_runtime_integrity_test::runtime_integrity_tests`
// calls `integrity_test()` with the default (valid) `TestCollectives` on every
// `cargo test` run. It appears in the test output as "test mock::...runtime_integrity_tests ... ok".

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
            // min > max — the collective can never satisfy both.
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
            // Some(0) silently disables rotations — integrity_test rejects it.
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
