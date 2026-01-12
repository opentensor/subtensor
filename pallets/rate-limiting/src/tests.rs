use frame_support::{assert_noop, assert_ok};
use sp_std::vec::Vec;

use crate::{
    CallGroups, CallReadOnly, Config, GroupMembers, GroupSharing, LastSeen, LimitSettingRules,
    Limits, RateLimit, RateLimitKind, RateLimitTarget, TransactionIdentifier, mock::*,
    pallet::Error,
};
use frame_support::traits::Get;

fn target(identifier: TransactionIdentifier) -> RateLimitTarget<GroupId> {
    RateLimitTarget::Transaction(identifier)
}

fn remark_call() -> RuntimeCall {
    RuntimeCall::System(frame_system::Call::<Test>::remark { remark: Vec::new() })
}

fn scoped_call() -> RuntimeCall {
    RuntimeCall::RateLimiting(RateLimitingCall::set_rate_limit {
        target: RateLimitTarget::Transaction(TransactionIdentifier::new(0, 0)),
        scope: Some(1),
        limit: RateLimitKind::Default,
    })
}

fn register(call: RuntimeCall, group: Option<GroupId>) -> TransactionIdentifier {
    let identifier = identifier_for(&call);
    assert_ok!(RateLimiting::register_call(
        RuntimeOrigin::root(),
        Box::new(call),
        group
    ));
    identifier
}

fn create_group(name: &[u8], sharing: GroupSharing) -> GroupId {
    assert_ok!(RateLimiting::create_group(
        RuntimeOrigin::root(),
        name.to_vec(),
        sharing,
    ));
    RateLimiting::next_group_id().saturating_sub(1)
}

fn last_event() -> RuntimeEvent {
    pop_last_event()
}

#[test]
fn set_rate_limit_respects_limit_setting_rule() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let tx_target = target(identifier);

        // Default rule is root-only.
        assert_noop!(
            RateLimiting::set_rate_limit(
                RuntimeOrigin::signed(1),
                tx_target,
                None,
                RateLimitKind::Exact(1),
            ),
            sp_runtime::DispatchError::BadOrigin
        );

        // Root updates the limit-setting rule for this transaction target.
        assert_ok!(RateLimiting::set_limit_setting_rule(
            RuntimeOrigin::root(),
            tx_target,
            LimitSettingRule::AnySigned,
        ));

        assert_eq!(
            LimitSettingRules::<Test, ()>::get(tx_target),
            LimitSettingRule::AnySigned
        );

        // Now any signed origin may set the limit for this target.
        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::signed(1),
            tx_target,
            None,
            RateLimitKind::Exact(7),
        ));
    });
}

#[test]
fn register_call_seeds_global_limit() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let tx_target = target(identifier);
        let stored = Limits::<Test, ()>::get(tx_target).expect("limit");
        assert!(matches!(stored, RateLimit::Global(RateLimitKind::Default)));

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::CallRegistered { transaction, .. })
            if transaction == identifier
        ));
    });
}

#[test]
fn register_call_seeds_scoped_limit() {
    new_test_ext().execute_with(|| {
        let identifier = register(scoped_call(), None);
        let tx_target = target(identifier);
        let stored = Limits::<Test, ()>::get(tx_target).expect("limit");
        match stored {
            RateLimit::Scoped(map) => {
                assert_eq!(map.get(&1u16), Some(&RateLimitKind::Default));
            }
            _ => panic!("expected scoped entry"),
        }

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::CallRegistered { transaction, scope, .. })
            if transaction == identifier && scope == Some(vec![1u16])
        ));
    });
}

#[test]
fn register_call_seeds_multi_scoped_limit() {
    new_test_ext().execute_with(|| {
        let call = RuntimeCall::RateLimiting(RateLimitingCall::set_rate_limit {
            target: RateLimitTarget::Transaction(TransactionIdentifier::new(0, 0)),
            scope: None,
            limit: RateLimitKind::Exact(42),
        });
        let identifier = register(call, None);
        let tx_target = target(identifier);
        let stored = Limits::<Test, ()>::get(tx_target).expect("limit");
        match stored {
            RateLimit::Scoped(map) => {
                assert_eq!(map.get(&42u16), Some(&RateLimitKind::Default));
                assert_eq!(map.get(&43u16), Some(&RateLimitKind::Default));
            }
            _ => panic!("expected scoped entry"),
        }

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::CallRegistered { transaction, scope, .. })
            if transaction == identifier && scope == Some(vec![42u16, 43u16])
        ));
    });
}

