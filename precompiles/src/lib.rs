#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::format;
use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, Pays, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, BalanceConverter, ExitError, GasWeightMapping, IsPrecompileResult, Precompile,
    PrecompileFailure, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use precompile_utils::EvmResult;
use sp_core::{H160, U256, crypto::ByteArray};
use sp_runtime::traits::Dispatchable;
use sp_runtime::traits::StaticLookup;
use subtensor_runtime_common::ProxyType;

use pallet_admin_utils::{PrecompileEnable, PrecompileEnum};
use sp_std::vec;

use crate::balance_transfer::*;
use crate::ed25519::*;
use crate::metagraph::*;
use crate::neuron::*;
use crate::staking::*;
use crate::subnet::*;

mod balance_transfer;
mod ed25519;
mod metagraph;
mod neuron;
mod staking;
mod subnet;

pub struct Precompiles<R>(PhantomData<R>);

impl<R> Default for Precompiles<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_balances::Config
        + pallet_admin_utils::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
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
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn used_addresses() -> [H160; 13] {
        [
            hash(1),
            hash(2),
            hash(3),
            hash(4),
            hash(5),
            hash(1024),
            hash(1025),
            hash(Ed25519Verify::INDEX),
            hash(BalanceTransferPrecompile::<R>::INDEX),
            hash(StakingPrecompile::<R>::INDEX),
            hash(SubnetPrecompile::<R>::INDEX),
            hash(MetagraphPrecompile::<R>::INDEX),
            hash(NeuronPrecompile::<R>::INDEX),
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
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
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
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            a if a == hash(Ed25519Verify::INDEX) => Some(Ed25519Verify::execute(handle)),
            // Subtensor specific precompiles :
            a if a == hash(BalanceTransferPrecompile::<R>::INDEX) => {
                if PrecompileEnable::<R>::get(PrecompileEnum::BalanceTransfer) {
                    Some(BalanceTransferPrecompile::<R>::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other(
                            "Precompile Balance Transfer is disabled".into(),
                        ),
                    }))
                }
            }
            a if a == hash(StakingPrecompile::<R>::INDEX) => {
                if PrecompileEnable::<R>::get(PrecompileEnum::Staking) {
                    Some(StakingPrecompile::<R>::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other(
                            "Precompile Balance Transfer is disabled".into(),
                        ),
                    }))
                }
            }

            a if a == hash(SubnetPrecompile::<R>::INDEX) => {
                if PrecompileEnable::<R>::get(PrecompileEnum::Subnet) {
                    Some(SubnetPrecompile::<R>::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other("Precompile Subnet is disabled".into()),
                    }))
                }
            }
            a if a == hash(MetagraphPrecompile::<R>::INDEX) => {
                if PrecompileEnable::<R>::get(PrecompileEnum::Metagraph) {
                    Some(MetagraphPrecompile::<R>::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other("Precompile Metagrah is disabled".into()),
                    }))
                }
            }
            a if a == hash(NeuronPrecompile::<R>::INDEX) => {
                if PrecompileEnable::<R>::get(PrecompileEnum::Neuron) {
                    Some(NeuronPrecompile::<R>::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other("Precompile Neuron is disabled".into()),
                    }))
                }
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

/// Takes a slice from bytes with PrecompileFailure as Error
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

fn parse_pubkey<A: From<[u8; 32]>>(data: &[u8]) -> Result<(A, vec::Vec<u8>), PrecompileFailure> {
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(parse_slice(data, 0, 32)?);

    Ok((
        pubkey.into(),
        data.get(32..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()),
    ))
}

fn contract_to_origin<A: From<[u8; 32]>>(
    contract: &[u8; 32],
) -> Result<RawOrigin<A>, PrecompileFailure> {
    let (account_id, _) = parse_pubkey::<A>(contract)?;
    Ok(RawOrigin::Signed(account_id))
}

fn try_u16_from_u256(value: U256) -> Result<u16, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("the value is outside of u16 bounds".into()),
    })
}

trait PrecompileHandleExt: PrecompileHandle {
    fn caller_account_id<R>(&self) -> R::AccountId
    where
        R: frame_system::Config + pallet_evm::Config,
        <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    {
        <R as pallet_evm::Config>::AddressMapping::into_account_id(self.context().caller)
    }

    fn try_convert_apparent_value<R>(&self) -> EvmResult<U256>
    where
        R: pallet_evm::Config,
    {
        let amount = self.context().apparent_value;
        <R as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount).ok_or(
            PrecompileFailure::Error {
                exit_status: ExitError::Other(
                    "error converting balance from ETH to subtensor".into(),
                ),
            },
        )
    }

    /// Dispatches a runtime call, but also checks and records the gas costs.
    fn try_dispatch_runtime_call<R, Call>(
        &mut self,
        call: Call,
        origin: RawOrigin<R::AccountId>,
    ) -> EvmResult<()>
    where
        R: frame_system::Config + pallet_evm::Config,
        R::RuntimeCall: From<Call>,
        R::RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
        R::RuntimeOrigin: From<RawOrigin<R::AccountId>>,
    {
        let call = R::RuntimeCall::from(call);
        let info = GetDispatchInfo::get_dispatch_info(&call);

        let target_gas = self.gas_limit();
        if let Some(gas) = target_gas {
            let valid_weight =
                <R as pallet_evm::Config>::GasWeightMapping::gas_to_weight(gas, false).ref_time();
            if info.weight.ref_time() > valid_weight {
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::OutOfGas,
                });
            }
        }

        self.record_external_cost(
            Some(info.weight.ref_time()),
            Some(info.weight.proof_size()),
            None,
        )?;

        match call.dispatch(R::RuntimeOrigin::from(origin)) {
            Ok(post_info) => {
                if post_info.pays_fee(&info) == Pays::Yes {
                    let actual_weight = post_info.actual_weight.unwrap_or(info.weight);
                    let cost =
                        <R as pallet_evm::Config>::GasWeightMapping::weight_to_gas(actual_weight);
                    self.record_cost(cost)?;

                    self.refund_external_cost(
                        Some(
                            info.weight
                                .ref_time()
                                .saturating_sub(actual_weight.ref_time()),
                        ),
                        Some(
                            info.weight
                                .proof_size()
                                .saturating_sub(actual_weight.proof_size()),
                        ),
                    );
                }

                log::info!("Dispatch succeeded. Post info: {:?}", post_info);

                Ok(())
            }
            Err(e) => {
                log::error!("Dispatch failed. Error: {:?}", e);
                log::warn!("Returning error PrecompileFailure::Error");
                Err(PrecompileFailure::Error {
                    exit_status: ExitError::Other(
                        format!("dispatch execution failed: {}", <&'static str>::from(e)).into(),
                    ),
                })
            }
        }
    }
}

impl<T> PrecompileHandleExt for T where T: PrecompileHandle {}

trait PrecompileExt: Precompile {
    const INDEX: u64;
    // ss58 public key i.e., the contract sends funds it received to the destination address from
    // the method parameter.
    const ADDRESS_SS58: [u8; 32];
}
