use frame_support::{assert_noop, assert_ok, error::BadOrigin};

use crate::{DefaultLimit, LastSeen, Limits, RateLimit, mock::*, pallet::Error};

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
        Limits::<Test, ()>::insert(identifier, None::<LimitContext>, RateLimit::Exact(7));

        let fetched =
            RateLimiting::limit_for_call_names("RateLimiting", "set_default_rate_limit", None)
                .expect("limit should exist");
        assert_eq!(fetched, RateLimit::Exact(7));
    });
}

#[test]
fn limit_for_call_names_prefers_context_specific_limit() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(identifier, None::<LimitContext>, RateLimit::Exact(3));
        Limits::<Test, ()>::insert(identifier, Some(5), RateLimit::Exact(8));

        let fetched =
            RateLimiting::limit_for_call_names("RateLimiting", "set_default_rate_limit", Some(5))
                .expect("limit should exist");
        assert_eq!(fetched, RateLimit::Exact(8));

        let fallback =
            RateLimiting::limit_for_call_names("RateLimiting", "set_default_rate_limit", Some(1))
                .expect("limit should exist");
        assert_eq!(fallback, RateLimit::Exact(3));
    });
}

#[test]
fn resolved_limit_for_call_names_resolves_default_value() {
    new_test_ext().execute_with(|| {
        DefaultLimit::<Test, ()>::put(3);
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(identifier, None::<LimitContext>, RateLimit::Default);

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
fn resolved_limit_for_call_names_prefers_context_specific_value() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(identifier, None::<LimitContext>, RateLimit::Exact(4));
        Limits::<Test, ()>::insert(identifier, Some(6), RateLimit::Exact(9));

        let resolved = RateLimiting::resolved_limit_for_call_names(
            "RateLimiting",
            "set_default_rate_limit",
            Some(6),
        )
        .expect("resolved limit");
        assert_eq!(resolved, 9);

        let fallback = RateLimiting::resolved_limit_for_call_names(
            "RateLimiting",
            "set_default_rate_limit",
            Some(1),
        )
        .expect("resolved limit");
        assert_eq!(fallback, 4);
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

        let result = RateLimiting::is_within_limit(&identifier, &None);
        assert_eq!(result.expect("no error expected"), true);
    });
}

#[test]
fn is_within_limit_false_when_rate_limited() {
    new_test_ext().execute_with(|| {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&call);
        Limits::<Test, ()>::insert(identifier, Some(1 as LimitContext), RateLimit::Exact(5));
        LastSeen::<Test, ()>::insert(identifier, Some(1 as LimitContext), 9);

        System::set_block_number(13);

        let within = RateLimiting::is_within_limit(&identifier, &Some(1 as LimitContext))
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
        Limits::<Test, ()>::insert(identifier, Some(2 as LimitContext), RateLimit::Exact(5));
        LastSeen::<Test, ()>::insert(identifier, Some(2 as LimitContext), 10);

        System::set_block_number(20);

        let within = RateLimiting::is_within_limit(&identifier, &Some(2 as LimitContext))
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
        let limit = RateLimit::Exact(9);

        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
            limit,
            None,
        ));

        let identifier = identifier_for(&target_call);
        assert_eq!(
            Limits::<Test, ()>::get(identifier, None::<LimitContext>),
            Some(limit)
        );

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::RateLimitSet {
                transaction,
                context,
                limit: emitted_limit,
                pallet,
                extrinsic,
            }) => {
                assert_eq!(transaction, identifier);
                assert_eq!(context, None);
                assert_eq!(emitted_limit, limit);
                assert_eq!(pallet, b"RateLimiting".to_vec());
                assert_eq!(extrinsic, b"set_default_rate_limit".to_vec());
            }
            other => panic!("unexpected event: {:?}", other),
        }
    });
}

#[test]
fn set_rate_limit_supports_context_specific_limit() {
    new_test_ext().execute_with(|| {
        let target_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let context = Some(7u16);
        assert_ok!(RateLimiting::set_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
            RateLimit::Exact(11),
            context,
        ));

        let identifier = identifier_for(&target_call);
        assert_eq!(
            Limits::<Test, ()>::get(identifier, Some(7)),
            Some(RateLimit::Exact(11))
        );
        // global remains untouched
        assert_eq!(
            Limits::<Test, ()>::get(identifier, None::<LimitContext>),
            None
        );
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
                RateLimit::Exact(1),
                None,
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
            RateLimit::Default,
            None,
        ));

        let identifier = identifier_for(&target_call);
        assert_eq!(
            Limits::<Test, ()>::get(identifier, None::<LimitContext>),
            Some(RateLimit::Default)
        );
    });
}

#[test]
fn clear_rate_limit_removes_entry_and_emits_event() {
    new_test_ext().execute_with(|| {
        System::reset_events();

        let target_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&target_call);
        Limits::<Test, ()>::insert(identifier, None::<LimitContext>, RateLimit::Exact(4));

        assert_ok!(RateLimiting::clear_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
            None,
        ));

        assert_eq!(
            Limits::<Test, ()>::get(identifier, None::<LimitContext>),
            None
        );

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::RateLimitCleared {
                transaction,
                context,
                pallet,
                extrinsic,
            }) => {
                assert_eq!(transaction, identifier);
                assert_eq!(context, None);
                assert_eq!(pallet, b"RateLimiting".to_vec());
                assert_eq!(extrinsic, b"set_default_rate_limit".to_vec());
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
            RateLimiting::clear_rate_limit(RuntimeOrigin::root(), Box::new(target_call), None),
            Error::<Test, ()>::MissingRateLimit
        );
    });
}

#[test]
fn clear_rate_limit_removes_only_selected_context() {
    new_test_ext().execute_with(|| {
        System::reset_events();

        let target_call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = identifier_for(&target_call);
        Limits::<Test, ()>::insert(identifier, None::<LimitContext>, RateLimit::Exact(5));
        Limits::<Test, ()>::insert(identifier, Some(9), RateLimit::Exact(7));

        assert_ok!(RateLimiting::clear_rate_limit(
            RuntimeOrigin::root(),
            Box::new(target_call.clone()),
            Some(9),
        ));

        assert_eq!(Limits::<Test, ()>::get(identifier, Some(9u16)), None);
        assert_eq!(
            Limits::<Test, ()>::get(identifier, None::<LimitContext>),
            Some(RateLimit::Exact(5))
        );

        match pop_last_event() {
            RuntimeEvent::RateLimiting(crate::pallet::Event::RateLimitCleared {
                transaction,
                context,
                ..
            }) => {
                assert_eq!(transaction, identifier);
                assert_eq!(context, Some(9));
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