#[test]
fn set_rate_limit_updates_transaction_target() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let tx_target = target(identifier);
        let limit = RateLimitKind::Exact(9);
        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            tx_target,
            None,
            limit,
        ));
        let stored = Limits::<Test, ()>::get(tx_target).expect("limit");
        assert!(matches!(stored, RateLimit::Global(RateLimitKind::Exact(9))));

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::RateLimitSet {
                target: RateLimitTarget::Transaction(t),
                limit: RateLimitKind::Exact(9),
                ..
            }) if t == identifier
        ));
    });
}

#[test]
fn set_rate_limit_requires_registration_and_group_targeting() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let target = target(identifier);

        // Unregistered call.
        let unknown = TransactionIdentifier::new(99, 0);
        assert_noop!(
            RateLimiting::set_rate_limit(
                RuntimeOrigin::root(),
                RateLimitTarget::Transaction(unknown),
                None,
                RateLimitKind::Exact(1),
            ),
            Error::<Test>::CallNotRegistered
        );

        // Group requires targeting the group.
        let group = create_group(b"cfg", GroupSharing::ConfigAndUsage);
        assert_ok!(RateLimiting::assign_call_to_group(
            RuntimeOrigin::root(),
            identifier,
            group,
            false,
        ));
        assert_noop!(
            RateLimiting::set_rate_limit(
                RuntimeOrigin::root(),
                target,
                None,
                RateLimitKind::Exact(2),
            ),
            Error::<Test>::MustTargetGroup
        );
    });
}

#[test]
fn set_rate_limit_respects_group_config_sharing() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let group = create_group(b"test", GroupSharing::ConfigAndUsage);
        // Consume group creation event to keep ordering predictable.
        let created = last_event();
        assert!(matches!(
            created,
            RuntimeEvent::RateLimiting(crate::Event::GroupCreated { group: g, .. }) if g == group
        ));
        assert_ok!(RateLimiting::assign_call_to_group(
            RuntimeOrigin::root(),
            identifier,
            group,
            false,
        ));
        let events: Vec<_> = System::events()
            .into_iter()
            .map(|e| e.event)
            .filter(|evt| matches!(evt, RuntimeEvent::RateLimiting(_)))
            .collect();
        assert!(events.iter().any(|evt| {
            matches!(
                evt,
                RuntimeEvent::RateLimiting(crate::Event::CallReadOnlyUpdated {
                    transaction,
                    group: g,
                    read_only: false,
                }) if *transaction == identifier && *g == group
            )
        }));
        assert!(events.iter().any(|evt| {
            matches!(
                evt,
                RuntimeEvent::RateLimiting(crate::Event::CallGroupUpdated {
                    transaction,
                    group: Some(g),
                }) if *transaction == identifier && *g == group
            )
        }));
        assert_noop!(
            RateLimiting::set_rate_limit(
                RuntimeOrigin::root(),
                RateLimitTarget::Transaction(identifier),
                None,
                RateLimitKind::Exact(5),
            ),
            Error::<Test>::MustTargetGroup
        );
    });
}

#[test]
fn assign_and_remove_group_membership() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let group = create_group(b"team", GroupSharing::UsageOnly);
        assert_ok!(RateLimiting::assign_call_to_group(
            RuntimeOrigin::root(),
            identifier,
            group,
            false,
        ));
        assert_eq!(CallGroups::<Test, ()>::get(identifier), Some(group));
        assert_eq!(CallReadOnly::<Test, ()>::get(identifier), Some(false));
        assert!(GroupMembers::<Test, ()>::get(group).contains(&identifier));
        assert_ok!(RateLimiting::remove_call_from_group(
            RuntimeOrigin::root(),
            identifier,
        ));
        assert!(CallGroups::<Test, ()>::get(identifier).is_none());

        // Last event should signal removal.
        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::CallGroupUpdated { transaction, group: None })
            if transaction == identifier
        ));
    });
}

#[test]
fn set_rate_limit_on_group_updates_storage() {
    new_test_ext().execute_with(|| {
        let group = create_group(b"grp", GroupSharing::ConfigOnly);
        let target = RateLimitTarget::Group(group);
        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            target,
            None,
            RateLimitKind::Exact(3),
        ));
        assert!(matches!(
            Limits::<Test, ()>::get(target),
            Some(RateLimit::Global(RateLimitKind::Exact(3)))
        ));

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::RateLimitSet {
                target: RateLimitTarget::Group(g),
                limit: RateLimitKind::Exact(3),
                ..
            }) if g == group
        ));
    });
}

