use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::{
    dispatch::{DispatchInfo, GetDispatchInfo},
    traits::{InherentBuilder, IsSubType, SignedTransactionBuilder},
};
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::TypeInfo;
use serde;
use sp_core::{H256, U256};
// use serde::{Deserialize, Serialize};
use pallet_revive::evm::runtime::EthExtra;
use sp_runtime::{
    OpaqueExtrinsic, RuntimeDebug,
    generic::{self, Preamble},
    traits::{
        self, Checkable, Dispatchable, ExtrinsicCall, ExtrinsicLike, ExtrinsicMetadata,
        IdentifyAccount, MaybeDisplay, Member, TransactionExtension,
    },
    transaction_validity::{InvalidTransaction, TransactionValidityError},
};
// use crate::TransactionExtensions;

// use crate::CheckedExtrinsic;
use fp_self_contained::{CheckedExtrinsic, CheckedSignature, SelfContainedCall};
/// A extrinsic right from the external world. This is unchecked and so
/// can contain a signature.
#[derive(PartialEq, Eq, Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct UncheckedExtrinsic<Address, Call, Signature, E: EthExtra>(
    pub generic::UncheckedExtrinsic<Address, Call, Signature, E::Extension>,
);

impl<Address, Call, Signature, E: EthExtra> UncheckedExtrinsic<Address, Call, Signature, E>
where
    E::Extension: TypeInfo,
{
    /// New instance of a signed extrinsic aka "transaction".
    pub fn new_signed(
        function: Call,
        signed: Address,
        signature: Signature,
        tx_ext: E::Extension,
    ) -> Self {
        Self(generic::UncheckedExtrinsic::new_signed(
            function, signed, signature, tx_ext,
        ))
    }

    /// New instance of an unsigned extrinsic aka "inherent".
    pub fn new_bare(function: Call) -> Self {
        Self(generic::UncheckedExtrinsic::new_bare(function))
    }
}

impl<Address: TypeInfo, Call: TypeInfo, Signature: TypeInfo, E: EthExtra> ExtrinsicLike
    for UncheckedExtrinsic<Address, Call, Signature, E>
{
    fn is_bare(&self) -> bool {
        ExtrinsicLike::is_bare(&self.0)
    }
}

impl<Address, AccountId, Call, Signature, Lookup, E> Checkable<Lookup>
    for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Address: Member + MaybeDisplay,
    Call: Encode
        + Member
        + SelfContainedCall
        + IsSubType<pallet_revive::Call<E::Config>>
        + From<pallet_revive::Call<E::Config>>,
    Signature: Member + traits::Verify,
    <Signature as traits::Verify>::Signer: IdentifyAccount<AccountId = AccountId>,
    E::Extension: Encode + TransactionExtension<Call>,
    <E::Config as frame_system::Config>::Nonce: TryFrom<U256>,
    <E::Config as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo>,
    pallet_revive::BalanceOf<E::Config>: Into<U256> + TryFrom<U256>,
    pallet_revive::MomentOf<E::Config>: Into<U256>,
    <E::Config as frame_system::Config>::RuntimeCall:
        From<pallet_revive::Call<E::Config>> + IsSubType<pallet_revive::Call<E::Config>>,
    <<E::Config as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<E::Config>>::Balance: Into<pallet_revive::BalanceOf<E::Config>>,
    AccountId: Member + MaybeDisplay,
    Lookup: traits::Lookup<Source = Address, Target = AccountId>,
    E: EthExtra,
    // CallOf<E::Config>: From<crate::Call<E::Config>> + IsSubType<crate::Call<E::Config>>,
    <E::Config as frame_system::Config>::Hash: frame_support::traits::IsType<H256>,
{
    type Checked =
        CheckedExtrinsic<AccountId, Call, E::Extension, <Call as SelfContainedCall>::SignedInfo>;

    fn check(self, lookup: &Lookup) -> Result<Self::Checked, TransactionValidityError> {
        if self.0.function.is_self_contained() {
            if matches!(self.0.preamble, Preamble::Signed(_, _, _)) {
                return Err(TransactionValidityError::Invalid(
                    InvalidTransaction::BadProof,
                ));
            }

            let signed_info = self.0.function.check_self_contained().ok_or(
                TransactionValidityError::Invalid(InvalidTransaction::BadProof),
            )??;
            Ok(CheckedExtrinsic {
                signed: CheckedSignature::SelfContained(signed_info),
                function: self.0.function,
            })
        } else {
            if !self.0.is_signed() {
                if let Some(pallet_revive::Call::eth_transact { payload }) =
                    self.0.function.is_sub_type()
                {
                    let checked = E::try_into_checked_extrinsic(
                        payload.to_vec(),
                        self.0.function.encoded_size(),
                    )?;
                    return Ok(CheckedExtrinsic {
                        signed: CheckedSignature::GenericDelegated(checked.format.into()),
                        function: checked.function.into(),
                    })
                };
            }
            // self.0.check(lookup)

            let checked = Checkable::<Lookup>::check(self.0, lookup)?;
            Ok(CheckedExtrinsic {
                signed: CheckedSignature::GenericDelegated(checked.format),
                function: checked.function,
            })
        }
    }

    #[cfg(feature = "try-runtime")]
    fn unchecked_into_checked_i_know_what_i_am_doing(
        self,
        lookup: &Lookup,
    ) -> Result<Self::Checked, TransactionValidityError> {
        use generic::ExtrinsicFormat;
        if self.0.function.is_self_contained() {
            match self.0.function.check_self_contained() {
                Some(signed_info) => Ok(CheckedExtrinsic {
                    signed: match signed_info {
                        Ok(info) => CheckedSignature::SelfContained(info),
                        _ => CheckedSignature::GenericDelegated(ExtrinsicFormat::Bare),
                    },
                    function: self.0.function,
                }),
                None => Ok(CheckedExtrinsic {
                    signed: CheckedSignature::GenericDelegated(ExtrinsicFormat::Bare),
                    function: self.0.function,
                }),
            }
        } else {
            let checked =
                Checkable::<Lookup>::unchecked_into_checked_i_know_what_i_am_doing(self.0, lookup)?;
            Ok(CheckedExtrinsic {
                signed: CheckedSignature::GenericDelegated(checked.format),
                function: checked.function,
            })
        }
    }
}

