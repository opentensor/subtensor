use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

use crate::{DefaultLimit, LastSeen, Limits, RateLimit, RateLimitKind, mock::*, pallet::Error};

#[test]
fn limit_for_call_names_returns_none_if_not_set() {
    new_test_ext().execute_with(|| {
        assert!(
            RateLimiting::limit_for_call_names("RateLimiting", "set_default_rate_limit", None)
                .is_none()
        );
    });
}

#[test]
fn limit_for_call_names_returns_stored_limit() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(identifier, RateLimit::global(RateLimitKind::Exact(7)));

        let fetched =
            RateLimiting::limit_for_call_names("RateLimiting", "set_default_rate_limit", None)
                .expect("limit should exist");
        assert_eq!(fetched, RateLimitKind::Exact(7));
    });
}

#[test]
fn limit_for_call_names_prefers_scope_specific_limit() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(
            identifier,
            RateLimit::scoped_single(5u16, RateLimitKind::Exact(8)),
        );

        let fetched =
            RateLimiting::limit_for_call_names("RateLimiting", "set_default_rate_limit", Some(5))
                .expect("limit should exist");
        assert_eq!(fetched, RateLimitKind::Exact(8));

        assert!(
            RateLimiting::limit_for_call_names("RateLimiting", "set_default_rate_limit", Some(1))
                .is_none()
        );
    });
}

#[test]
fn resolved_limit_for_call_names_resolves_default_value() {
    new_test_ext().execute_with(|| {
        DefaultLimit::<Test, ()>::put(3);
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(identifier, RateLimit::global(RateLimitKind::Default));

        let resolved = RateLimiting::resolved_limit_for_call_names(
            "RateLimiting",
            "set_default_rate_limit",
            None,
        )
        .expect("resolved limit");
        assert_eq!(resolved, 3);
    });
}

#[test]
fn resolved_limit_for_call_names_prefers_scope_specific_value() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        let mut map = BTreeMap::new();
        map.insert(6u16, RateLimitKind::Exact(9));
        map.insert(2u16, RateLimitKind::Exact(4));
        Limits::<Test, ()>::insert(identifier, RateLimit::Scoped(map));

        let resolved = RateLimiting::resolved_limit_for_call_names(
            "RateLimiting",
            "set_default_rate_limit",
            Some(6),
        )
        .expect("resolved limit");
        assert_eq!(resolved, 9);

        assert!(
            RateLimiting::resolved_limit_for_call_names(
                "RateLimiting",
                "set_default_rate_limit",
                Some(1),
            )
            .is_none()
        );
    });
}

#[test]
fn resolved_limit_for_call_names_returns_none_when_unset() {
    new_test_ext().execute_with(|| {
        assert!(
            RateLimiting::resolved_limit_for_call_names(
                "RateLimiting",
                "set_default_rate_limit",
                None,
            )
            .is_none()
        );
    });
}

#[test]
fn is_within_limit_is_true_when_no_limit() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);

        let result = RateLimiting::is_within_limit(&identifier, &None, &None);
        assert_eq!(result.expect("no error expected"), true);
    });
}

#[test]
fn is_within_limit_false_when_rate_limited() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(
            identifier,
            RateLimit::scoped_single(1 as LimitScope, RateLimitKind::Exact(5)),
        );
        LastSeen::<Test, ()>::insert(identifier, Some(1 as UsageKey), 9);

        System::set_block_number(13);

        let within = RateLimiting::is_within_limit(
            &identifier,
            &Some(1 as LimitScope),
            &Some(1 as UsageKey),
        )
        .expect("call succeeds");
        assert!(!within);
    });
}

#[test]
fn is_within_limit_true_after_required_span() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(
            identifier,
            RateLimit::scoped_single(2 as LimitScope, RateLimitKind::Exact(5)),
        );
        LastSeen::<Test, ()>::insert(identifier, Some(2 as UsageKey), 10);

        System::set_block_number(20);

        let within = RateLimiting::is_within_limit(
            &identifier,
            &Some(2 as LimitScope),
            &Some(2 as UsageKey),
        )
        .expect("call succeeds");
        assert!(within);
    });
}

#[test]
fn set_rate_limit_updates_storage_and_emits_event() {
    new_test_ext().execute_with(|| {
        System::reset_events();

        let target_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let limit = RateLimitKind::Exact(9);

        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
            limit,
        ));

        let identifier = identifier_for(&target_call);
        assert_eq!(
            Limits::<Test, ()>::get(identifier),
            Some(RateLimit::scoped_single(0, limit))
        );

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::RateLimitSet {
                transaction,
                scope,
                limit: emitted_limit,
                pallet,
                extrinsic,
            }) => {
                assert_eq!(transaction, identifier);
                assert_eq!(scope, Some(0));
                assert_eq!(emitted_limit, limit);
                assert_eq!(pallet, b"RateLimiting".to_vec());
                assert_eq!(extrinsic, b"set_default_rate_limit".to_vec());
            }
            other => panic!("unexpected event: {:?}", other),
        }
    });
}