#[test]
fn create_and_delete_group_emit_events() {
    new_test_ext().execute_with(|| {
        assert_ok!(RateLimiting::create_group(
            RuntimeOrigin::root(),
            b"ev".to_vec(),
            GroupSharing::UsageOnly,
        ));
        let group = RateLimiting::next_group_id().saturating_sub(1);
        let created = last_event();
        assert!(matches!(
            created,
            RuntimeEvent::RateLimiting(crate::Event::GroupCreated { group: g, .. }) if g == group
        ));

        assert_ok!(RateLimiting::delete_group(RuntimeOrigin::root(), group));
        let deleted = last_event();
        assert!(matches!(
            deleted,
            RuntimeEvent::RateLimiting(crate::Event::GroupDeleted { group: g }) if g == group
        ));
    });
}

#[test]
fn deregister_call_scope_removes_entry() {
    new_test_ext().execute_with(|| {
        let identifier = register(scoped_call(), None);
        let tx_target = target(identifier);
        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            tx_target,
            Some(2u16),
            RateLimitKind::Exact(4),
        ));
        LastSeen::<Test, ()>::insert(tx_target, Some(9u16), 10);
        assert_ok!(RateLimiting::deregister_call(
            RuntimeOrigin::root(),
            identifier,
            Some(2u16),
            false,
        ));
        match Limits::<Test, ()>::get(tx_target) {
            Some(RateLimit::Scoped(map)) => {
                assert!(map.contains_key(&1u16));
                assert!(!map.contains_key(&2u16));
            }
            other => panic!("unexpected config: {:?}", other),
        }
        // usage remains intact when clear_usage is false
        assert_eq!(LastSeen::<Test, ()>::get(tx_target, Some(9u16)), Some(10));

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::CallDeregistered {
                target,
                transaction: Some(t),
                scope: Some(sc),
                ..
            }) if target == tx_target && t == identifier && sc == 2u16
        ));

        // No group assigned in this test.
        assert!(CallGroups::<Test, ()>::get(identifier).is_none());
    });
}

#[test]
fn register_call_rejects_duplicates_and_unknown_group() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        // Duplicate should fail.
        assert_noop!(
            RateLimiting::register_call(RuntimeOrigin::root(), Box::new(remark_call()), None),
            Error::<Test>::CallAlreadyRegistered
        );

        // Unknown group should fail.
        assert_noop!(
            RateLimiting::register_call(RuntimeOrigin::root(), Box::new(scoped_call()), Some(99)),
            Error::<Test>::UnknownGroup
        );

        assert!(Limits::<Test, ()>::contains_key(target(identifier)));
    });
}

#[test]
fn group_name_limits_and_uniqueness_enforced() {
    new_test_ext().execute_with(|| {
        // Overlong name.
        let max_name = <<Test as Config>::MaxGroupNameLength as Get<u32>>::get() as usize;
        let long_name = vec![0u8; max_name + 1];
        assert_noop!(
            RateLimiting::create_group(RuntimeOrigin::root(), long_name, GroupSharing::UsageOnly),
            Error::<Test>::GroupNameTooLong
        );

        // Duplicate names rejected on create and update.
        let first = create_group(b"alpha", GroupSharing::UsageOnly);
        let second = create_group(b"beta", GroupSharing::UsageOnly);

        assert_noop!(
            RateLimiting::create_group(
                RuntimeOrigin::root(),
                b"alpha".to_vec(),
                GroupSharing::UsageOnly
            ),
            Error::<Test>::DuplicateGroupName
        );

        assert_noop!(
            RateLimiting::update_group(
                RuntimeOrigin::root(),
                second,
                Some(b"alpha".to_vec()),
                None
            ),
            Error::<Test>::DuplicateGroupName
        );

        // Unknown group update.
        assert_noop!(
            RateLimiting::update_group(RuntimeOrigin::root(), 99, None, None),
            Error::<Test>::UnknownGroup
        );

        assert_eq!(
            RateLimiting::groups(first).unwrap().name.into_inner(),
            b"alpha".to_vec()
        );

        // Updating first group emits event.
        assert_ok!(RateLimiting::update_group(
            RuntimeOrigin::root(),
            first,
            Some(b"gamma".to_vec()),
            None,
        ));
        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::GroupUpdated { group, .. }) if group == first
        ));
    });
}