impl<Address, Call, Signature, E> ExtrinsicMetadata
    for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Call: Dispatchable,
    E::Extension: TransactionExtension<Call>,
    E: EthExtra,
{
    const VERSIONS: &'static [u8] =
        generic::UncheckedExtrinsic::<Address, Call, Signature, E::Extension>::VERSIONS;
    type TransactionExtensions = E::Extension;
}

impl<Address, Call, Signature, E> ExtrinsicCall for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Address: TypeInfo,
    Call: TypeInfo,
    Signature: TypeInfo,
    E::Extension: TypeInfo,
    E: EthExtra,
{
    type Call = Call;

    fn call(&self) -> &Self::Call {
        &self.0.function
    }
}

impl<Address, Call, Signature, E: EthExtra> GetDispatchInfo
    for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Call: GetDispatchInfo + Dispatchable,
    E::Extension: TransactionExtension<Call>,
{
    fn get_dispatch_info(&self) -> DispatchInfo {
        self.0.function.get_dispatch_info()
    }
}

// #[cfg(feature = "serde")]
impl<Address: Encode, Signature: Encode, Call, E: EthExtra> serde::Serialize
    for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Call: Encode + Dispatchable,
    E::Extension: Encode + TransactionExtension<Call>,
{
    fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(seq)
    }
}

// #[cfg(feature = "serde")]
impl<'a, Address: Decode, Signature: Decode, Call, E: EthExtra> serde::Deserialize<'a>
    for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Call: Decode + Dispatchable + DecodeWithMemTracking,
    E::Extension: Decode + TransactionExtension<Call>,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        <generic::UncheckedExtrinsic<Address, Call, Signature, E::Extension>>::deserialize(de)
            .map(Self)
    }
}

impl<Address, Signature, Call, E: EthExtra> SignedTransactionBuilder
    for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Address: TypeInfo,
    Signature: TypeInfo,
    Call: TypeInfo,
    E::Extension: TypeInfo,
{
    type Address = Address;
    type Signature = Signature;
    type Extension = E::Extension;

    fn new_signed_transaction(
        call: Self::Call,
        signed: Address,
        signature: Signature,
        tx_ext: E::Extension,
    ) -> Self {
        generic::UncheckedExtrinsic::new_signed(call, signed, signature, tx_ext).into()
    }
}

impl<Address, Signature, Call, E: EthExtra> InherentBuilder
    for UncheckedExtrinsic<Address, Call, Signature, E>
where
    Address: TypeInfo,
    Signature: TypeInfo,
    Call: TypeInfo,
    E::Extension: TypeInfo,
{
    fn new_inherent(call: Self::Call) -> Self {
        generic::UncheckedExtrinsic::new_bare(call).into()
    }
}

impl<Address, Call, Signature, E: EthExtra> From<UncheckedExtrinsic<Address, Call, Signature, E>>
    for OpaqueExtrinsic
where
    Address: Encode,
    Signature: Encode,
    Call: Encode,
    E::Extension: Encode,
    E: Encode,
{
    fn from(extrinsic: UncheckedExtrinsic<Address, Call, Signature, E>) -> Self {
        extrinsic.0.into()
    }
}

impl<Address, Call, Signature, E: EthExtra>
    From<generic::UncheckedExtrinsic<Address, Call, Signature, E::Extension>>
    for UncheckedExtrinsic<Address, Call, Signature, E>
{
    fn from(utx: generic::UncheckedExtrinsic<Address, Call, Signature, E::Extension>) -> Self {
        Self(utx)
    }
}
