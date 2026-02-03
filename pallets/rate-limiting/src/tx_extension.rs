use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::{
    dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
    pallet_prelude::Weight,
    sp_runtime::{
        traits::{
            DispatchInfoOf, DispatchOriginOf, Dispatchable, Implication, TransactionExtension,
            ValidateResult, Zero,
        },
        transaction_validity::{
            InvalidTransaction, TransactionSource, TransactionValidityError, ValidTransaction,
        },
    },
};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_std::{
    collections::btree_set::BTreeSet, marker::PhantomData, result::Result, vec, vec::Vec,
};

use crate::{
    Config, LastSeen, Pallet,
    types::{
        RateLimitScopeResolver, RateLimitTarget, RateLimitUsageResolver, TransactionIdentifier,
    },
};

/// Identifier returned in the transaction metadata for the rate limiting extension.
const IDENTIFIER: &str = "RateLimitTransactionExtension";

/// Custom error code used to signal a rate limit violation.
const RATE_LIMIT_DENIED: u8 = 1;

/// Transaction extension that enforces pallet rate limiting rules.
#[derive(Default, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct RateLimitTransactionExtension<T, I = ()>(PhantomData<(T, I)>)
where
    T: Config<I> + Send + Sync + TypeInfo,
    I: 'static + TypeInfo;

impl<T, I> RateLimitTransactionExtension<T, I>
where
    T: Config<I> + Send + Sync + TypeInfo,
    I: 'static + TypeInfo,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn validate_calls_same_block<'a>(
        &self,
        origin: DispatchOriginOf<<T as Config<I>>::RuntimeCall>,
        calls: impl IntoIterator<Item = &'a <T as Config<I>>::RuntimeCall>,
    ) -> ValidateResult<
        Vec<
            Option<(
                RateLimitTarget<<T as Config<I>>::GroupId>,
                Option<Vec<<T as Config<I>>::UsageKey>>,
                bool,
            )>,
        >,
        <T as Config<I>>::RuntimeCall,
    > {
        let mut usage_seen_in_block = BTreeSet::<(
            RateLimitTarget<<T as Config<I>>::GroupId>,
            Option<<T as Config<I>>::UsageKey>,
        )>::new();
        let mut vals = Vec::new();

        for call in calls {
            let val = self.validate_single_call(&origin, call, &mut usage_seen_in_block)?;
            vals.push(val);
        }

        Ok((ValidTransaction::default(), vals, origin))
    }

    fn validate_single_call(
        &self,
        origin: &DispatchOriginOf<<T as Config<I>>::RuntimeCall>,
        call: &<T as Config<I>>::RuntimeCall,
        usage_seen_in_block: &mut BTreeSet<(
            RateLimitTarget<<T as Config<I>>::GroupId>,
            Option<<T as Config<I>>::UsageKey>,
        )>,
    ) -> Result<
        Option<(
            RateLimitTarget<<T as Config<I>>::GroupId>,
            Option<Vec<<T as Config<I>>::UsageKey>>,
            bool,
        )>,
        TransactionValidityError,
    > {
        let Some(identifier) = TransactionIdentifier::from_call(call) else {
            return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
        };

        if !Pallet::<T, I>::is_registered(&identifier) {
            return Ok(None);
        }

        let scopes = <T as Config<I>>::LimitScopeResolver::context(origin, call);
        let usage = <T as Config<I>>::UsageResolver::context(origin, call);

        let config_target = Pallet::<T, I>::config_target(&identifier)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;
        let usage_target = Pallet::<T, I>::usage_target(&identifier)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;
        let bypass = <T as Config<I>>::LimitScopeResolver::should_bypass(origin, call);
        let should_record =
            bypass.record_usage && Pallet::<T, I>::should_record_usage(&identifier, &usage_target);

        if bypass.bypass_enforcement {
            return Ok(should_record.then_some((usage_target, usage, true)));
        }

        let usage_keys: Vec<Option<<T as Config<I>>::UsageKey>> = match usage.clone() {
            None => vec![None],
            Some(keys) => keys.into_iter().map(Some).collect(),
        };

        let mut unique_usage_keys = BTreeSet::new();
        for key in usage_keys.iter().cloned() {
            unique_usage_keys.insert(key);
        }

        let mut last_seen_per_key: Vec<(
            Option<<T as Config<I>>::UsageKey>,
            Option<BlockNumberFor<T>>,
        )> = Vec::with_capacity(unique_usage_keys.len());
        let fallback_block = frame_system::Pallet::<T>::block_number();

        for key in unique_usage_keys {
            let last_seen = Self::resolve_last_seen_for_key(
                usage_target,
                key.clone(),
                should_record,
                fallback_block,
                usage_seen_in_block,
            );

            last_seen_per_key.push((key, last_seen));
        }

        let scope_list: Vec<Option<<T as Config<I>>::LimitScope>> = match scopes {
            None => vec![None],
            Some(resolved) if resolved.is_empty() => vec![None],
            Some(resolved) => resolved.into_iter().map(Some).collect(),
        };

        let mut enforced = false;
        for scope in scope_list {
            let Some(block_span) =
                Pallet::<T, I>::effective_span(origin, call, &config_target, &scope)
            else {
                continue;
            };
            if block_span.is_zero() {
                continue;
            }
            enforced = true;
            let within_limit = last_seen_per_key.iter().all(|(key, last_seen)| {
                Pallet::<T, I>::within_span(&usage_target, key, block_span, last_seen.clone())
            });
            if !within_limit {
                return Err(TransactionValidityError::Invalid(
                    InvalidTransaction::Custom(RATE_LIMIT_DENIED),
                ));
            }
        }

        if !enforced {
            return Ok(None);
        }

        Ok(Some((usage_target, usage, should_record)))
    }

    fn resolve_last_seen_for_key(
        usage_target: RateLimitTarget<<T as Config<I>>::GroupId>,
        key: Option<<T as Config<I>>::UsageKey>,
        should_record: bool,
        fallback_block: BlockNumberFor<T>,
        usage_seen_in_block: &mut BTreeSet<(
            RateLimitTarget<<T as Config<I>>::GroupId>,
            Option<<T as Config<I>>::UsageKey>,
        )>,
    ) -> Option<BlockNumberFor<T>> {
        if should_record {
            let entry = (usage_target, key.clone());
            if !usage_seen_in_block.insert(entry) {
                Some(fallback_block)
            } else {
                LastSeen::<T, I>::get(&usage_target, &key)
            }
        } else {
            LastSeen::<T, I>::get(&usage_target, &key)
        }
    }
}

