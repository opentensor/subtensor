use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::{
    dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
    pallet_prelude::Weight,
    sp_runtime::{
        traits::{
            DispatchInfoOf, DispatchOriginOf, Dispatchable, Implication, TransactionExtension,
            ValidateResult,
        },
        transaction_validity::{
            InvalidTransaction, TransactionSource, TransactionValidityError, ValidTransaction,
        },
    },
};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, result::Result};

use crate::{Config, LastSeen, Limits, Pallet, types::TransactionIdentifier};

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
    type Val = Option<TransactionIdentifier>;
    type Pre = Option<TransactionIdentifier>;

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

        if Limits::<T>::get(&identifier).is_none() {
            return Ok((ValidTransaction::default(), None, origin));
        }

        let within_limit = Pallet::<T>::is_within_limit(&identifier)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

        if !within_limit {
            return Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(RATE_LIMIT_DENIED),
            ));
        }

        Ok((ValidTransaction::default(), Some(identifier), origin))
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
            if let Some(identifier) = pre {
                let block_number = frame_system::Pallet::<T>::block_number();
                LastSeen::<T>::insert(&identifier, block_number);
            }
        }
        Ok(())
    }
}
