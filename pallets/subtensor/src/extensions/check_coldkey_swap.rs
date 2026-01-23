use crate::{Call, ColdkeySwapAnnouncements, ColdkeySwapDisputes, Config, CustomTransactionError};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use pallet_subtensor_proxy::Call as ProxyCall;
use scale_info::TypeInfo;
use sp_runtime::{
    impl_tx_ext_default,
    traits::{
        AsSystemOriginSigner, DispatchInfoOf, Dispatchable, Implication, StaticLookup,
        TransactionExtension, ValidateResult,
    },
    transaction_validity::TransactionSource,
};
use sp_std::marker::PhantomData;
use subtensor_macros::freeze_struct;

type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
type OriginOf<T> = <T as frame_system::Config>::RuntimeOrigin;
type LookupOf<T> = <T as frame_system::Config>::Lookup;

#[freeze_struct("483277dc74a5aa56")]
#[derive(Default, Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
pub struct CheckColdkeySwap<T: Config + TypeInfo + Send + Sync>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for CheckColdkeySwap<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "CheckColdkeySwap")
    }
}

impl<T: Config + Send + Sync + TypeInfo + pallet_shield::Config + pallet_subtensor_proxy::Config>
    TransactionExtension<CallOf<T>> for CheckColdkeySwap<T>
where
    CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<Call<T>>
        + IsSubType<pallet_subtensor_proxy::Call<T>>
        + IsSubType<pallet_shield::Call<T>>,
    OriginOf<T>: AsSystemOriginSigner<T::AccountId> + Clone,
{
    const IDENTIFIER: &'static str = "CheckColdkeySwap";

    type Implicit = ();
    type Val = ();
    type Pre = ();

    fn validate(
        &self,
        origin: OriginOf<T>,
        call: &CallOf<T>,
        _info: &DispatchInfoOf<CallOf<T>>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, CallOf<T>> {
        // Ensure the transaction is signed, else we just skip the extension.
        let Some(who) = origin.as_system_origin_signer() else {
            return Ok((Default::default(), (), origin));
        };

        // Get the real account if we are behind a proxy.
        let who =
            if let Some(ProxyCall::proxy { real, .. } | ProxyCall::proxy_announced { real, .. }) =
                call.is_sub_type()
            {
                LookupOf::<T>::lookup(real.clone())
                    .map_err(|_| CustomTransactionError::InvalidRealAccount)?
            } else {
                who.clone()
            };

        if ColdkeySwapAnnouncements::<T>::contains_key(&who) {
            if ColdkeySwapDisputes::<T>::contains_key(&who) {
                return Err(CustomTransactionError::ColdkeySwapDisputed.into());
            }

            let is_allowed_direct = matches!(
                call.is_sub_type(),
                Some(Call::announce_coldkey_swap { .. } | Call::swap_coldkey_announced { .. })
            );

            let is_mev_protected = matches!(
                IsSubType::<pallet_shield::Call<T>>::is_sub_type(call),
                Some(pallet_shield::Call::submit_encrypted { .. })
            );

            if !is_allowed_direct && !is_mev_protected {
                return Err(CustomTransactionError::ColdkeySwapAnnounced.into());
            }
        }

        Ok((Default::default(), (), origin))
    }

    impl_tx_ext_default!(CallOf<T>; weight prepare);
}
