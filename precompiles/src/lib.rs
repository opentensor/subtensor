#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::marker::PhantomData;

use fp_evm::{ExitError, PrecompileFailure};
use frame_support::traits::IsSubType;
use frame_support::{
    dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo},
    pallet_prelude::Decode,
};
use pallet_evm::{
    AddressMapping, IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult,
    PrecompileSet,
};
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_subtensor_proxy as pallet_proxy;
use sp_core::{H160, U256, crypto::ByteArray};
use sp_runtime::traits::{AsSystemOriginSigner, Dispatchable, StaticLookup};
use subtensor_runtime_common::ProxyType;

use pallet_admin_utils::PrecompileEnum;

use crate::alpha::*;
use crate::balance_transfer::*;
use crate::crowdloan::*;
use crate::ed25519::*;
use crate::extensions::*;
use crate::leasing::*;
use crate::metagraph::*;
use crate::neuron::*;
use crate::proxy::*;
use crate::sr25519::*;
use crate::staking::*;
use crate::storage_query::*;
use crate::subnet::*;
use crate::uid_lookup::*;

mod alpha;
mod balance_transfer;
mod crowdloan;
mod ed25519;
mod extensions;
mod leasing;
mod metagraph;
mod neuron;
mod proxy;
mod sr25519;
mod staking;
mod storage_query;
mod subnet;
mod uid_lookup;

pub struct Precompiles<R>(PhantomData<R>);

impl<R> Default for Precompiles<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_balances::Config
        + pallet_admin_utils::Config
        + pallet_subtensor::Config
        + pallet_subtensor_swap::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_crowdloan::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R> Precompiles<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_balances::Config
        + pallet_admin_utils::Config
        + pallet_subtensor::Config
        + pallet_subtensor_swap::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_crowdloan::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn used_addresses() -> [H160; 25] {
        [
            hash(1),
            hash(2),
            hash(3),
            hash(4),
            hash(5),
            hash(6),
            hash(7),
            hash(8),
            hash(9),
            hash(1024),
            hash(1025),
            hash(Ed25519Verify::<R::AccountId>::INDEX),
            hash(Sr25519Verify::<R::AccountId>::INDEX),
            hash(BalanceTransferPrecompile::<R>::INDEX),
            hash(StakingPrecompile::<R>::INDEX),
            hash(SubnetPrecompile::<R>::INDEX),
            hash(MetagraphPrecompile::<R>::INDEX),
            hash(NeuronPrecompile::<R>::INDEX),
            hash(StakingPrecompileV2::<R>::INDEX),
            hash(StorageQueryPrecompile::<R>::INDEX),
            hash(UidLookupPrecompile::<R>::INDEX),
            hash(AlphaPrecompile::<R>::INDEX),
            hash(CrowdloanPrecompile::<R>::INDEX),
            hash(LeasingPrecompile::<R>::INDEX),
            hash(ProxyPrecompile::<R>::INDEX),
        ]
    }
}
impl<R> PrecompileSet for Precompiles<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_balances::Config
        + pallet_admin_utils::Config
        + pallet_subtensor::Config
        + pallet_subtensor_swap::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_crowdloan::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + Decode,
    <<R as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
        From<Option<pallet_evm::AccountIdOf<R>>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(handle)),
            a if a == hash(2) => Some(Sha256::execute(handle)),
            a if a == hash(3) => Some(Ripemd160::execute(handle)),
            a if a == hash(4) => Some(Identity::execute(handle)),
            a if a == hash(5) => Some(Modexp::execute(handle)),
            a if a == hash(6) => Some(Dispatch::<R>::execute(handle)),
            a if a == hash(7) => Some(Bn128Mul::execute(handle)),
            a if a == hash(8) => Some(Bn128Pairing::execute(handle)),
            a if a == hash(9) => Some(Bn128Add::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => Some(Sha3FIPS256::<R, ()>::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            a if a == hash(Ed25519Verify::<R::AccountId>::INDEX) => {
                Some(Ed25519Verify::<R::AccountId>::execute(handle))
            }
            a if a == hash(Sr25519Verify::<R::AccountId>::INDEX) => {
                Some(Sr25519Verify::<R::AccountId>::execute(handle))
            }
            // Subtensor specific precompiles :
            a if a == hash(BalanceTransferPrecompile::<R>::INDEX) => {
                BalanceTransferPrecompile::<R>::try_execute::<R>(
                    handle,
                    PrecompileEnum::BalanceTransfer,
                )
            }
            a if a == hash(StakingPrecompile::<R>::INDEX) => {
                StakingPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Staking)
            }
            a if a == hash(StakingPrecompileV2::<R>::INDEX) => {
                StakingPrecompileV2::<R>::try_execute::<R>(handle, PrecompileEnum::Staking)
            }
            a if a == hash(SubnetPrecompile::<R>::INDEX) => {
                SubnetPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Subnet)
            }
            a if a == hash(MetagraphPrecompile::<R>::INDEX) => {
                MetagraphPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Metagraph)
            }
            a if a == hash(NeuronPrecompile::<R>::INDEX) => {
                NeuronPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Neuron)
            }
            a if a == hash(UidLookupPrecompile::<R>::INDEX) => {
                UidLookupPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::UidLookup)
            }
            a if a == hash(StorageQueryPrecompile::<R>::INDEX) => {
                Some(StorageQueryPrecompile::<R>::execute(handle))
            }
            a if a == hash(AlphaPrecompile::<R>::INDEX) => {
                AlphaPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Alpha)
            }
            a if a == hash(CrowdloanPrecompile::<R>::INDEX) => {
                CrowdloanPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Crowdloan)
            }
            a if a == hash(LeasingPrecompile::<R>::INDEX) => {
                LeasingPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Leasing)
            }
            a if a == hash(ProxyPrecompile::<R>::INDEX) => {
                ProxyPrecompile::<R>::try_execute::<R>(handle, PrecompileEnum::Proxy)
            }
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
        IsPrecompileResult::Answer {
            is_precompile: Self::used_addresses().contains(&address),
            extra_cost: 0,
        }
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}

/*
 *
 * This is used to parse a slice from bytes with PrecompileFailure as Error
 *
 */
fn parse_slice(data: &[u8], from: usize, to: usize) -> Result<&[u8], PrecompileFailure> {
    let maybe_slice = data.get(from..to);
    if let Some(slice) = maybe_slice {
        Ok(slice)
    } else {
        log::error!(
            "fail to get slice from data, {:?}, from {}, to {}",
            &data,
            from,
            to
        );
        Err(PrecompileFailure::Error {
            exit_status: ExitError::InvalidRange,
        })
    }
}
