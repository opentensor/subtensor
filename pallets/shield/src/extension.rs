use crate::{Call, Config, CurrentKey, NextKey, ShieldedTransaction};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::pallet_prelude::*;
use frame_support::traits::IsSubType;
use scale_info::TypeInfo;
use sp_io::hashing::twox_128;
use sp_runtime::impl_tx_ext_default;
use sp_runtime::traits::{
    AsSystemOriginSigner, DispatchInfoOf, Dispatchable, Implication, TransactionExtension,
    ValidateResult,
};
use sp_runtime::transaction_validity::TransactionSource;
use subtensor_macros::freeze_struct;

#[freeze_struct("dabd89c6963de25d")]
#[derive(Default, Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
pub struct CheckShieldedTxValidity<T: Config + Send + Sync + TypeInfo>(PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> CheckShieldedTxValidity<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for CheckShieldedTxValidity<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "CheckShieldedTxValidity")
    }
}

impl<T: Config + Send + Sync + TypeInfo>
    TransactionExtension<<T as frame_system::Config>::RuntimeCall> for CheckShieldedTxValidity<T>
where
    <T as frame_system::Config>::RuntimeCall: Dispatchable + IsSubType<Call<T>>,
    <T as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId>,
{
    const IDENTIFIER: &'static str = "CheckShieldedTxValidity";

    type Implicit = ();
    type Val = ();
    type Pre = ();

    impl_tx_ext_default!(<T as frame_system::Config>::RuntimeCall; weight prepare);

    fn validate(
        &self,
        origin: <T as frame_system::Config>::RuntimeOrigin,
        call: &<T as frame_system::Config>::RuntimeCall,
        _info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        source: TransactionSource,
    ) -> ValidateResult<Self::Val, <T as frame_system::Config>::RuntimeCall> {
        // Ensure the transaction is signed, else we just skip the extension.
        let Some(_who) = origin.as_system_origin_signer() else {
            return Ok((Default::default(), (), origin));
        };

        // Ensure the transaction is a shielded transaction, else we just skip the extension.
        let Some(Call::submit_encrypted { ciphertext }) = IsSubType::<Call<T>>::is_sub_type(call)
        else {
            return Ok((Default::default(), (), origin));
        };

        // Reject malformed ciphertext regardless of source.
        let Some(ShieldedTransaction { key_hash, .. }) = ShieldedTransaction::parse(&ciphertext)
        else {
            return Err(InvalidTransaction::BadProof.into());
        };

        // Only enforce the key_hash check during block building/import.
        // The fork-aware tx pool validates against multiple views (recent block states),
        // and stale views may not contain the key the tx was encrypted with,
        // causing spurious rejections. Pool validation only checks structure above.
        if source == TransactionSource::InBlock {
            let matches_any = [CurrentKey::<T>::get(), NextKey::<T>::get()]
                .iter()
                .any(|k| k.as_ref().is_some_and(|k| twox_128(&k[..]) == key_hash));

            if !matches_any {
                return Err(InvalidTransaction::BadProof.into());
            }
        }

        Ok((Default::default(), (), origin))
    }
}
