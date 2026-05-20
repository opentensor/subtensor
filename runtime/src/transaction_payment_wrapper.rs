use crate::{NORMAL_DISPATCH_BASE_PRIORITY, OPERATIONAL_DISPATCH_PRIORITY, Weight};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_election_provider_support::private::sp_arithmetic::traits::SaturatedConversion;
use frame_support::dispatch::{DispatchClass, DispatchInfo, PostDispatchInfo};
use frame_support::pallet_prelude::TypeInfo;
use frame_support::traits::{Get, IsSubType, IsType};
use pallet_subtensor_proxy as pallet_proxy;
use pallet_subtensor_utility as pallet_utility;
use pallet_transaction_payment::OnChargeTransaction;
use pallet_transaction_payment::{ChargeTransactionPayment, Config, Pre, Val};
use sp_runtime::DispatchResult;
use sp_runtime::traits::{
    AsSystemOriginSigner, DispatchInfoOf, DispatchOriginOf, Dispatchable, Implication,
    PostDispatchInfoOf, StaticLookup, TransactionExtension, TransactionExtensionMetadata,
    ValidateResult,
};
use sp_runtime::transaction_validity::{
    TransactionPriority, TransactionSource, TransactionValidity, TransactionValidityError,
};
use sp_std::boxed::Box;
use sp_std::vec::Vec;
use subtensor_macros::freeze_struct;

type BalanceOf<T> = <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance;

type RuntimeCallOf<T> = <T as frame_system::Config>::RuntimeCall;
type RuntimeOriginOf<T> = <T as frame_system::Config>::RuntimeOrigin;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type LookupOf<T> = <T as frame_system::Config>::Lookup;

const MAX_REAL_PAYS_FEE_PROXY_DEPTH: u8 = 3;

