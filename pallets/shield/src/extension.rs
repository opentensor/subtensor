use crate::{Call, Config, ShieldedTransaction};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::{DispatchClass, GetDispatchInfo};
use frame_support::pallet_prelude::*;
use frame_support::traits::IsSubType;
use scale_info::TypeInfo;
use sp_runtime::impl_tx_ext_default;
use sp_runtime::traits::{
    AsSystemOriginSigner, DispatchInfoOf, Dispatchable, Implication, TransactionExtension,
    ValidateResult,
};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionSource};
use stp_mev_shield_ibe::IbeEncryptedExtrinsicV1;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::CustomTransactionError;

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

impl<T> TransactionExtension<<T as Config>::RuntimeCall> for CheckShieldedTxValidity<T>
where
    T: Config + Send + Sync + TypeInfo,
    <T as Config>::RuntimeCall: Dispatchable + GetDispatchInfo + IsSubType<Call<T>>,
    <<T as Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
        AsSystemOriginSigner<<T as frame_system::Config>::AccountId>,
{
    const IDENTIFIER: &'static str = "CheckShieldedTxValidity";

    type Implicit = ();
    type Val = ();
    type Pre = ();

    impl_tx_ext_default!(<T as Config>::RuntimeCall; prepare);

    fn weight(&self, _call: &<T as Config>::RuntimeCall) -> Weight {
        // Some arbitrary weight added to account for the cost
        // of reading the PendingKey from the proposer.
        Weight::from_parts(1_000_000, 0).saturating_add(T::DbWeight::get().reads(5))
    }

    fn validate(
        &self,
        origin: <<T as Config>::RuntimeCall as Dispatchable>::RuntimeOrigin,
        call: &<T as Config>::RuntimeCall,
        _info: &DispatchInfoOf<<T as Config>::RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<(), <T as Config>::RuntimeCall> {
        // Ensure the transaction is signed, else we just skip the extension.
        if origin.as_system_origin_signer().is_none() {
            return Ok((Default::default(), (), origin));
        }

        // Encrypted-admission calls are not plaintext preemption. Keep the
        // existing ciphertext sanity checks for them, then allow them through.
        if let Some(Call::submit_encrypted { ciphertext }) = IsSubType::<Call<T>>::is_sub_type(call)
        {
            if IbeEncryptedExtrinsicV1::is_v2_prefixed(ciphertext.as_slice()) {
                if IbeEncryptedExtrinsicV1::decode_v2(ciphertext.as_slice()).is_err() {
                    return Err(CustomTransactionError::FailedShieldedTxParsing.into());
                }
            } else {
                let Some(ShieldedTransaction { .. }) = ShieldedTransaction::parse(ciphertext)
                else {
                    return Err(CustomTransactionError::FailedShieldedTxParsing.into());
                };
            }
            return Ok((Default::default(), (), origin));
        }

        if let Some(Call::store_encrypted { encrypted_call }) =
            IsSubType::<Call<T>>::is_sub_type(call)
        {
            if IbeEncryptedExtrinsicV1::is_v2_prefixed(encrypted_call.as_slice())
                && IbeEncryptedExtrinsicV1::decode_v2(encrypted_call.as_slice()).is_err()
            {
                return Err(CustomTransactionError::FailedShieldedTxParsing.into());
            }
            return Ok((Default::default(), (), origin));
        }

        // Runtime-level no-preemption invariant. If on_initialize left a due
        // threshold-IBE queue head behind, ordinary non-operational plaintext
        // transactions are invalid for this block. The drain-in-progress guard
        // lets decrypted encrypted inner extrinsics apply in FIFO order even
        // while additional due entries remain behind them.
        let dispatch_class = call.get_dispatch_info().class;
        if !crate::Pallet::<T>::is_ibe_queue_drain_in_progress()
            && dispatch_class != DispatchClass::Operational
            && crate::Pallet::<T>::has_due_ibe_queue_head()
        {
            return Err(InvalidTransaction::ExhaustsResources.into());
        }

        Ok((Default::default(), (), origin))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use frame_support::dispatch::GetDispatchInfo;
    use frame_support::pallet_prelude::{BoundedVec, ConstU32};
    use sp_runtime::traits::TxBaseImplication;
    use sp_runtime::transaction_validity::{TransactionValidityError, ValidTransaction};

    /// Build wire-format ciphertext with a given key_hash.
    /// Layout: key_hash(16) || kem_ct_len(2 LE) || kem_ct(N) || nonce(24) || aead_ct(rest)
    fn build_ciphertext(key_hash: [u8; 16]) -> BoundedVec<u8, ConstU32<8192>> {
        let kem_ct = [0xAA; 4];
        let nonce = [0xBB; 24];
        let aead_ct = [0xDD; 16];

        let mut buf = Vec::new();
        buf.extend_from_slice(&key_hash);
        buf.extend_from_slice(&(kem_ct.len() as u16).to_le_bytes());
        buf.extend_from_slice(&kem_ct);
        buf.extend_from_slice(&nonce);
        buf.extend_from_slice(&aead_ct);

        BoundedVec::truncate_from(buf)
    }

    fn make_submit_call(key_hash: [u8; 16]) -> RuntimeCall {
        RuntimeCall::MevShield(crate::Call::submit_encrypted {
            ciphertext: build_ciphertext(key_hash),
        })
    }

    fn validate_ext(
        who: Option<u64>,
        call: &RuntimeCall,
        source: TransactionSource,
    ) -> Result<ValidTransaction, TransactionValidityError> {
        let ext = CheckShieldedTxValidity::<Test>::new();
        let info = call.get_dispatch_info();
        let origin = match who {
            Some(id) => RuntimeOrigin::signed(id),
            None => RuntimeOrigin::none(),
        };
        ext.validate(origin, call, &info, 0, (), &TxBaseImplication(call), source)
            .map(|(validity, _, _)| validity)
    }

    #[test]
    fn non_shield_call_passes_through() {
        new_test_ext().execute_with(|| {
            let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
            let validity = validate_ext(Some(1), &call, TransactionSource::InBlock).unwrap();
            assert_eq!(validity.longevity, u64::MAX);
        });
    }

    #[test]
    fn unsigned_origin_passes_through() {
        new_test_ext().execute_with(|| {
            let call = make_submit_call([0xFF; 16]);
            let validity = validate_ext(None, &call, TransactionSource::InBlock).unwrap();
            assert_eq!(validity.longevity, u64::MAX);
        });
    }

    #[test]
    fn malformed_ciphertext_rejected_inblock() {
        new_test_ext().execute_with(|| {
            let call = RuntimeCall::MevShield(crate::Call::submit_encrypted {
                ciphertext: BoundedVec::truncate_from(vec![0u8; 5]),
            });
            assert_eq!(
                validate_ext(Some(1), &call, TransactionSource::InBlock),
                Err(CustomTransactionError::FailedShieldedTxParsing.into())
            );
        });
    }

    #[test]
    fn malformed_ciphertext_rejected_from_pool() {
        new_test_ext().execute_with(|| {
            let call = RuntimeCall::MevShield(crate::Call::submit_encrypted {
                ciphertext: BoundedVec::truncate_from(vec![0u8; 5]),
            });
            assert_eq!(
                validate_ext(Some(1), &call, TransactionSource::External),
                Err(CustomTransactionError::FailedShieldedTxParsing.into())
            );
        });
    }

    #[test]
    fn wellformed_ciphertext_accepted_inblock() {
        new_test_ext().execute_with(|| {
            let call = make_submit_call([0xFF; 16]);
            let validity = validate_ext(Some(1), &call, TransactionSource::InBlock).unwrap();
            assert_eq!(validity, ValidTransaction::default());
        });
    }

    #[test]
    fn wellformed_ciphertext_accepted_external() {
        new_test_ext().execute_with(|| {
            let call = make_submit_call([0xFF; 16]);
            let validity = validate_ext(Some(1), &call, TransactionSource::External).unwrap();
            assert_eq!(validity, ValidTransaction::default());
        });
    }

    #[test]
    fn wellformed_ciphertext_accepted_local() {
        new_test_ext().execute_with(|| {
            let call = make_submit_call([0xFF; 16]);
            let validity = validate_ext(Some(1), &call, TransactionSource::Local).unwrap();
            assert_eq!(validity, ValidTransaction::default());
        });
    }

    fn seed_ibe_queue_head(current_block: u64, target_block: u64) {
        System::set_block_number(current_block);
        crate::PendingExtrinsics::<Test>::insert(
            0,
            crate::PendingExtrinsic::<Test> {
                who: 1,
                encrypted_call: BoundedVec::truncate_from(vec![0xA5; 32]),
                submitted_at: current_block,
            },
        );
        crate::PendingIbeMetadata::<Test>::insert(
            0,
            crate::PendingIbeMeta::<Test> {
                epoch: 0,
                target_block,
                key_id: [0u8; stp_mev_shield_ibe::KEY_ID_LEN],
                commitment: sp_core::H256::repeat_byte(0x11),
                submitted_at: current_block,
                submitted_tx_index: 0,
                submitter: 1,
            },
        );
        crate::NextPendingExtrinsicIndex::<Test>::put(1);
    }

    #[test]
    fn due_ibe_head_blocks_plaintext_non_operational() {
        new_test_ext().execute_with(|| {
            seed_ibe_queue_head(10, 10);
            let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
            assert_eq!(
                validate_ext(Some(1), &call, TransactionSource::InBlock),
                Err(InvalidTransaction::ExhaustsResources.into())
            );
        });
    }

    #[test]
    fn future_ibe_head_does_not_block_plaintext() {
        new_test_ext().execute_with(|| {
            seed_ibe_queue_head(10, 11);
            let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
            assert!(validate_ext(Some(1), &call, TransactionSource::InBlock).is_ok());
        });
    }

    #[test]
    fn due_ibe_head_allows_encrypted_admission_and_drain_context() {
        new_test_ext().execute_with(|| {
            seed_ibe_queue_head(10, 10);
            let shield_call = make_submit_call([0xFF; 16]);
            assert!(validate_ext(Some(1), &shield_call, TransactionSource::InBlock).is_ok());

            crate::IbeQueueDrainInProgress::<Test>::put(true);
            let plain_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
            assert!(validate_ext(Some(1), &plain_call, TransactionSource::InBlock).is_ok());
        });
    }
}
