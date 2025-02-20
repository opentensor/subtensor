extern crate alloc;

use alloc::format;
use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, Pays};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, BalanceConverter, ExitError, GasWeightMapping, HashedAddressMapping,
    IsPrecompileResult, Precompile, PrecompileFailure, PrecompileHandle, PrecompileResult,
    PrecompileSet,
};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use precompile_utils::EvmResult;
use sp_core::{H160, U256};
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::{traits::Dispatchable, AccountId32};

use pallet_admin_utils::{PrecompileEnable, PrecompileEnum};
use sp_std::vec;

use crate::{Runtime, RuntimeCall};

// Include custom precompiles
mod balance_transfer;
mod ed25519;
mod metagraph;
mod neuron;
mod staking;
mod subnet;

use balance_transfer::*;
use ed25519::*;
use metagraph::*;
use neuron::*;
use staking::*;
use subnet::*;

pub struct FrontierPrecompiles<R>(PhantomData<R>);
impl<R> Default for FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R> FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
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
            hash(BalanceTransferPrecompile::INDEX),
            hash(StakingPrecompile::INDEX),
            hash(SubnetPrecompile::INDEX),
            hash(MetagraphPrecompile::INDEX),
            hash(NeuronPrecompile::INDEX),
        ]
    }
}
impl<R> PrecompileSet for FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
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
            a if a == hash(BalanceTransferPrecompile::INDEX) => {
                if PrecompileEnable::<Runtime>::get(PrecompileEnum::BalanceTransfer) {
                    Some(BalanceTransferPrecompile::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other(
                            "Precompile Balance Transfer is disabled".into(),
                        ),
                    }))
                }
            }
            a if a == hash(StakingPrecompile::INDEX) => {
                if PrecompileEnable::<Runtime>::get(PrecompileEnum::Staking) {
                    Some(StakingPrecompile::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other(
                            "Precompile Balance Transfer is disabled".into(),
                        ),
                    }))
                }
            }

            a if a == hash(SubnetPrecompile::INDEX) => {
                if PrecompileEnable::<Runtime>::get(PrecompileEnum::Subnet) {
                    Some(SubnetPrecompile::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other("Precompile Subnet is disabled".into()),
                    }))
                }
            }
            a if a == hash(MetagraphPrecompile::INDEX) => {
                if PrecompileEnable::<Runtime>::get(PrecompileEnum::Metagraph) {
                    Some(MetagraphPrecompile::execute(handle))
                } else {
                    Some(Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other("Precompile Metagrah is disabled".into()),
                    }))
                }
            }
            a if a == hash(NeuronPrecompile::INDEX) => {
                if PrecompileEnable::<Runtime>::get(PrecompileEnum::Neuron) {
                    Some(NeuronPrecompile::execute(handle))
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

fn parse_pubkey(data: &[u8]) -> Result<(AccountId32, vec::Vec<u8>), PrecompileFailure> {
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(parse_slice(data, 0, 32)?);

    Ok((
        pubkey.into(),
        data.get(32..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()),
    ))
}

fn try_u16_from_u256(value: U256) -> Result<u16, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("the value is outside of u16 bounds".into()),
    })
}

fn contract_to_origin(contract: &[u8; 32]) -> Result<RawOrigin<AccountId32>, PrecompileFailure> {
    let (account_id, _) = parse_pubkey(contract)?;
    Ok(RawOrigin::Signed(account_id))
}

trait PrecompileHandleExt: PrecompileHandle {
    fn caller_account_id(&self) -> AccountId32 {
        <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
            self.context().caller,
        )
    }

    fn try_convert_apparent_value(&self) -> EvmResult<U256> {
        let amount = self.context().apparent_value;
        <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount).ok_or(
            PrecompileFailure::Error {
                exit_status: ExitError::Other(
                    "error converting balance from ETH to subtensor".into(),
                ),
            },
        )
    }

    /// Dispatches a runtime call, but also checks and records the gas costs.
    fn try_dispatch_runtime_call(
        &mut self,
        call: impl Into<RuntimeCall>,
        origin: RawOrigin<AccountId32>,
    ) -> EvmResult<()> {
        let call = Into::<RuntimeCall>::into(call);
        let info = call.get_dispatch_info();

        let target_gas = self.gas_limit();
        if let Some(gas) = target_gas {
            let valid_weight =
                <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(gas, false)
                    .ref_time();
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

        match call.dispatch(origin.into()) {
            Ok(post_info) => {
                if post_info.pays_fee(&info) == Pays::Yes {
                    let actual_weight = post_info.actual_weight.unwrap_or(info.weight);
                    let cost = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
                        actual_weight,
                    );
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
