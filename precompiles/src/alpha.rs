use core::marker::PhantomData;

use fp_evm::{ExitError, PrecompileFailure};
use pallet_evm::PrecompileHandle;
use precompile_utils::EvmResult;
use sp_core::U256;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::NetUid;
use subtensor_swap_interface::{OrderType, SwapHandler};

use crate::PrecompileExt;

pub struct AlphaPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for AlphaPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + pallet_subtensor_swap::Config,
    R::AccountId: From<[u8; 32]>,
{
    const INDEX: u64 = 2056;
}

#[precompile_utils::precompile]
impl<R> AlphaPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + pallet_subtensor_swap::Config,
{
    #[precompile::public("getAlphaPrice(uint16)")]
    #[precompile::view]
    fn get_alpha_price(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        let price =
            <pallet_subtensor_swap::Pallet<R> as SwapHandler<R::AccountId>>::current_alpha_price(
                netuid.into(),
            );
        Ok(U256::from(price.saturating_to_num::<u64>()))
    }

    #[precompile::public("getMovingAlphaPrice(uint16)")]
    #[precompile::view]
    fn get_moving_alpha_price(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        let price: U96F32 = pallet_subtensor::Pallet::<R>::get_moving_alpha_price(netuid.into());
        Ok(U256::from(price.saturating_to_num::<u64>()))
    }

    #[precompile::public("getTaoInPool(uint16)")]
    #[precompile::view]
    fn get_tao_in_pool(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::SubnetTAO::<R>::get(NetUid::from(netuid)))
    }

    #[precompile::public("getAlphaInPool(uint16)")]
    #[precompile::view]
    fn get_alpha_in_pool(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::SubnetAlphaIn::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("getAlphaOutPool(uint16)")]
    #[precompile::view]
    fn get_alpha_out_pool(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::SubnetAlphaOut::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("getAlphaIssuance(uint16)")]
    #[precompile::view]
    fn get_alpha_issuance(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::Pallet::<R>::get_alpha_issuance(
            netuid.into(),
        ))
    }

    #[precompile::public("getTaoWeight()")]
    #[precompile::view]
    fn get_tao_weight(_handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        let weight = pallet_subtensor::Pallet::<R>::get_tao_weight();
        Ok(U256::from(weight.saturating_to_num::<u64>()))
    }

    #[precompile::public("simSwapTaoForAlpha(uint16,uint64)")]
    #[precompile::view]
    fn sim_swap_tao_for_alpha(
        _handle: &mut impl PrecompileHandle,
        netuid: u16,
        tao: u64,
    ) -> EvmResult<U256> {
        let swap_result =
            <pallet_subtensor_swap::Pallet<R> as SwapHandler<R::AccountId>>::sim_swap(
                netuid.into(),
                OrderType::Buy,
                tao,
            )
            .map_err(|e| PrecompileFailure::Error {
                exit_status: ExitError::Other(Into::<&'static str>::into(e).into()),
            })?;
        Ok(U256::from(swap_result.amount_paid_out))
    }

    #[precompile::public("simSwapAlphaForTao(uint16,uint64)")]
    #[precompile::view]
    fn sim_swap_alpha_for_tao(
        _handle: &mut impl PrecompileHandle,
        netuid: u16,
        alpha: u64,
    ) -> EvmResult<U256> {
        let swap_result =
            <pallet_subtensor_swap::Pallet<R> as SwapHandler<R::AccountId>>::sim_swap(
                netuid.into(),
                OrderType::Sell,
                alpha,
            )
            .map_err(|e| PrecompileFailure::Error {
                exit_status: ExitError::Other(Into::<&'static str>::into(e).into()),
            })?;
        Ok(U256::from(swap_result.amount_paid_out))
    }

    #[precompile::public("getSubnetMechanism(uint16)")]
    #[precompile::view]
    fn get_subnet_mechanism(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::SubnetMechanism::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("getRootNetuid()")]
    #[precompile::view]
    fn get_root_netuid(_handle: &mut impl PrecompileHandle) -> EvmResult<u16> {
        Ok(NetUid::ROOT.into())
    }

    #[precompile::public("getEMAPriceHalvingBlocks(uint16)")]
    #[precompile::view]
    fn get_ema_price_halving_blocks(
        _handle: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<u64> {
        Ok(pallet_subtensor::EMAPriceHalvingBlocks::<R>::get(
            NetUid::from(netuid),
        ))
    }

    #[precompile::public("getSubnetVolume(uint16)")]
    #[precompile::view]
    fn get_subnet_volume(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        Ok(U256::from(pallet_subtensor::SubnetVolume::<R>::get(
            NetUid::from(netuid),
        )))
    }
}