#[test]
fn group_member_limit_and_removal_errors() {
    new_test_ext().execute_with(|| {
        let group = create_group(b"cap", GroupSharing::UsageOnly);

        let max_members = <<Test as Config>::MaxGroupMembers as Get<u32>>::get();
        GroupMembers::<Test, ()>::mutate(group, |members| {
            for i in 0..max_members {
                let _ = members.try_insert(TransactionIdentifier::new(0, (i + 1) as u8));
            }
        });

        // Next insert should fail.
        let extra = register(remark_call(), None);
        assert_noop!(
            RateLimiting::assign_call_to_group(RuntimeOrigin::root(), extra, group, false),
            Error::<Test>::GroupMemberLimitExceeded
        );

        // Removing a call not in a group errors.
        assert_noop!(
            RateLimiting::remove_call_from_group(RuntimeOrigin::root(), extra),
            Error::<Test>::CallNotInGroup
        );
    });
}

#[test]
fn set_call_read_only_requires_group() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        assert_noop!(
            RateLimiting::set_call_read_only(RuntimeOrigin::root(), identifier, true),
            Error::<Test>::CallNotInGroup
        );
    });
}

#[test]
fn set_call_read_only_updates_assignment_and_emits_event() {
    new_test_ext().execute_with(|| {
        let group = create_group(b"ro", GroupSharing::UsageOnly);
        let identifier = register(remark_call(), None);
        assert_ok!(RateLimiting::assign_call_to_group(
            RuntimeOrigin::root(),
            identifier,
            group,
            false,
        ));

        assert_ok!(RateLimiting::set_call_read_only(
            RuntimeOrigin::root(),
            identifier,
            true
        ));

        assert_eq!(CallGroups::<Test, ()>::get(identifier), Some(group));
        assert_eq!(CallReadOnly::<Test, ()>::get(identifier), Some(true));

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::CallReadOnlyUpdated {
                transaction,
                group: g,
                read_only: true,
            }) if transaction == identifier && g == group
        ));
    });
}

#[test]
fn cannot_delete_group_in_use_or_unknown() {
    new_test_ext().execute_with(|| {
        let group = create_group(b"busy", GroupSharing::ConfigOnly);
        let identifier = register(remark_call(), Some(group));
        let target = RateLimitTarget::Group(group);
        Limits::<Test, ()>::insert(target, RateLimit::global(RateLimitKind::Exact(1)));
        LastSeen::<Test, ()>::insert(target, None::<UsageKey>, 10);

        // Remove member so only config/usage keep the group in-use.
        assert_ok!(RateLimiting::remove_call_from_group(
            RuntimeOrigin::root(),
            identifier
        ));

        // Cannot delete when in use.
        assert_noop!(
            RateLimiting::delete_group(RuntimeOrigin::root(), group),
            Error::<Test>::GroupInUse
        );

        // Clear state then delete.
        Limits::<Test, ()>::remove(target);
        let _ = LastSeen::<Test, ()>::clear_prefix(&target, u32::MAX, None);
        assert_ok!(RateLimiting::delete_group(RuntimeOrigin::root(), group));

        // Unknown group.
        assert_noop!(
            RateLimiting::delete_group(RuntimeOrigin::root(), 999),
            Error::<Test>::UnknownGroup
        );
    });
}

#[test]
fn deregister_call_clears_registration() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let tx_target = target(identifier);
        LastSeen::<Test, ()>::insert(tx_target, None::<UsageKey>, 5);
        assert_ok!(RateLimiting::deregister_call(
            RuntimeOrigin::root(),
            identifier,
            None,
            true,
        ));
        assert!(Limits::<Test, ()>::get(tx_target).is_none());
        assert!(LastSeen::<Test, ()>::get(tx_target, None::<UsageKey>).is_none());
        assert!(CallGroups::<Test, ()>::get(identifier).is_none());

        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::CallDeregistered {
                target,
                transaction: Some(t),
                scope: None,
                ..
            }) if target == tx_target && t == identifier
        ));
    });
}

#[test]
fn deregister_errors_for_unknown_or_missing_scope() {
    new_test_ext().execute_with(|| {
        let unknown = TransactionIdentifier::new(10, 1);
        assert_noop!(
            RateLimiting::deregister_call(RuntimeOrigin::root(), unknown, None, true),
            Error::<Test>::CallNotRegistered
        );

        let identifier = register(scoped_call(), None);
        let tx_target = target(identifier);
        // Removing a non-existent scoped entry fails.
        assert_noop!(
            RateLimiting::deregister_call(RuntimeOrigin::root(), identifier, Some(99u16), false),
            Error::<Test>::MissingRateLimit
        );

        // Removing the last scoped entry clears Limits and LastSeen.
        LastSeen::<Test, ()>::insert(tx_target, Some(1u16), 5);
        assert_ok!(RateLimiting::deregister_call(
            RuntimeOrigin::root(),
            identifier,
            Some(1u16),
            true,
        ));
        assert!(Limits::<Test, ()>::get(tx_target).is_none());
        assert!(LastSeen::<Test, ()>::get(tx_target, Some(1u16)).is_none());
    });
}

