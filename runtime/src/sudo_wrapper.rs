use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::{DispatchClass, DispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use frame_system::Config;
use pallet_shield::AuthorityOriginExt;
use pallet_shield::Call as MevShieldCall;
use pallet_sudo::Call as SudoCall;
use scale_info::TypeInfo;
use sp_runtime::impl_tx_ext_default;
use sp_runtime::traits::{
    AsSystemOriginSigner, DispatchInfoOf, Dispatchable, Implication, TransactionExtension,
    ValidateResult,
};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionSource};
use sp_std::marker::PhantomData;
use subtensor_macros::freeze_struct;

#[freeze_struct("99dce71278b36b44")]
#[derive(Default, Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
pub struct SudoTransactionExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for SudoTransactionExtension<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "SudoTransactionExtension",)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync + TypeInfo> SudoTransactionExtension<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<T> TransactionExtension<<T as Config>::RuntimeCall> for SudoTransactionExtension<T>
where
    T: Config + Send + Sync + TypeInfo + pallet_sudo::Config + pallet_shield::Config,
    <T as Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as Config>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
    <T as Config>::RuntimeCall: IsSubType<SudoCall<T>> + IsSubType<MevShieldCall<T>>,
{
    const IDENTIFIER: &'static str = "SudoTransactionExtension";

    type Implicit = ();
    type Val = ();
    type Pre = ();

    impl_tx_ext_default!(<T as Config>::RuntimeCall; weight prepare);

    fn validate(
        &self,
        origin: <T as Config>::RuntimeOrigin,
        call: &<T as Config>::RuntimeCall,
        info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, <T as Config>::RuntimeCall> {
        // --------------------------------------------------------------------
        // 1) pallet_sudo exception:
        // If this is a sudo call, require it be signed by the configured sudo key.
        // --------------------------------------------------------------------
        if let Some(_sudo_call) = IsSubType::<SudoCall<T>>::is_sub_type(call) {
            // Sudo extrinsics must be signed.
            let Some(who) = origin.as_system_origin_signer() else {
                return Err(InvalidTransaction::BadSigner.into());
            };

            let sudo_key = pallet_sudo::pallet::Key::<T>::get();

            // No sudo key configured → reject.
            let Some(expected_who) = sudo_key else {
                return Err(InvalidTransaction::BadSigner.into());
            };

            // Signer does not match the sudo key → reject.
            if *who != expected_who {
                return Err(InvalidTransaction::BadSigner.into());
            }

            // Valid sudo transaction → allow into pool.
            return Ok((Default::default(), (), origin));
        }

        // --------------------------------------------------------------------
        // 2) Generic BadOrigin spam prevention for *all pallets*:
        //
        // Default rule:
        //   - If DispatchClass is Operational, only allow Root-origin transactions.
        //   - If a *signed* tx is Operational but NOT Root → reject from the pool.
        //
        // Remaining exception (Operational but NOT Root-only):
        //   - MevShield::announce_next_key : must pass T::AuthorityOrigin::ensure_validator(origin)
        // --------------------------------------------------------------------
        if info.class == DispatchClass::Operational {
            // Always allow true Root origins.
            if frame_system::ensure_root(origin.clone()).is_ok() {
                return Ok((Default::default(), (), origin));
            }

            // Exception: MevShield::announce_next_key (Operational, but signed-validator origin)
            if let Some(mev_call) = IsSubType::<MevShieldCall<T>>::is_sub_type(call) {
                match mev_call {
                    MevShieldCall::announce_next_key { .. } => {
                        // Only a current Aura validator may call this.
                        if T::AuthorityOrigin::ensure_validator(origin.clone()).is_err() {
                            return Err(InvalidTransaction::BadSigner.into());
                        }
                        return Ok((Default::default(), (), origin));
                    }
                    _ => {}
                }
            }

            // Default Operational rule: signed Operational txs that aren't Root and aren't
            // one of the allowed exceptions are rejected from the pool.
            if origin.as_system_origin_signer().is_some() {
                return Err(InvalidTransaction::Call.into());
            }
        }

        Ok((Default::default(), (), origin))
    }
}
