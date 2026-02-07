use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use frame_system::Config;
use pallet_sudo::Call as SudoCall;
use scale_info::TypeInfo;
use sp_runtime::{
    impl_tx_ext_default,
    traits::{
        AsSystemOriginSigner, DispatchInfoOf, Dispatchable, Implication, TransactionExtension,
        ValidateResult,
    },
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionSource, ValidTransaction,
    },
};
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

impl<T: Config + Send + Sync + TypeInfo + pallet_sudo::Config>
    TransactionExtension<<T as Config>::RuntimeCall> for SudoTransactionExtension<T>
where
    <T as Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as Config>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
    <T as Config>::RuntimeCall: IsSubType<SudoCall<T>>,
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
        _info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, <T as Config>::RuntimeCall> {
        // Ensure the transaction is signed, else we just skip the extension.
        let Some(who) = origin.as_system_origin_signer() else {
            return Ok((Default::default(), (), origin));
        };

        // Check validity of the signer for sudo call
        if let Some(_sudo_call) = IsSubType::<pallet_sudo::Call<T>>::is_sub_type(call) {
            let sudo_key = pallet_sudo::pallet::Key::<T>::get();

            // No sudo key configured → reject
            let Some(expected_who) = sudo_key else {
                return Err(InvalidTransaction::BadSigner.into());
            };

            // Signer does not match the sudo key → reject
            if *who != expected_who {
                return Err(InvalidTransaction::BadSigner.into());
            }

            // We bump the priority of the transaction to the maximum possible
            let mut valid_transaction = ValidTransaction::default();
            valid_transaction.priority = TransactionPriority::max_value();

            return Ok((valid_transaction, (), origin));
        }

        Ok((Default::default(), (), origin))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        derive_impl, dispatch::GetDispatchInfo, dispatch::RawOrigin, traits::OriginTrait,
    };
    use sp_runtime::{
        BuildStorage, traits::TxBaseImplication, transaction_validity::TransactionValidityError,
    };
    use sp_std::{boxed::Box, vec};

    #[test]
    fn skip_validation_if_not_signed() {
        new_test_ext().execute_with(|| {
            let ext = SudoTransactionExtension::<Test>::new();
            let call = sudo_remark_call();
            let info = call.get_dispatch_info();
            let len = call.using_encoded(|b| b.len());
            let implicit = ();
            let implication = TxBaseImplication(());
            let source = TransactionSource::External;

            let origin = RuntimeOrigin::none();
            let (valid, _, returned_origin) = ext
                .validate(origin, &call, &info, len, implicit, &implication, source)
                .unwrap();
            assert_eq!(valid, Default::default());
            assert!(matches!(
                returned_origin.as_system_ref(),
                Some(RawOrigin::None)
            ));

            let origin = RuntimeOrigin::root();
            let (valid, _, returned_origin) = ext
                .validate(origin, &call, &info, len, implicit, &implication, source)
                .unwrap();
            assert_eq!(valid, Default::default());
            assert!(matches!(
                returned_origin.as_system_ref(),
                Some(RawOrigin::Root)
            ));
        });
    }

    #[test]
    fn skip_validation_if_signed_but_not_sudo_call() {
        new_test_ext().execute_with(|| {
            let ext = SudoTransactionExtension::<Test>::new();
            let call = call_remark();
            let info = call.get_dispatch_info();
            let len = call.using_encoded(|b| b.len());
            let implicit = ();
            let implication = TxBaseImplication(());
            let source = TransactionSource::External;
            let origin = RuntimeOrigin::signed(42);

            pallet_sudo::Key::<Test>::put(42);

            let (valid, _, returned_origin) = ext
                .validate(origin, &call, &info, len, implicit, &implication, source)
                .unwrap();

            assert_eq!(valid, Default::default());
            assert!(matches!(
                returned_origin.as_system_ref(),
                Some(RawOrigin::Signed(42))
            ));
        });
    }

    #[test]
    fn error_if_no_sudo_key_configured() {
        new_test_ext().execute_with(|| {
            let ext = SudoTransactionExtension::<Test>::new();
            let call = sudo_remark_call();
            let info = call.get_dispatch_info();
            let len = call.using_encoded(|b| b.len());
            let implicit = ();
            let implication = TxBaseImplication(());
            let source = TransactionSource::External;
            let origin = RuntimeOrigin::signed(42);

            assert_eq!(
                ext.validate(origin, &call, &info, len, implicit, &implication, source)
                    .unwrap_err(),
                TransactionValidityError::Invalid(InvalidTransaction::BadSigner),
            );
        });
    }

    #[test]
    fn error_if_signed_but_not_from_sudo() {
        new_test_ext().execute_with(|| {
            let ext = SudoTransactionExtension::<Test>::new();
            let call = sudo_remark_call();
            let info = call.get_dispatch_info();
            let len = call.using_encoded(|b| b.len());
            let implicit = ();
            let implication = TxBaseImplication(());
            let source = TransactionSource::External;
            let origin = RuntimeOrigin::signed(42);

            pallet_sudo::Key::<Test>::put(99);

            assert_eq!(
                ext.validate(origin, &call, &info, len, implicit, &implication, source)
                    .unwrap_err(),
                TransactionValidityError::Invalid(InvalidTransaction::BadSigner),
            );
        });
    }

    #[test]
    fn priority_is_set_to_max_value_for_root_origin() {
        new_test_ext().execute_with(|| {
            let ext = SudoTransactionExtension::<Test>::new();
            let call = sudo_remark_call();
            let info = call.get_dispatch_info();
            let len = call.using_encoded(|b| b.len());
            let implicit = ();
            let implication = TxBaseImplication(());
            let source = TransactionSource::External;
            let origin = RuntimeOrigin::signed(42);

            pallet_sudo::Key::<Test>::put(42);

            let (valid, _, returned_origin) = ext
                .validate(origin, &call, &info, len, implicit, &implication, source)
                .unwrap();

            assert_eq!(valid.priority, TransactionPriority::max_value());
            assert!(matches!(
                returned_origin.as_system_ref(),
                Some(RawOrigin::Signed(42))
            ));
        });
    }

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(RuntimeCall, RuntimeEvent, RuntimeError, RuntimeOrigin, RuntimeTask)]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type Sudo = pallet_sudo;
    }

    type Block = frame_system::mocking::MockBlock<Test>;

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl frame_system::Config for Test {
        type Block = Block;
    }

    #[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
    impl pallet_sudo::Config for Test {}

    pub fn new_test_ext() -> sp_io::TestExternalities {
        frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .into()
    }

    fn call_remark() -> RuntimeCall {
        RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
    }

    fn sudo_remark_call() -> RuntimeCall {
        RuntimeCall::Sudo(pallet_sudo::Call::sudo {
            call: Box::new(call_remark()),
        })
    }
}
