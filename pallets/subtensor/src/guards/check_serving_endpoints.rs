use crate::{Call, Config, Error, Pallet};
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchExtension, DispatchInfo, PostDispatchInfo},
    pallet_prelude::*,
    traits::{IsSubType, OriginTrait},
};
use sp_runtime::traits::Dispatchable;
use sp_std::marker::PhantomData;

type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
type DispatchableOriginOf<T> = <CallOf<T> as Dispatchable>::RuntimeOrigin;

/// Dispatch extension for axon/prometheus endpoint validation.
///
/// Signed serving calls are checked before dispatch; unrelated calls and
/// non-signed origins pass through.
pub struct CheckServingEndpoints<T: Config>(PhantomData<T>);

impl<T: Config> CheckServingEndpoints<T> {
    pub fn check(who: &T::AccountId, call: &Call<T>) -> Result<(), Error<T>> {
        match call {
            Call::serve_axon {
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2,
            }
            | Call::serve_axon_tls {
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2,
                ..
            } => Pallet::<T>::validate_serve_axon(
                who,
                *netuid,
                *version,
                *ip,
                *port,
                *ip_type,
                *protocol,
                *placeholder1,
                *placeholder2,
            ),
            Call::serve_prometheus {
                netuid,
                version,
                ip,
                port,
                ip_type,
            } => {
                Pallet::<T>::validate_serve_prometheus(who, *netuid, *version, *ip, *port, *ip_type)
                    .map(|_| ())
            }
            _ => Ok(()),
        }
    }
}

impl<T> DispatchExtension<CallOf<T>> for CheckServingEndpoints<T>
where
    T: Config,
    CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
    DispatchableOriginOf<T>: OriginTrait<AccountId = T::AccountId>,
{
    type Pre = ();

    fn weight(_call: &CallOf<T>) -> Weight {
        T::DbWeight::get().reads(4)
    }

    fn pre_dispatch(
        origin: &DispatchableOriginOf<T>,
        call: &CallOf<T>,
    ) -> Result<Self::Pre, DispatchErrorWithPostInfo> {
        let Some(who) = origin.as_signer() else {
            return Ok(());
        };

        let Some(call) = call.is_sub_type() else {
            return Ok(());
        };

        Self::check(who, call).map_err(Into::into)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::CheckServingEndpoints;
    use crate::{Error, tests::mock::*};
    use frame_support::{
        assert_ok, dispatch::DispatchResultWithPostInfo, traits::ExtendedDispatchable,
    };
    use frame_system::Call as SystemCall;
    use sp_core::U256;
    use sp_runtime::DispatchError;
    use subtensor_runtime_common::NetUid;

    fn dispatch_with_ext(call: RuntimeCall, origin: RuntimeOrigin) -> DispatchResultWithPostInfo {
        <CheckServingEndpoints<Test> as ExtendedDispatchable<RuntimeCall>>::dispatch_with_extension(
            origin, call,
        )
    }

    fn err(result: DispatchResultWithPostInfo) -> DispatchError {
        result.err().unwrap().error
    }

    fn serve_axon_call(netuid: NetUid) -> RuntimeCall {
        RuntimeCall::SubtensorModule(SubtensorCall::serve_axon {
            netuid,
            version: 1,
            ip: u128::from(u32::from_be_bytes([8, 8, 8, 8])),
            port: 1,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
        })
    }

    fn serve_prometheus_call(netuid: NetUid) -> RuntimeCall {
        RuntimeCall::SubtensorModule(SubtensorCall::serve_prometheus {
            netuid,
            version: 1,
            ip: u128::from(u32::from_be_bytes([8, 8, 4, 4])),
            port: 1,
            ip_type: 4,
        })
    }

    fn register_hotkey(netuid: NetUid, hotkey: U256, coldkey: U256) {
        add_network(netuid, 1, 0);
        setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        register_ok_neuron(netuid, hotkey, coldkey, 0);
    }

    #[test]
    fn unrelated_calls_pass_through() {
        new_test_ext(0).execute_with(|| {
            let call = RuntimeCall::System(SystemCall::remark { remark: vec![] });

            assert_ok!(dispatch_with_ext(
                call,
                RuntimeOrigin::signed(U256::from(1))
            ));
        });
    }

    #[test]
    fn unregistered_hotkey_blocks_axon() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);

            assert_eq!(
                err(dispatch_with_ext(
                    serve_axon_call(netuid),
                    RuntimeOrigin::signed(hotkey)
                )),
                Error::<Test>::HotKeyNotRegisteredInNetwork.into()
            );
        });
    }

    #[test]
    fn registered_hotkey_allows_axon() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            register_hotkey(netuid, hotkey, U256::from(2));

            assert_ok!(dispatch_with_ext(
                serve_axon_call(netuid),
                RuntimeOrigin::signed(hotkey)
            ));
        });
    }

    #[test]
    fn registered_hotkey_allows_prometheus() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            register_hotkey(netuid, hotkey, U256::from(2));

            assert_ok!(dispatch_with_ext(
                serve_prometheus_call(netuid),
                RuntimeOrigin::signed(hotkey)
            ));
        });
    }
}