#[freeze_struct("f003cde1f9da4a90")]
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeTransactionPaymentWrapper<T: Config> {
    inner: ChargeTransactionPayment<T>,
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

impl<T: Config> ChargeTransactionPaymentWrapper<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    BalanceOf<T>: Send + Sync,
{
    pub fn new(fee: BalanceOf<T>) -> Self {
        let inner = ChargeTransactionPayment::<T>::from(fee);
        Self { inner }
    }
}

impl<T: Config + pallet_proxy::Config + pallet_utility::Config> ChargeTransactionPaymentWrapper<T>
where
    RuntimeCallOf<T>: IsSubType<pallet_proxy::Call<T>> + IsSubType<pallet_utility::Call<T>>,
    RuntimeOriginOf<T>: AsSystemOriginSigner<AccountIdOf<T>> + Clone,
{
    /// Extract (real, delegate, inner_call) from a `proxy` call.
    /// `signer` is used as the delegate since it is implicit (the caller).
    /// `proxy_announced` is intentionally not handled here; fee propagation
    /// only applies to `proxy` calls to keep the logic simple.
    fn extract_proxy_parts<'a>(
        call: &'a RuntimeCallOf<T>,
        signer: &AccountIdOf<T>,
    ) -> Option<(
        AccountIdOf<T>,
        AccountIdOf<T>,
        &'a Box<<T as pallet_proxy::Config>::RuntimeCall>,
    )> {
        match call.is_sub_type()? {
            pallet_proxy::Call::proxy { real, call, .. } => {
                let real = LookupOf::<T>::lookup(real.clone()).ok()?;
                Some((real, signer.clone(), call))
            }
            _ => None,
        }
    }

    /// Determine who should pay the transaction fee for a proxy call.
    ///
    /// Follows the RealPaysFee chain up to three proxy levels deep:
    /// - Case 1: `proxy(real=A, call)` → A pays if `RealPaysFee<A, signer>`
    /// - Case 2: `proxy(real=B, proxy(real=A, call))` → A pays if both
    ///   `RealPaysFee<B, signer>` and `RealPaysFee<A, B>` are set; B pays if only the former.
    /// - Case 3: `proxy(real=B, batch([proxy(real=A, ..), ..]))` → A pays if
    ///   `RealPaysFee<B, signer>`, all batch items are proxy calls with the same real A,
    ///   and `RealPaysFee<A, B>` is set; B pays if only the first condition holds.
    /// - Case 4: `proxy(real=C, proxy(real=B, batch([proxy(real=A, ..), ..])))`
    ///   → A pays if all three `RealPaysFee` relationships are set and the batch is homogeneous.
    ///
    /// Returns `None` if the signer should pay (no RealPaysFee opt-in).
    fn extract_real_fee_payer(
        call: &RuntimeCallOf<T>,
        origin: &RuntimeOriginOf<T>,
    ) -> Option<AccountIdOf<T>> {
        let signer = origin.as_system_origin_signer()?;
        Self::resolve_real_fee_payer(call, signer, MAX_REAL_PAYS_FEE_PROXY_DEPTH)
    }

    fn resolve_real_fee_payer(
        call: &RuntimeCallOf<T>,
        delegate: &AccountIdOf<T>,
        remaining_proxy_depth: u8,
    ) -> Option<AccountIdOf<T>> {
        let Some((real, _, inner_call)) = Self::extract_proxy_parts(call, delegate) else {
            return None;
        };

        if !pallet_proxy::Pallet::<T>::is_real_pays_fee(&real, delegate) {
            return None;
        }

        if remaining_proxy_depth <= 1 {
            return Some(real);
        }

        let inner_call: &RuntimeCallOf<T> = (*inner_call).as_ref().into_ref();

        if let Some(payer) =
            Self::resolve_real_fee_payer(inner_call, &real, remaining_proxy_depth - 1)
        {
            return Some(payer);
        }

        if let Some(payer) = Self::extract_batch_proxy_payer(inner_call, &real) {
            return Some(payer);
        }

        Some(real)
    }

    /// Check if an inner call is a batch where ALL items are proxy calls with the same real
    /// account, and that real account has opted in to pay.
    /// `outer_real` is used as the implicit delegate for `proxy` calls within the batch.
    fn extract_batch_proxy_payer(
        inner_call: &RuntimeCallOf<T>,
        outer_real: &AccountIdOf<T>,
    ) -> Option<AccountIdOf<T>> {
        let Some(utility_call) = inner_call.is_sub_type() else {
            return None;
        };
        let calls: &Vec<<T as pallet_utility::Config>::RuntimeCall> = match utility_call {
            pallet_utility::Call::batch { calls }
            | pallet_utility::Call::batch_all { calls }
            | pallet_utility::Call::force_batch { calls } => calls,
            _ => return None,
        };

        if calls.is_empty() {
            return None;
        }

        let mut common_real: Option<AccountIdOf<T>> = None;

        for call in calls.iter() {
            let call_ref: &RuntimeCallOf<T> = call.into_ref();
            let Some((inner_real, inner_delegate, _)) =
                Self::extract_proxy_parts(call_ref, outer_real)
            else {
                return None;
            };

            match &common_real {
                None => {
                    // Check RealPaysFee once on the first item and memoize. Batch fee
                    // propagation only supports homogeneous `proxy` calls, so a single
                    // read covers the entire batch.
                    if !pallet_proxy::Pallet::<T>::is_real_pays_fee(&inner_real, &inner_delegate) {
                        return None;
                    }
                    common_real = Some(inner_real);
                }
                Some(existing) if *existing != inner_real => return None,
                _ => {}
            }
        }

        common_real
    }
}

impl<T: Config + pallet_proxy::Config + pallet_utility::Config>
    TransactionExtension<RuntimeCallOf<T>> for ChargeTransactionPaymentWrapper<T>