impl<T, I> Clone for RateLimitTransactionExtension<T, I>
where
    T: Config<I> + Send + Sync + TypeInfo,
    I: 'static + TypeInfo,
{
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T, I> PartialEq for RateLimitTransactionExtension<T, I>
where
    T: Config<I> + Send + Sync + TypeInfo,
    I: 'static + TypeInfo,
{
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T, I> Eq for RateLimitTransactionExtension<T, I>
where
    T: Config<I> + Send + Sync + TypeInfo,
    I: 'static + TypeInfo,
{
}

impl<T, I> core::fmt::Debug for RateLimitTransactionExtension<T, I>
where
    T: Config<I> + Send + Sync + TypeInfo,
    I: 'static + TypeInfo,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(IDENTIFIER)
    }
}

impl<T, I> TransactionExtension<<T as Config<I>>::RuntimeCall>
    for RateLimitTransactionExtension<T, I>
where
    T: Config<I> + Send + Sync + TypeInfo,
    I: 'static + TypeInfo + Send + Sync,
    <T as Config<I>>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
    const IDENTIFIER: &'static str = IDENTIFIER;

    type Implicit = ();
    type Val = Option<(
        RateLimitTarget<<T as Config<I>>::GroupId>,
        Option<Vec<<T as Config<I>>::UsageKey>>,
        bool,
    )>;
    type Pre = Option<(
        RateLimitTarget<<T as Config<I>>::GroupId>,
        Option<Vec<<T as Config<I>>::UsageKey>>,
        bool,
    )>;

    fn weight(&self, _call: &<T as Config<I>>::RuntimeCall) -> Weight {
        Weight::zero()
    }

    fn validate(
        &self,
        origin: DispatchOriginOf<<T as Config<I>>::RuntimeCall>,
        call: &<T as Config<I>>::RuntimeCall,
        _info: &DispatchInfoOf<<T as Config<I>>::RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, <T as Config<I>>::RuntimeCall> {
        let (valid, vals, origin) =
            self.validate_calls_same_block(origin, sp_std::iter::once(call))?;
        Ok((valid, vals.into_iter().next().unwrap_or(None), origin))
    }

    fn prepare(
        self,
        val: Self::Val,
        _origin: &DispatchOriginOf<<T as Config<I>>::RuntimeCall>,
        _call: &<T as Config<I>>::RuntimeCall,
        _info: &DispatchInfoOf<<T as Config<I>>::RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(val)
    }

    fn post_dispatch(
        pre: Self::Pre,
        _info: &DispatchInfoOf<<T as Config<I>>::RuntimeCall>,
        _post_info: &mut PostDispatchInfo,
        _len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        if result.is_ok() {
            if let Some((target, usage, should_record)) = pre {
                if !should_record {
                    return Ok(());
                }
                let block_number = frame_system::Pallet::<T>::block_number();
                match usage {
                    None => LastSeen::<T, I>::insert(
                        target,
                        None::<<T as Config<I>>::UsageKey>,
                        block_number,
                    ),
                    Some(keys) => {
                        for key in keys {
                            LastSeen::<T, I>::insert(target, Some(key), block_number);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use codec::Encode;
    use frame_support::{
        assert_ok,
        dispatch::{GetDispatchInfo, PostDispatchInfo},
    };
    use sp_runtime::{
        traits::{TransactionExtension, TxBaseImplication},
        transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidityError},
    };

    use crate::{
        GroupSharing, LastSeen, Limits,
        types::{RateLimit, RateLimitKind},
    };

    use super::*;
    use crate::mock::*;
    use sp_std::collections::btree_map::BTreeMap;

    fn remark_call() -> RuntimeCall {
        RuntimeCall::System(frame_system::Call::<Test>::remark { remark: Vec::new() })
    }

    fn bypass_call() -> RuntimeCall {
        RuntimeCall::RateLimiting(RateLimitingCall::remove_call_from_group {
            transaction: TransactionIdentifier::new(0, 0),
        })
    }

    fn adjustable_call() -> RuntimeCall {
        RuntimeCall::RateLimiting(RateLimitingCall::deregister_call {
            transaction: TransactionIdentifier::new(0, 0),
            scope: None,
            clear_usage: false,
        })
    }

    fn multi_scope_call(block_span: u64) -> RuntimeCall {
        RuntimeCall::RateLimiting(RateLimitingCall::set_rate_limit {
            target: RateLimitTarget::Transaction(TransactionIdentifier::new(0, 0)),
            scope: None,
            limit: RateLimitKind::Exact(block_span),
        })
    }

    fn new_tx_extension() -> RateLimitTransactionExtension<Test> {
        RateLimitTransactionExtension(Default::default())
    }

    fn target_for_call(call: &RuntimeCall) -> RateLimitTarget<GroupId> {
        RateLimitTarget::Transaction(identifier_for(call))
    }

    fn validate_with_tx_extension(
        extension: &RateLimitTransactionExtension<Test>,
        call: &RuntimeCall,
    ) -> Result<
        (
            sp_runtime::transaction_validity::ValidTransaction,
            Option<(RateLimitTarget<GroupId>, Option<Vec<UsageKey>>, bool)>,
            RuntimeOrigin,
        ),
        TransactionValidityError,
    > {
        let info = call.get_dispatch_info();
        let len = call.encode().len();
        extension.validate(
            RuntimeOrigin::signed(42),
            call,
            &info,
            len,
            (),
            &TxBaseImplication(()),
            TransactionSource::External,
        )
    }

    fn validate_same_block_calls(
        extension: &RateLimitTransactionExtension<Test>,
        calls: &[RuntimeCall],
    ) -> Result<
        (
            sp_runtime::transaction_validity::ValidTransaction,
            Vec<Option<(RateLimitTarget<GroupId>, Option<Vec<UsageKey>>, bool)>>,
            RuntimeOrigin,
        ),
        TransactionValidityError,
    > {
        extension.validate_calls_same_block(RuntimeOrigin::signed(42), calls.iter())
    }

    #[test]
    fn tx_extension_allows_calls_without_limit() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = remark_call();

            let (_valid, val, _origin) =
                validate_with_tx_extension(&extension, &call).expect("valid");
            assert!(val.is_none());

            let info = call.get_dispatch_info();
            let len = call.encode().len();
            let origin_for_prepare = RuntimeOrigin::signed(42);
            let pre = extension
                .clone()
                .prepare(val.clone(), &origin_for_prepare, &call, &info, len)
                .expect("prepare succeeds");

            let mut post = PostDispatchInfo::default();
            RateLimitTransactionExtension::<Test>::post_dispatch(
                pre,
                &info,
                &mut post,
                len,
                &Ok(()),
            )
            .expect("post_dispatch succeeds");

            let target = target_for_call(&call);
            assert_eq!(LastSeen::<Test, ()>::get(target, None::<UsageKey>), None);
        });
    }

    #[test]
    fn tx_extension_honors_bypass_signal() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = bypass_call();

            let (valid, val, _) =
                validate_with_tx_extension(&extension, &call).expect("bypass should succeed");
            assert_eq!(valid.priority, 0);
            assert!(val.is_none());

            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);
            Limits::<Test, ()>::insert(target, RateLimit::global(RateLimitKind::Exact(3)));
            LastSeen::<Test, ()>::insert(target, None::<UsageKey>, 1);

            let (_valid, post_val, _) =
                validate_with_tx_extension(&extension, &call).expect("still bypassed");
            assert!(post_val.is_none());
        });
    }

    #[test]
    fn tx_extension_applies_adjusted_span() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = adjustable_call();
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);
            Limits::<Test, ()>::insert(target, RateLimit::global(RateLimitKind::Exact(4)));
            LastSeen::<Test, ()>::insert(target, Some(1u16), 10);

            System::set_block_number(14);

            // Stored span (4) would allow the call, but adjusted span (8) should block it.
            let err = validate_with_tx_extension(&extension, &call)
                .expect_err("adjusted span should apply");
            match err {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code)) => {
                    assert_eq!(code, RATE_LIMIT_DENIED);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }

    #[test]
    fn tx_extension_rejects_when_any_scope_fails() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = multi_scope_call(43);
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);

            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call.clone()),
                None,
            ));

            let mut scopes = BTreeMap::new();
            scopes.insert(43u16, RateLimitKind::Exact(5));
            scopes.insert(44u16, RateLimitKind::Exact(3));
            Limits::<Test, ()>::insert(target, RateLimit::Scoped(scopes));
            LastSeen::<Test, ()>::insert(target, Some(43u16), 10);

            System::set_block_number(14);

            let err =
                validate_with_tx_extension(&extension, &call).expect_err("one scope should block");
            match err {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code)) => {
                    assert_eq!(code, RATE_LIMIT_DENIED);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }

    #[test]
    fn tx_extension_rejects_when_any_usage_key_fails() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = multi_scope_call(42);
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);

            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call.clone()),
                None,
            ));

            let mut scopes = BTreeMap::new();
            scopes.insert(42u16, RateLimitKind::Exact(5));
            scopes.insert(43u16, RateLimitKind::Exact(5));
            Limits::<Test, ()>::insert(target, RateLimit::Scoped(scopes));
            LastSeen::<Test, ()>::insert(target, Some(42u16), 8);
            LastSeen::<Test, ()>::insert(target, Some(43u16), 12);

            System::set_block_number(14);

            let err = validate_with_tx_extension(&extension, &call)
                .expect_err("one usage key should block");
            match err {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code)) => {
                    assert_eq!(code, RATE_LIMIT_DENIED);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }

    #[test]
    fn tx_extension_records_usage_on_bypass() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit {
                block_span: 2,
            });
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);

            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call.clone()),
                None,
            ));

            System::set_block_number(5);

            let (_valid, val, origin) =
                validate_with_tx_extension(&extension, &call).expect("bypass should succeed");
            assert!(val.is_some(), "bypass decision should still record usage");

            let info = call.get_dispatch_info();
            let len = call.encode().len();
            let pre = extension
                .clone()
                .prepare(val.clone(), &origin, &call, &info, len)
                .expect("prepare succeeds");

            let mut post = PostDispatchInfo::default();
            RateLimitTransactionExtension::<Test>::post_dispatch(
                pre,
                &info,
                &mut post,
                len,
                &Ok(()),
            )
            .expect("post_dispatch succeeds");

            assert_eq!(
                LastSeen::<Test, ()>::get(target, Some(2u16)),
                Some(5u64.into())
            );
        });
    }

    #[test]
    fn tx_extension_records_last_seen_for_successful_call() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = remark_call();
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);
            Limits::<Test, ()>::insert(target, RateLimit::global(RateLimitKind::Exact(5)));

            System::set_block_number(10);

            let (_valid, val, _) = validate_with_tx_extension(&extension, &call).expect("valid");
            assert!(val.is_some());

            let info = call.get_dispatch_info();
            let len = call.encode().len();
            let origin_for_prepare = RuntimeOrigin::signed(42);
            let pre = extension
                .clone()
                .prepare(val.clone(), &origin_for_prepare, &call, &info, len)
                .expect("prepare succeeds");

            let mut post = PostDispatchInfo::default();
            RateLimitTransactionExtension::<Test>::post_dispatch(
                pre,
                &info,
                &mut post,
                len,
                &Ok(()),
            )
            .expect("post_dispatch succeeds");

            assert_eq!(
                LastSeen::<Test, ()>::get(target, None::<UsageKey>),
                Some(10)
            );
        });
    }

    #[test]
    fn tx_extension_rejects_when_call_occurs_too_soon() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = remark_call();
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);
            Limits::<Test, ()>::insert(target, RateLimit::global(RateLimitKind::Exact(5)));
            LastSeen::<Test, ()>::insert(target, None::<UsageKey>, 20);

            System::set_block_number(22);

            let err =
                validate_with_tx_extension(&extension, &call).expect_err("should be rate limited");
            match err {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code)) => {
                    assert_eq!(code, 1);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }

    #[test]
    fn tx_extension_same_block_rejects_duplicate_usage() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = remark_call();
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);

            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call.clone()),
                None,
            ));
            Limits::<Test, ()>::insert(target, RateLimit::global(RateLimitKind::Exact(5)));

            System::set_block_number(10);

            let err = validate_same_block_calls(&extension, &[call.clone(), call.clone()])
                .expect_err("duplicate should block");
            match err {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code)) => {
                    assert_eq!(code, RATE_LIMIT_DENIED);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }

    #[test]
    fn tx_extension_same_block_allows_distinct_usage_keys() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call_a = multi_scope_call(5);
            let call_b = multi_scope_call(6);
            let identifier = identifier_for(&call_a);
            let target = RateLimitTarget::Transaction(identifier);

            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call_a.clone()),
                None,
            ));

            let mut scopes = BTreeMap::new();
            scopes.insert(5u16, RateLimitKind::Exact(5));
            scopes.insert(6u16, RateLimitKind::Exact(5));
            Limits::<Test, ()>::insert(target, RateLimit::Scoped(scopes));

            System::set_block_number(10);

            let (_valid, vals, _) =
                validate_same_block_calls(&extension, &[call_a, call_b]).expect("valid");
            assert_eq!(vals.len(), 2);
        });
    }

    #[test]
    fn tx_extension_skips_last_seen_when_span_zero() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = remark_call();
            let identifier = identifier_for(&call);
            let target = RateLimitTarget::Transaction(identifier);
            Limits::<Test, ()>::insert(target, RateLimit::global(RateLimitKind::Exact(0)));

            System::set_block_number(30);

            let (_valid, val, _) = validate_with_tx_extension(&extension, &call).expect("valid");
            assert!(val.is_none());

            let info = call.get_dispatch_info();
            let len = call.encode().len();
            let origin_for_prepare = RuntimeOrigin::signed(42);
            let pre = extension
                .clone()
                .prepare(val.clone(), &origin_for_prepare, &call, &info, len)
                .expect("prepare succeeds");

            let mut post = PostDispatchInfo::default();
            RateLimitTransactionExtension::<Test>::post_dispatch(
                pre,
                &info,
                &mut post,
                len,
                &Ok(()),
            )
            .expect("post_dispatch succeeds");

            assert_eq!(LastSeen::<Test, ()>::get(target, None::<UsageKey>), None);
        });
    }

    #[test]
    fn tx_extension_skips_write_for_read_only_group_member() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            assert_ok!(RateLimiting::create_group(
                RuntimeOrigin::root(),
                b"use-ro".to_vec(),
                GroupSharing::UsageOnly,
            ));
            let group = RateLimiting::next_group_id().saturating_sub(1);

            let call = remark_call();
            let identifier = identifier_for(&call);
            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call.clone()),
                Some(group),
            ));
            assert_ok!(RateLimiting::set_call_read_only(
                RuntimeOrigin::root(),
                identifier,
                true
            ));

            let tx_target = RateLimitTarget::Transaction(identifier);
            let usage_target = RateLimitTarget::Group(group);
            Limits::<Test, ()>::insert(tx_target, RateLimit::global(RateLimitKind::Exact(2)));
            LastSeen::<Test, ()>::insert(usage_target, Some(1u16), 2);

            System::set_block_number(5);

            let (_valid, val, _) = validate_with_tx_extension(&extension, &call).expect("valid");
            let info = call.get_dispatch_info();
            let len = call.encode().len();
            let origin_for_prepare = RuntimeOrigin::signed(42);
            let pre = extension
                .clone()
                .prepare(val.clone(), &origin_for_prepare, &call, &info, len)
                .expect("prepare succeeds");

            let mut post = PostDispatchInfo::default();
            RateLimitTransactionExtension::<Test>::post_dispatch(
                pre,
                &info,
                &mut post,
                len,
                &Ok(()),
            )
            .expect("post_dispatch succeeds");

            // Usage key should remain untouched because the call is read-only.
            assert_eq!(LastSeen::<Test, ()>::get(usage_target, Some(1u16)), Some(2));
        });
    }

    #[test]
    fn tx_extension_respects_usage_group_sharing() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            assert_ok!(RateLimiting::create_group(
                RuntimeOrigin::root(),
                b"use".to_vec(),
                GroupSharing::UsageOnly,
            ));
            let group = RateLimiting::next_group_id().saturating_sub(1);

            let call = remark_call();
            let identifier = identifier_for(&call);
            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call.clone()),
                Some(group),
            ));

            let tx_target = RateLimitTarget::Transaction(identifier);
            let usage_target = RateLimitTarget::Group(group);
            Limits::<Test, ()>::insert(tx_target, RateLimit::global(RateLimitKind::Exact(5)));
            LastSeen::<Test, ()>::insert(usage_target, None::<UsageKey>, 10);
            System::set_block_number(12);

            let err = validate_with_tx_extension(&extension, &call)
                .expect_err("usage grouping should rate limit");
            match err {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code)) => {
                    assert_eq!(code, RATE_LIMIT_DENIED);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }

    #[test]
    fn tx_extension_respects_config_group_sharing() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            assert_ok!(RateLimiting::create_group(
                RuntimeOrigin::root(),
                b"cfg".to_vec(),
                GroupSharing::ConfigOnly,
            ));
            let group = RateLimiting::next_group_id().saturating_sub(1);

            let call = remark_call();
            let identifier = identifier_for(&call);
            assert_ok!(RateLimiting::register_call(
                RuntimeOrigin::root(),
                Box::new(call.clone()),
                Some(group),
            ));

            let tx_target = RateLimitTarget::Transaction(identifier);
            let group_target = RateLimitTarget::Group(group);
            Limits::<Test, ()>::remove(tx_target);
            Limits::<Test, ()>::insert(group_target, RateLimit::global(RateLimitKind::Exact(5)));
            LastSeen::<Test, ()>::insert(tx_target, None::<UsageKey>, 10);
            System::set_block_number(12);

            let err = validate_with_tx_extension(&extension, &call)
                .expect_err("config grouping should rate limit");
            match err {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code)) => {
                    assert_eq!(code, RATE_LIMIT_DENIED);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }
}
