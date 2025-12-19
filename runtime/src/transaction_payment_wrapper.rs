use crate::{NORMAL_DISPATCH_BASE_PRIORITY, OPERATIONAL_DISPATCH_PRIORITY, Weight};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_election_provider_support::private::sp_arithmetic::traits::SaturatedConversion;
use frame_support::dispatch::{DispatchClass, DispatchInfo, PostDispatchInfo};
use frame_support::pallet_prelude::TypeInfo;
use pallet_transaction_payment::{ChargeTransactionPayment, Config, Pre, Val};
use sp_runtime::DispatchResult;
use sp_runtime::traits::{
    DispatchInfoOf, DispatchOriginOf, Dispatchable, Implication, PostDispatchInfoOf,
    TransactionExtension, TransactionExtensionMetadata, ValidateResult,
};
use sp_runtime::transaction_validity::{
    TransactionPriority, TransactionSource, TransactionValidity, TransactionValidityError,
};
use sp_std::vec::Vec;
use subtensor_macros::freeze_struct;

#[freeze_struct("5f10cb9db06873c0")]
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeTransactionPaymentWrapper<T: Config> {
    charge_transaction_payment: ChargeTransactionPayment<T>,
}

impl<T: Config> core::fmt::Debug for ChargeTransactionPaymentWrapper<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "ChargeTransactionPaymentWrapper",)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
        Ok(())
    }
}

impl<T: Config> ChargeTransactionPaymentWrapper<T> {
    pub fn new(charge_transaction_payment: ChargeTransactionPayment<T>) -> Self {
        Self {
            charge_transaction_payment,
        }
    }
}

impl<T: Config> TransactionExtension<T::RuntimeCall> for ChargeTransactionPaymentWrapper<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPaymentWrapper";
    type Implicit = ();
    type Val = Val<T>;
    type Pre = Pre<T>;

    fn weight(&self, call: &T::RuntimeCall) -> Weight {
        self.charge_transaction_payment.weight(call)
    }

    fn validate(
        &self,
        origin: DispatchOriginOf<T::RuntimeCall>,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
        self_implicit: Self::Implicit,
        inherited_implication: &impl Implication,
        source: TransactionSource,
    ) -> ValidateResult<Self::Val, T::RuntimeCall> {
        let inner_validate = self.charge_transaction_payment.validate(
            origin,
            call,
            info,
            len,
            self_implicit,
            inherited_implication,
            source,
        );

        match inner_validate {
            Ok((mut valid_transaction, val, origin)) => {
                let overridden_priority = {
                    let base: TransactionPriority = match info.class {
                        DispatchClass::Normal => NORMAL_DISPATCH_BASE_PRIORITY,
                        DispatchClass::Mandatory => NORMAL_DISPATCH_BASE_PRIORITY,
                        DispatchClass::Operational => OPERATIONAL_DISPATCH_PRIORITY,
                    };
                    base.saturated_into::<TransactionPriority>()
                };

                valid_transaction.priority = overridden_priority;

                Ok((valid_transaction, val, origin))
            }
            Err(err) => Err(err),
        }
    }

    fn prepare(
        self,
        val: Self::Val,
        origin: &DispatchOriginOf<T::RuntimeCall>,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        self.charge_transaction_payment
            .prepare(val, origin, call, info, len)
    }
    fn metadata() -> Vec<TransactionExtensionMetadata> {
        ChargeTransactionPayment::<T>::metadata()
    }
    fn post_dispatch_details(
        pre: Self::Pre,
        info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<Weight, TransactionValidityError> {
        ChargeTransactionPayment::<T>::post_dispatch_details(pre, info, post_info, len, result)
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &mut PostDispatchInfoOf<T::RuntimeCall>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        ChargeTransactionPayment::<T>::post_dispatch(pre, info, post_info, len, result)
    }

    fn bare_validate(
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> TransactionValidity {
        ChargeTransactionPayment::<T>::bare_validate(call, info, len)
    }

    fn bare_validate_and_prepare(
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> Result<(), TransactionValidityError> {
        ChargeTransactionPayment::<T>::bare_validate_and_prepare(call, info, len)
    }

    fn bare_post_dispatch(
        info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &mut PostDispatchInfoOf<T::RuntimeCall>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        ChargeTransactionPayment::<T>::bare_post_dispatch(info, post_info, len, result)
    }
}