#[test]
fn is_within_limit_detects_rate_limited_scope() {
    new_test_ext().execute_with(|| {
        let call = scoped_call();
        let identifier = identifier_for(&call);
        let tx_target = target(identifier);
        Limits::<Test, ()>::insert(
            tx_target,
            RateLimit::scoped_single(1u16, RateLimitKind::Exact(3)),
        );
        LastSeen::<Test, ()>::insert(tx_target, Some(1u16), 9);
        System::set_block_number(11);
        let result = RateLimiting::is_within_limit(
            &RuntimeOrigin::signed(1),
            &call,
            &identifier,
            &Some(vec![1u16]),
            &Some(1u16),
        )
        .expect("ok");
        assert!(!result);
    });
}

#[test]
fn migrate_usage_key_tracks_scope() {
    new_test_ext().execute_with(|| {
        let call = scoped_call();
        let identifier = identifier_for(&call);
        let tx_target = target(identifier);
        LastSeen::<Test, ()>::insert(tx_target, Some(6u16), 10);
        assert!(RateLimiting::migrate_usage_key(
            tx_target,
            Some(6u16),
            Some(7u16)
        ));
        assert_eq!(LastSeen::<Test, ()>::get(tx_target, Some(7u16)), Some(10));
    });
}

#[test]
fn migrate_limit_scope_covers_transitions() {
    new_test_ext().execute_with(|| {
        let identifier = register(remark_call(), None);
        let tx_target = target(identifier);

        // global -> scoped
        assert!(RateLimiting::migrate_limit_scope(
            tx_target,
            None,
            Some(42u16)
        ));
        match Limits::<Test, ()>::get(tx_target) {
            Some(RateLimit::Scoped(map)) => {
                assert_eq!(map.get(&42u16), Some(&RateLimitKind::Default))
            }
            other => panic!("unexpected config: {:?}", other),
        }

        // scoped -> scoped
        assert!(RateLimiting::migrate_limit_scope(
            tx_target,
            Some(42u16),
            Some(43u16)
        ));
        match Limits::<Test, ()>::get(tx_target) {
            Some(RateLimit::Scoped(map)) => {
                assert_eq!(map.get(&43u16), Some(&RateLimitKind::Default))
            }
            other => panic!("unexpected config: {:?}", other),
        }

        // scoped -> global (only entry)
        assert!(RateLimiting::migrate_limit_scope(
            tx_target,
            Some(43u16),
            None
        ));
        assert!(matches!(
            Limits::<Test, ()>::get(tx_target),
            Some(RateLimit::Global(RateLimitKind::Default))
        ));

        // no-op when scopes identical
        assert!(RateLimiting::migrate_limit_scope(tx_target, None, None));
    });
}

#[test]
fn set_default_limit_updates_span_and_resolves_in_enforcement() {
    new_test_ext().execute_with(|| {
        assert_eq!(RateLimiting::default_limit(), 0);
        assert_ok!(RateLimiting::set_default_rate_limit(
            RuntimeOrigin::root(),
            5
        ));
        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::RateLimiting(crate::Event::DefaultRateLimitSet { block_span: 5 })
        ));
        assert_eq!(RateLimiting::default_limit(), 5);

        let call = remark_call();
        let identifier = register(call.clone(), None);
        let tx_target = target(identifier);

        System::set_block_number(10);
        // No last-seen yet, first call passes.
        assert!(
            RateLimiting::is_within_limit(
                &RuntimeOrigin::signed(1),
                &call,
                &identifier,
                &None,
                &None,
            )
            .unwrap()
        );

        LastSeen::<Test, ()>::insert(tx_target, None::<UsageKey>, 12);
        System::set_block_number(15);
        // Span 5 should block when delta < 5.
        assert!(
            !RateLimiting::is_within_limit(
                &RuntimeOrigin::signed(1),
                &call,
                &identifier,
                &None,
                &None,
            )
            .unwrap()
        );
    });
}

#[test]
fn limit_for_call_names_prefers_scoped_value() {
    new_test_ext().execute_with(|| {
        let call = scoped_call();
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(
            target(identifier),
            RateLimit::scoped_single(9u16, RateLimitKind::Exact(8)),
        );
        let fetched =
            RateLimiting::limit_for_call_names("RateLimiting", "set_rate_limit", Some(9u16))
                .expect("limit");
        assert_eq!(fetched, RateLimitKind::Exact(8));
    });
}
