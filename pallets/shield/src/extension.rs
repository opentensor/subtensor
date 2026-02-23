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
use sp_runtime::transaction_validity::{TransactionSource, ValidTransaction};
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
        let Some(ShieldedTransaction { key_hash, .. }) = ShieldedTransaction::parse(ciphertext)
        else {
            return Err(CustomTransactionError::FailedShieldedTxParsing.into());
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
                return Err(CustomTransactionError::InvalidShieldedTxPubKeyHash.into());
            }
        }

        // Shielded txs get a short longevity so they are evicted from the pool
        // if not included within a few blocks. Keys rotate every block, so a tx
        // encrypted against a key that has rotated out of the 2-key window
        // (CurrentKey + NextKey) will never be included — without this it would
        // stay in the pool indefinitely since pool revalidation skips the key check.
        let validity = ValidTransaction {
            longevity: 3,
            ..Default::default()
        };

        Ok((validity, (), origin))
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

    fn set_current_key(pk: &[u8]) {
        CurrentKey::<Test>::put(BoundedVec::<u8, ConstU32<2048>>::truncate_from(pk.to_vec()));
    }

    fn set_next_key(pk: &[u8]) {
        NextKey::<Test>::put(BoundedVec::<u8, ConstU32<2048>>::truncate_from(pk.to_vec()));
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

    const PK_A: [u8; 32] = [0x11; 32];
    const PK_B: [u8; 32] = [0x22; 32];

    #[test]
    fn non_shield_call_passes_through() {
        new_test_ext().execute_with(|| {
            let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
            let validity = validate_ext(Some(1), &call, TransactionSource::InBlock).unwrap();
            // Non-shield calls get default (max) longevity.
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
    fn inblock_matches_current_key() {
        new_test_ext().execute_with(|| {
            set_current_key(&PK_A);
            let call = make_submit_call(twox_128(&PK_A));
            let validity = validate_ext(Some(1), &call, TransactionSource::InBlock).unwrap();
            assert_eq!(validity.longevity, 3);
        });
    }

    #[test]
    fn inblock_matches_next_key() {
        new_test_ext().execute_with(|| {
            set_next_key(&PK_B);
            let call = make_submit_call(twox_128(&PK_B));
            let validity = validate_ext(Some(1), &call, TransactionSource::InBlock).unwrap();
            assert_eq!(validity.longevity, 3);
        });
    }

    #[test]
    fn inblock_no_match_rejected() {
        new_test_ext().execute_with(|| {
            set_current_key(&PK_A);
            set_next_key(&PK_B);
            let call = make_submit_call([0xFF; 16]);
            assert_eq!(
                validate_ext(Some(1), &call, TransactionSource::InBlock),
                Err(CustomTransactionError::InvalidShieldedTxPubKeyHash.into())
            );
        });
    }

    #[test]
    fn inblock_no_keys_set_rejected() {
        new_test_ext().execute_with(|| {
            let call = make_submit_call(twox_128(&PK_A));
            assert_eq!(
                validate_ext(Some(1), &call, TransactionSource::InBlock),
                Err(CustomTransactionError::InvalidShieldedTxPubKeyHash.into())
            );
        });
    }

    #[test]
    fn pool_local_skips_key_check() {
        new_test_ext().execute_with(|| {
            let call = make_submit_call([0xFF; 16]);
            let validity = validate_ext(Some(1), &call, TransactionSource::Local).unwrap();
            // Pool sources skip key check but still get short longevity.
            assert_eq!(validity.longevity, 3);
        });
    }

    #[test]
    fn pool_external_skips_key_check() {
        new_test_ext().execute_with(|| {
            let call = make_submit_call([0xFF; 16]);
            let validity = validate_ext(Some(1), &call, TransactionSource::External).unwrap();
            assert_eq!(validity.longevity, 3);
        });
    }
}
