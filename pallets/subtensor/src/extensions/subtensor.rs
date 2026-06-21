use crate::{Config, Error};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::{DispatchExtension, DispatchInfo, PostDispatchInfo};
use scale_info::TypeInfo;
use sp_runtime::traits::{
    DispatchInfoOf, Dispatchable, Implication, TransactionExtension, ValidateResult,
};
use sp_runtime::{
    DispatchError, impl_tx_ext_default,
    transaction_validity::{TransactionSource, TransactionValidityError},
};
use sp_std::marker::PhantomData;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::CustomTransactionError;

type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
type OriginOf<T> = <T as frame_system::Config>::RuntimeOrigin;

#[allow(deprecated)]
impl<T: Config> From<Error<T>> for CustomTransactionError {
    fn from(error: Error<T>) -> Self {
        match error {
            Error::<T>::AmountTooLow | Error::<T>::NotEnoughStakeToSetWeights => {
                Self::StakeAmountTooLow
            }
            Error::<T>::SubnetNotExists => Self::SubnetNotExists,
            Error::<T>::NotEnoughBalanceToStake => Self::BalanceTooLow,
            Error::<T>::HotKeyAccountNotExists => Self::HotkeyAccountDoesntExist,
            Error::<T>::NotEnoughStakeToWithdraw => Self::NotEnoughStakeToWithdraw,
            Error::<T>::InsufficientLiquidity => Self::InsufficientLiquidity,
            Error::<T>::SlippageTooHigh => Self::SlippageTooHigh,
            Error::<T>::TransferDisallowed => Self::TransferDisallowed,
            Error::<T>::HotKeyNotRegisteredInNetwork => Self::HotKeyNotRegisteredInNetwork,
            Error::<T>::InvalidIpAddress => Self::InvalidIpAddress,
            Error::<T>::ServingRateLimitExceeded => Self::ServingRateLimitExceeded,
            Error::<T>::InvalidPort => Self::InvalidPort,
            Error::<T>::NonAssociatedColdKey => Self::NonAssociatedColdKey,
            Error::<T>::DelegateTakeTooLow => Self::DelegateTakeTooLow,
            Error::<T>::DelegateTakeTooHigh => Self::DelegateTakeTooHigh,
            Error::<T>::InputLengthsUnequal => Self::InputLengthsUnequal,
            Error::<T>::NoWeightsCommitFound => Self::CommitNotFound,
            Error::<T>::RevealTooEarly => Self::CommitBlockNotInRevealRange,
            Error::<T>::InvalidRevealRound => Self::InvalidRevealRound,
            Error::<T>::CommittingWeightsTooFast
            | Error::<T>::SettingWeightsTooFast
            | Error::<T>::NetworkTxRateLimitExceeded => Self::RateLimitExceeded,
            Error::<T>::HotKeyNotRegisteredInSubNet => Self::UidNotFound,
            Error::<T>::EvmKeyAssociateRateLimitExceeded => Self::EvmKeyAssociateRateLimitExceeded,
            Error::<T>::ColdkeySwapAnnounced => Self::ColdkeyInSwapSchedule,
            Error::<T>::ColdkeySwapDisputed => Self::ColdkeySwapDisputed,
            _ => Self::BadRequest,
        }
    }
}

#[freeze_struct("2e02eb32e5cb25d3")]
#[derive(Default, Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
pub struct SubtensorTransactionExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for SubtensorTransactionExtension<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "SubtensorTransactionExtension")
    }
}

impl<T: Config + Send + Sync + TypeInfo> SubtensorTransactionExtension<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    fn map_error(error: DispatchError) -> CustomTransactionError {
        let DispatchError::Module(module_error) = error.stripped() else {
            return CustomTransactionError::BadRequest;
        };

        if usize::from(module_error.index)
            != <crate::Pallet<T> as frame_support::traits::PalletInfoAccess>::index()
        {
            return CustomTransactionError::BadRequest;
        }

        <Error<T> as Decode>::decode(&mut &module_error.error[..])
            .map(Into::into)
            .unwrap_or(CustomTransactionError::BadRequest)
    }

    fn check(origin: &OriginOf<T>, call: &CallOf<T>) -> Result<(), DispatchError>
    where
        CallOf<T>: Dispatchable<RuntimeOrigin = OriginOf<T>>,
    {
        <<T as frame_system::Config>::DispatchExtension as DispatchExtension<CallOf<T>>>::pre_dispatch(
            origin, call,
        )
        .map(|_| ())
        .map_err(|error| error.error)
    }
}

impl<T> TransactionExtension<CallOf<T>> for SubtensorTransactionExtension<T>
where
    T: Config + Send + Sync + TypeInfo,
    CallOf<T>:
        Dispatchable<RuntimeOrigin = OriginOf<T>, Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    OriginOf<T>: Clone,
{
    const IDENTIFIER: &'static str = "SubtensorTransactionExtension";

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
        Self::check(&origin, call)
            .map(|()| (Default::default(), (), origin))
            .map_err(|error| TransactionValidityError::from(Self::map_error(error)))
    }

    impl_tx_ext_default!(CallOf<T>; weight prepare);
}
