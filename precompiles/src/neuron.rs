use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, PrecompileHandle};
use precompile_utils::{EvmResult, prelude::UnboundedBytes};
use sp_core::H256;
use sp_runtime::traits::Dispatchable;
use sp_std::vec::Vec;

use crate::{PrecompileExt, PrecompileHandleExt};

pub struct NeuronPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for NeuronPrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2052;
}

#[precompile_utils::precompile]
impl<R> NeuronPrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    #[precompile::public("setWeights(uint16,uint16[],uint16[],uint64)")]
    #[precompile::payable]
    pub fn set_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        dests: Vec<u16>,
        weights: Vec<u16>,
        version_key: u64,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<R>::set_weights {
            netuid: netuid.into(),
            dests,
            weights,
            version_key,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("commitWeights(uint16,bytes32)")]
    #[precompile::payable]
    pub fn commit_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        commit_hash: H256,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<R>::commit_weights {
            netuid: netuid.into(),
            commit_hash,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("revealWeights(uint16,uint16[],uint16[],uint16[],uint64)")]
    #[precompile::payable]
    pub fn reveal_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        uids: Vec<u16>,
        values: Vec<u16>,
        salt: Vec<u16>,
        version_key: u64,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<R>::reveal_weights {
            netuid: netuid.into(),
            uids,
            values,
            salt,
            version_key,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("burnedRegister(uint16,bytes32)")]
    #[precompile::payable]
    fn burned_register(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        hotkey: H256,
    ) -> EvmResult<()> {
        let coldkey = handle.caller_account_id::<R>();

        let hotkey = R::AccountId::from(hotkey.0);
        let call = pallet_subtensor::Call::<R>::burned_register {
            netuid: netuid.into(),
            hotkey,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(coldkey))
    }

    #[precompile::public("serveAxon(uint16,uint32,uint128,uint16,uint8,uint8,uint8,uint8)")]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn serve_axon(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
        protocol: u8,
        placeholder1: u8,
        placeholder2: u8,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<R>::serve_axon {
            netuid: netuid.into(),
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public(
        "serveAxonTls(uint16,uint32,uint128,uint16,uint8,uint8,uint8,uint8,bytes)"
    )]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn serve_axon_tls(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
        protocol: u8,
        placeholder1: u8,
        placeholder2: u8,
        certificate: UnboundedBytes,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<R>::serve_axon_tls {
            netuid: netuid.into(),
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
            certificate: certificate.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("servePrometheus(uint16,uint32,uint128,uint16,uint8)")]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn serve_prometheus(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<R>::serve_prometheus {
            netuid: netuid.into(),
            version,
            ip,
            port,
            ip_type,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }
}