where
    RuntimeCallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_proxy::Call<T>>
        + IsSubType<pallet_utility::Call<T>>,
    RuntimeOriginOf<T>: AsSystemOriginSigner<AccountIdOf<T>>
        + Clone
        + From<frame_system::RawOrigin<AccountIdOf<T>>>,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPaymentWrapper";
    type Implicit = ();
    type Val = Val<T>;
    type Pre = Pre<T>;

    fn weight(&self, call: &RuntimeCallOf<T>) -> Weight {
        // Account for up to four storage reads in the worst-case fee payer resolution
        // (three proxy hops + margin).
        self.inner
            .weight(call)
            .saturating_add(T::DbWeight::get().reads(4))
    }

    fn validate(
        &self,
        origin: DispatchOriginOf<RuntimeCallOf<T>>,
        call: &RuntimeCallOf<T>,
        info: &DispatchInfoOf<RuntimeCallOf<T>>,
        len: usize,
        self_implicit: Self::Implicit,
        inherited_implication: &impl Implication,
        source: TransactionSource,
    ) -> ValidateResult<Self::Val, RuntimeCallOf<T>> {
        let overridden_priority = {
            let base: TransactionPriority = match info.class {
                DispatchClass::Normal => NORMAL_DISPATCH_BASE_PRIORITY,
                DispatchClass::Mandatory => NORMAL_DISPATCH_BASE_PRIORITY,
                DispatchClass::Operational => OPERATIONAL_DISPATCH_PRIORITY,
            };
            base.saturated_into::<TransactionPriority>()
        };

        // If a real account opted in to pay fees, create a synthetic origin for fee validation.
        // Otherwise, the signer pays as usual.
        let fee_origin = if let Some(real) = Self::extract_real_fee_payer(call, &origin) {
            frame_system::RawOrigin::Signed(real).into()
        } else {
            origin.clone()
        };

        let (mut valid_transaction, val, _fee_origin) = self.inner.validate(
            fee_origin,
            call,
            info,
            len,
            self_implicit,
            inherited_implication,
            source,
        )?;

        valid_transaction.priority = overridden_priority;

        // Always return the original origin so the actual signer remains
        // the origin for dispatch and all subsequent extensions.
        Ok((valid_transaction, val, origin))
    }

    fn prepare(
        self,
        val: Self::Val,
        origin: &DispatchOriginOf<RuntimeCallOf<T>>,
        call: &RuntimeCallOf<T>,
        info: &DispatchInfoOf<RuntimeCallOf<T>>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        self.inner.prepare(val, origin, call, info, len)
    }

    fn metadata() -> Vec<TransactionExtensionMetadata> {
        ChargeTransactionPayment::<T>::metadata()
    }

    fn post_dispatch_details(
        pre: Self::Pre,
        info: &DispatchInfoOf<RuntimeCallOf<T>>,
        post_info: &PostDispatchInfoOf<RuntimeCallOf<T>>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<Weight, TransactionValidityError> {
        ChargeTransactionPayment::<T>::post_dispatch_details(pre, info, post_info, len, result)
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<RuntimeCallOf<T>>,
        post_info: &mut PostDispatchInfoOf<RuntimeCallOf<T>>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        ChargeTransactionPayment::<T>::post_dispatch(pre, info, post_info, len, result)
    }

    fn bare_validate(
        call: &RuntimeCallOf<T>,
        info: &DispatchInfoOf<RuntimeCallOf<T>>,
        len: usize,
    ) -> TransactionValidity {
        ChargeTransactionPayment::<T>::bare_validate(call, info, len)
    }

    fn bare_validate_and_prepare(
        call: &RuntimeCallOf<T>,
        info: &DispatchInfoOf<RuntimeCallOf<T>>,
        len: usize,
    ) -> Result<(), TransactionValidityError> {
        ChargeTransactionPayment::<T>::bare_validate_and_prepare(call, info, len)
    }

    fn bare_post_dispatch(
        info: &DispatchInfoOf<RuntimeCallOf<T>>,
        post_info: &mut PostDispatchInfoOf<RuntimeCallOf<T>>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        ChargeTransactionPayment::<T>::bare_post_dispatch(info, post_info, len, result)
    }
}
