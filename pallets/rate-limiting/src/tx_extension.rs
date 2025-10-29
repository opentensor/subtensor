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
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, result::Result};

use crate::{
    Config, LastSeen, Pallet,
    types::{RateLimitContextResolver, TransactionIdentifier},
};

/// Identifier returned in the transaction metadata for the rate limiting extension.
const IDENTIFIER: &str = "RateLimitTransactionExtension";

/// Custom error code used to signal a rate limit violation.
const RATE_LIMIT_DENIED: u8 = 1;

/// Transaction extension that enforces pallet rate limiting rules.
#[derive(Default, Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
pub struct RateLimitTransactionExtension<T: Config + Send + Sync + TypeInfo>(PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> core::fmt::Debug for RateLimitTransactionExtension<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(IDENTIFIER)
    }
}

impl<T> TransactionExtension<<T as Config>::RuntimeCall> for RateLimitTransactionExtension<T>
where
    T: Config + Send + Sync + TypeInfo,
    <T as Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
    const IDENTIFIER: &'static str = IDENTIFIER;

    type Implicit = ();
    type Val = Option<(TransactionIdentifier, Option<T::LimitContext>)>;
    type Pre = Option<(TransactionIdentifier, Option<T::LimitContext>)>;

    fn weight(&self, _call: &<T as Config>::RuntimeCall) -> Weight {
        Weight::zero()
    }

    fn validate(
        &self,
        origin: DispatchOriginOf<<T as Config>::RuntimeCall>,
        call: &<T as Config>::RuntimeCall,
        _info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, <T as Config>::RuntimeCall> {
        let identifier = match TransactionIdentifier::from_call::<T>(call) {
            Ok(identifier) => identifier,
            Err(_) => return Err(TransactionValidityError::Invalid(InvalidTransaction::Call)),
        };

        let context = <T as Config>::ContextResolver::context(call);

        let Some(block_span) = Pallet::<T>::resolved_limit(&identifier, &context) else {
            return Ok((ValidTransaction::default(), None, origin));
        };

        if block_span.is_zero() {
            return Ok((ValidTransaction::default(), None, origin));
        }

        let within_limit = Pallet::<T>::is_within_limit(&identifier, &context)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

        if !within_limit {
            return Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(RATE_LIMIT_DENIED),
            ));
        }

        Ok((
            ValidTransaction::default(),
            Some((identifier, context)),
            origin,
        ))
    }

    fn prepare(
        self,
        val: Self::Val,
        _origin: &DispatchOriginOf<<T as Config>::RuntimeCall>,
        _call: &<T as Config>::RuntimeCall,
        _info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(val)
    }

    fn post_dispatch(
        pre: Self::Pre,
        _info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _post_info: &mut PostDispatchInfo,
        _len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        if result.is_ok() {
            if let Some((identifier, context)) = pre {
                let block_number = frame_system::Pallet::<T>::block_number();
                LastSeen::<T>::insert(&identifier, context, block_number);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use codec::Encode;
    use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
    use sp_runtime::{
        traits::{TransactionExtension, TxBaseImplication},
        transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidityError},
    };

    use crate::{LastSeen, Limits, RateLimit, types::TransactionIdentifier};

    use super::*;
    use crate::mock::*;

    fn remark_call() -> RuntimeCall {
        RuntimeCall::System(frame_system::Call::<Test>::remark { remark: Vec::new() })
    }

    fn new_tx_extension() -> RateLimitTransactionExtension<Test> {
        RateLimitTransactionExtension(Default::default())
    }

    fn validate_with_tx_extension(
        extension: &RateLimitTransactionExtension<Test>,
        call: &RuntimeCall,
    ) -> Result<
        (
            sp_runtime::transaction_validity::ValidTransaction,
            Option<(TransactionIdentifier, Option<LimitContext>)>,
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

            let identifier = identifier_for(&call);
            assert_eq!(
                LastSeen::<Test>::get(identifier, None::<LimitContext>),
                None
            );
        });
    }

    #[test]
    fn tx_extension_records_last_seen_for_successful_call() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = remark_call();
            let identifier = identifier_for(&call);
            Limits::<Test>::insert(identifier, None::<LimitContext>, RateLimit::Exact(5));

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
                LastSeen::<Test>::get(identifier, None::<LimitContext>),
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
            Limits::<Test>::insert(identifier, None::<LimitContext>, RateLimit::Exact(5));
            LastSeen::<Test>::insert(identifier, None::<LimitContext>, 20);

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
    fn tx_extension_skips_last_seen_when_span_zero() {
        new_test_ext().execute_with(|| {
            let extension = new_tx_extension();
            let call = remark_call();
            let identifier = identifier_for(&call);
            Limits::<Test>::insert(identifier, None::<LimitContext>, RateLimit::Exact(0));

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

            assert_eq!(
                LastSeen::<Test>::get(identifier, None::<LimitContext>),
                None
            );
        });
    }
}
