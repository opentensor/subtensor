use core::marker::PhantomData;

use crate::{PrecompileExt, PrecompileHandleExt};
use codec::DecodeLimit;
use fp_evm::{ExitError, PrecompileFailure};
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, PrecompileHandle};
use precompile_utils::EvmResult;
use sp_core::keccak_256;
use sp_core::{H160, H256};
use sp_runtime::traits::Dispatchable;
use sp_std::vec::Vec;

pub struct PureProxyPrecompile<R>(PhantomData<R>);
const MAX_DECODE_DEPTH: u32 = 8;
impl<R> PureProxyPrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
{
    fn into_pure_proxy_account_id(address: &H160) -> R::AccountId {
        let mut data = [0u8; 30];
        data[0..10].copy_from_slice(b"pureproxy:");
        data[10..30].copy_from_slice(&address[..]);
        let hash = keccak_256(&data);

        R::AccountId::from(Into::<[u8; 32]>::into(hash))
    }
}

impl<R> PrecompileExt<R::AccountId> for PureProxyPrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2057;
}

#[precompile_utils::precompile]
impl<R> PureProxyPrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    #[precompile::public("createPureProxy()")]
    #[precompile::payable]
    pub fn create_pure_proxy(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
        let caller = handle.context().caller;
        if pallet_subtensor::PureProxyAccount::<R>::get(caller).is_none() {
            let account = Self::into_pure_proxy_account_id(&caller);
            pallet_subtensor::PureProxyAccount::<R>::insert(caller, account.clone());
            let buf: [u8; 32] = account.into();

            Ok(H256::from(buf))
        } else {
            Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Pure proxy account not created yet".into()),
            })
        }
    }

    #[precompile::public("pureProxyCall(uint8[])")]
    #[precompile::payable]
    pub fn pure_proxy_call(handle: &mut impl PrecompileHandle, call: Vec<u8>) -> EvmResult<()> {
        let caller = handle.context().caller;
        match pallet_subtensor::PureProxyAccount::<R>::get(caller) {
            Some(account) => {
                let call = <R as frame_system::Config>::RuntimeCall::decode_with_depth_limit(
                    MAX_DECODE_DEPTH,
                    &mut &call[..],
                )
                .map_err(|_| PrecompileFailure::Error {
                    exit_status: ExitError::Other("The raw call data not correctly encoded".into()),
                })?;

                handle.try_dispatch_runtime_call::<R, <R as frame_system::Config>::RuntimeCall>(
                    call,
                    RawOrigin::Signed(account),
                )
            }
            None => Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Pure proxy account not created yet".into()),
            }),
        }
    }

    #[precompile::public("getPureProxy()")]
    #[precompile::view]
    fn get_pure_proxy(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
        let caller = handle.context().caller;
        let result = pallet_subtensor::PureProxyAccount::<R>::get(caller);
        let buf: [u8; 32] = result.map(|account| account.into()).unwrap_or([0_u8; 32]);

        Ok(H256::from(buf))
    }
}
