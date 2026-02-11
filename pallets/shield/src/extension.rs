use crate::{Call, Config, NextKey, ShieldedTransaction};
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
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, <T as frame_system::Config>::RuntimeCall> {
        // Ensure the transaction is signed, else we just skip the extension.
        let Some(_who) = origin.as_system_origin_signer() else {
            return Ok((Default::default(), (), origin));
        };

        // Ensure the transaction is encrypted
        let Some(Call::submit_encrypted { ciphertext }) = IsSubType::<Call<T>>::is_sub_type(call)
        else {
            return Err(InvalidTransaction::BadSigner.into());
        };

        let Some(shielded_tx) = ShieldedTransaction::parse(&ciphertext) else {
            return Err(InvalidTransaction::BadProof.into());
        };

        let next_key_hash = NextKey::<T>::get().map(|key| twox_128(&key.into_inner()[..]));

        // The transaction must be encrypted with the next key or we discard it
        if next_key_hash.is_none() || next_key_hash.is_some_and(|hash| hash != shielded_tx.key_hash)
        {
            return Err(InvalidTransaction::BadProof)?;
        }

        Ok((Default::default(), (), origin))
    }
}
