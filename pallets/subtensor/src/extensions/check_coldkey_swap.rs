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

impl<T: Config + Send + Sync + TypeInfo> CheckColdkeySwap<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

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

        // Get the real account and origin if we are behind a proxy.
        let (who, call) = if let Some(
            ProxyCall::proxy { real, call, .. } | ProxyCall::proxy_announced { real, call, .. },
        ) = call.is_sub_type()
        {
            let real = LookupOf::<T>::lookup(real.clone())
                .map_err(|_| CustomTransactionError::InvalidRealAccount)?;
            (real, (*call.clone()).into())
        } else {
            (who.clone(), call.clone())
        };

        if ColdkeySwapAnnouncements::<T>::contains_key(&who) {
            if ColdkeySwapDisputes::<T>::contains_key(&who) {
                return Err(CustomTransactionError::ColdkeySwapDisputed.into());
            }

            let is_allowed_direct = matches!(
                call.is_sub_type(),
                Some(
                    Call::announce_coldkey_swap { .. }
                        | Call::swap_coldkey_announced { .. }
                        | Call::dispute_coldkey_swap { .. }
                )
            );

            let is_mev_protected = matches!(
                IsSubType::<pallet_shield::Call<T>>::is_sub_type(&call),
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

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{BalancesCall, DefaultMinStake, tests::mock::*};
    use frame_support::testing_prelude::*;
    use frame_support::{dispatch::GetDispatchInfo, traits::OriginTrait};
    use frame_system::Call as SystemCall;
    use sp_core::U256;
    use sp_runtime::{
        BoundedVec,
        traits::{AsTransactionAuthorizedOrigin, Hash, TxBaseImplication},
    };
    use subtensor_runtime_common::{Currency, NetUid};

    type HashingOf<T> = <T as frame_system::Config>::Hashing;

    const CALL: RuntimeCall = RuntimeCall::System(SystemCall::remark { remark: vec![] });

    #[test]
    fn skipped_for_non_signed_origins() {
        new_test_ext(1).execute_with(|| {
            let info = CALL.get_dispatch_info();
            let len = 0_usize;

            let (_, _, origin) = CheckColdkeySwap::<Test>::new()
                .validate(
                    None.into(),
                    &CALL,
                    &info,
                    len,
                    (),
                    &TxBaseImplication(CALL),
                    TransactionSource::External,
                )
                .unwrap();
            assert!(!origin.is_transaction_authorized());

            let (_, _, origin) = CheckColdkeySwap::<Test>::new()
                .validate(
                    RuntimeOrigin::root().into(),
                    &CALL,
                    &info,
                    len,
                    (),
                    &TxBaseImplication(CALL),
                    TransactionSource::External,
                )
                .unwrap();
            assert!(origin.as_system_ref().unwrap().is_root());
        })
    }

    #[test]
    fn skipped_if_no_active_swap() {
        new_test_ext(1).execute_with(|| {
            let who = U256::from(1);
            let info = CALL.get_dispatch_info();
            let len = 0_usize;

            let (_, _, origin) = CheckColdkeySwap::<Test>::new()
                .validate(
                    RuntimeOrigin::signed(who).into(),
                    &CALL,
                    &info,
                    len,
                    (),
                    &TxBaseImplication(CALL),
                    TransactionSource::External,
                )
                .unwrap();
            assert_eq!(origin.as_signer(), Some(&who));
        })
    }

    #[test]
    fn validate_calls_correctly() {
        new_test_ext(1).execute_with(|| {
            let netuid = NetUid::from(1);
            let stake = DefaultMinStake::<Test>::get().to_u64();
            let who = U256::from(1);
            let now = System::block_number();
            let another_coldkey = U256::from(3);
            let another_coldkey_hash = HashingOf::<Test>::hash_of(&another_coldkey);
            let new_coldkey = U256::from(42);
            let new_coldkey_hash = HashingOf::<Test>::hash_of(&new_coldkey);
            ColdkeySwapAnnouncements::<Test>::insert(who, (now, new_coldkey_hash));

            let reserve = stake * 10;
            setup_reserves(netuid, reserve.into(), reserve.into());

            // Setup network and neuron
            let hotkey = U256::from(2);
            add_network(netuid, 1, 0);
            register_ok_neuron(netuid, hotkey, who, 0);

            SubtensorModule::add_balance_to_coldkey_account(&who, u64::MAX);

            let forbidden_calls: Vec<RuntimeCall> = vec![
                RuntimeCall::SubtensorModule(SubtensorCall::dissolve_network {
                    netuid,
                    coldkey: who,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
                    hotkey,
                    netuid,
                    amount_staked: stake.into(),
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::add_stake_limit {
                    hotkey,
                    netuid,
                    amount_staked: stake.into(),
                    limit_price: stake.into(),
                    allow_partial: false,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::swap_stake {
                    hotkey,
                    origin_netuid: netuid,
                    destination_netuid: netuid,
                    alpha_amount: stake.into(),
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::swap_stake_limit {
                    hotkey,
                    origin_netuid: netuid,
                    destination_netuid: netuid,
                    alpha_amount: stake.into(),
                    limit_price: stake.into(),
                    allow_partial: false,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::move_stake {
                    origin_hotkey: hotkey,
                    destination_hotkey: hotkey,
                    origin_netuid: netuid,
                    destination_netuid: netuid,
                    alpha_amount: stake.into(),
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::transfer_stake {
                    destination_coldkey: new_coldkey,
                    hotkey,
                    origin_netuid: netuid,
                    destination_netuid: netuid,
                    alpha_amount: stake.into(),
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
                    hotkey,
                    netuid,
                    amount_unstaked: (DefaultMinStake::<Test>::get().to_u64() * 2).into(),
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::remove_stake_limit {
                    hotkey,
                    netuid,
                    amount_unstaked: (stake * 2).into(),
                    limit_price: 123456789.into(),
                    allow_partial: true,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::burned_register { netuid, hotkey }),
                RuntimeCall::Balances(BalancesCall::transfer_all {
                    dest: new_coldkey,
                    keep_alive: false,
                }),
                RuntimeCall::Balances(BalancesCall::transfer_keep_alive {
                    dest: new_coldkey,
                    value: 100_000_000_000,
                }),
                RuntimeCall::Balances(BalancesCall::transfer_allow_death {
                    dest: new_coldkey,
                    value: 100_000_000_000,
                }),
            ];

            // Forbidden calls through direct origin
            for call in &forbidden_calls {
                assert_eq!(
                    ext_validate(who, call.clone()).unwrap_err(),
                    CustomTransactionError::ColdkeySwapAnnounced.into()
                );
            }

            let delegate = U256::from(2);

            // Forbidden calls through proxy
            for call in &forbidden_calls {
                let proxy_calls = build_proxy_calls(who, delegate, call.clone());
                for proxy_call in proxy_calls {
                    assert_eq!(
                        ext_validate(delegate, proxy_call.clone()).unwrap_err(),
                        CustomTransactionError::ColdkeySwapAnnounced.into()
                    );
                }
            }

            let authorized_calls: Vec<RuntimeCall> = vec![
                RuntimeCall::SubtensorModule(SubtensorCall::announce_coldkey_swap {
                    new_coldkey_hash: another_coldkey_hash,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::swap_coldkey_announced { new_coldkey }),
                RuntimeCall::SubtensorModule(SubtensorCall::dispute_coldkey_swap {}),
                RuntimeCall::Shield(pallet_shield::Call::submit_encrypted {
                    commitment: <Test as frame_system::Config>::Hashing::hash_of(&new_coldkey),
                    ciphertext: BoundedVec::truncate_from(vec![1, 2, 3, 4]),
                }),
            ];

            // Authorized calls through direct origin
            for call in &authorized_calls {
                let (_, _, origin) = ext_validate(who, call.clone()).unwrap();
                assert_eq!(origin.as_signer(), Some(&who));
            }

            // Authorized calls through proxy
            for call in &authorized_calls {
                let proxy_calls = build_proxy_calls(who, delegate, call.clone());
                for proxy_call in proxy_calls {
                    let (_, _, origin) = ext_validate(delegate, proxy_call.clone()).unwrap();
                    assert_eq!(origin.as_signer(), Some(&delegate));
                }
            }

            ColdkeySwapDisputes::<Test>::insert(who, now);

            // All calls should fail when the coldkey swap is disputed
            let all_calls = forbidden_calls.iter().chain(authorized_calls.iter());

            // All calls through direct origin during dispute
            for call in all_calls.clone() {
                assert_eq!(
                    ext_validate(who, call.clone()).unwrap_err(),
                    CustomTransactionError::ColdkeySwapDisputed.into()
                );
            }

            // All calls through proxy during dispute
            for call in all_calls {
                let proxy_calls = build_proxy_calls(who, delegate, call.clone());
                for proxy_call in proxy_calls {
                    assert_eq!(
                        ext_validate(delegate, proxy_call.clone()).unwrap_err(),
                        CustomTransactionError::ColdkeySwapDisputed.into()
                    );
                }
            }
        })
    }

    fn build_proxy_calls(who: U256, delegate: U256, call: RuntimeCall) -> Vec<RuntimeCall> {
        vec![
            RuntimeCall::Proxy(ProxyCall::proxy {
                real: who,
                force_proxy_type: None,
                call: Box::new(call.clone()),
            }),
            RuntimeCall::Proxy(ProxyCall::proxy_announced {
                delegate,
                real: who,
                force_proxy_type: None,
                call: Box::new(call.clone()),
            }),
        ]
    }

    fn ext_validate(who: U256, call: RuntimeCall) -> ValidateResult<(), RuntimeCall> {
        let info = call.get_dispatch_info();
        let len = 0_usize;

        CheckColdkeySwap::<Test>::new().validate(
            RuntimeOrigin::signed(who).into(),
            &call.clone(),
            &info,
            len,
            (),
            &TxBaseImplication(call),
            TransactionSource::External,
        )
    }
}