#[test]
fn set_rate_limit_stores_global_when_scope_absent() {
    new_test_ext().execute_with(|| {
        System::reset_events();

        let target_call =
            RuntimeCall::System(frame_system::Call::<Test>::remark { remark: Vec::new() });
        let limit = RateLimitKind::Exact(11);

        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
            limit,
        ));

        let identifier = identifier_for(&target_call);
        assert_eq!(
            Limits::<Test, ()>::get(identifier),
            Some(RateLimit::global(limit))
        );

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::RateLimitSet {
                transaction,
                scope,
                limit: emitted_limit,
                pallet,
                extrinsic,
            }) => {
                assert_eq!(transaction, identifier);
                assert_eq!(scope, None);
                assert_eq!(emitted_limit, limit);
                assert_eq!(pallet, b"System".to_vec());
                assert_eq!(extrinsic, b"remark".to_vec());
            }
            other => panic!("unexpected event: {:?}", other),
        }
    });
}

#[test]
fn set_rate_limit_requires_root() {
    new_test_ext().execute_with(|| {
        let target_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });

        assert_noop!(
            RateLimiting::set_rate_limit(
                RuntimeOrigin::signed(1),
                Box::new(target_call),
                RateLimitKind::Exact(1),
            ),
            BadOrigin
        );
    });
}

#[test]
fn set_rate_limit_accepts_default_variant() {
    new_test_ext().execute_with(|| {
        let target_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });

        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
            RateLimitKind::Default,
        ));

        let identifier = identifier_for(&target_call);
        assert_eq!(
            Limits::<Test, ()>::get(identifier),
            Some(RateLimit::scoped_single(0, RateLimitKind::Default))
        );
    });
}

#[test]
fn clear_rate_limit_removes_entry_and_emits_event() {
    new_test_ext().execute_with(|| {
        System::reset_events();

        let target_call =
            RuntimeCall::System(frame_system::Call::<Test>::remark { remark: Vec::new() });
        let identifier = identifier_for(&target_call);
        Limits::<Test, ()>::insert(identifier, RateLimit::global(RateLimitKind::Exact(4)));

        assert_ok!(RateLimiting::clear_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
        ));

        assert!(Limits::<Test, ()>::get(identifier).is_none());

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::RateLimitCleared {
                transaction,
                scope,
                pallet,
                extrinsic,
            }) => {
                assert_eq!(transaction, identifier);
                assert_eq!(scope, None);
                assert_eq!(pallet, b"System".to_vec());
                assert_eq!(extrinsic, b"remark".to_vec());
            }
            other => panic!("unexpected event: {:?}", other),
        }
    });
}

#[test]
fn clear_rate_limit_fails_when_missing() {
    new_test_ext().execute_with(|| {
        let target_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });

        assert_noop!(
            RateLimiting::clear_rate_limit(RuntimeOrigin::root(), Box::new(target_call)),
            Error::<Test, ()>::MissingRateLimit
        );
    });
}

#[test]
fn clear_rate_limit_removes_only_selected_scope() {
    new_test_ext().execute_with(|| {
        System::reset_events();

        let base_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&base_call);
        let mut map = BTreeMap::new();
        map.insert(9u16, RateLimitKind::Exact(7));
        map.insert(10u16, RateLimitKind::Exact(5));
        Limits::<Test, ()>::insert(identifier, RateLimit::Scoped(map));

        let scoped_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 9 });

        assert_ok!(RateLimiting::clear_rate_limit(
            RuntimeOrigin::root(),
            Box::new(scoped_call.clone()),
        ));

        let config = Limits::<Test, ()>::get(identifier).expect("config remains");
        assert!(config.kind_for(Some(&9u16)).is_none());
        assert_eq!(
            config.kind_for(Some(&10u16)).copied(),
            Some(RateLimitKind::Exact(5))
        );

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::RateLimitCleared {
                transaction,
                scope,
                ..
            }) => {
                assert_eq!(transaction, identifier);
                assert_eq!(scope, Some(9));
            }
            other => panic!("unexpected event: {:?}", other),
        }
    });
}

#[test]
fn set_default_rate_limit_updates_storage_and_emits_event() {
    new_test_ext().execute_with(|| {
        System::reset_events();

        assert_ok!(RateLimiting::set_default_rate_limit(
            RuntimeOrigin::root(),
            42
        ));

        assert_eq!(DefaultLimit::<Test, ()>::get(), 42);

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::DefaultRateLimitSet {
                block_span,
            }) => {
                assert_eq!(block_span, 42);
            }
            other => panic!("unexpected event: {:?}", other),
        }
    });
}

#[test]
fn set_default_rate_limit_requires_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            RateLimiting::set_default_rate_limit(RuntimeOrigin::signed(1), 5),
            BadOrigin
        );
    });
}
