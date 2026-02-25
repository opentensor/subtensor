use core::marker::PhantomData;

use fp_evm::{ExitError, PrecompileFailure};
use pallet_evm::{BalanceConverter, PrecompileHandle, SubstrateBalance};
use precompile_utils::EvmResult;
use sp_core::U256;
use sp_std::vec::Vec;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{NetUid, Token};
use subtensor_swap_interface::{Order, SwapHandler};

use crate::PrecompileExt;

pub struct AlphaPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for AlphaPrecompile<R>
where
    R: frame_system::Config
        + pallet_subtensor::Config
        + pallet_subtensor_swap::Config
        + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
{
    const INDEX: u64 = 2056;
}

#[precompile_utils::precompile]
impl<R> AlphaPrecompile<R>
where
    R: frame_system::Config
        + pallet_subtensor::Config
        + pallet_subtensor_swap::Config
        + pallet_evm::Config,
{
    #[precompile::public("getAlphaPrice(uint16)")]
    #[precompile::view]
    fn get_alpha_price(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        let current_alpha_price =
            <pallet_subtensor_swap::Pallet<R> as SwapHandler>::current_alpha_price(netuid.into());
        let price = current_alpha_price.saturating_mul(U96F32::from_num(1_000_000_000));
        let price: SubstrateBalance = price.saturating_to_num::<u64>().into();
        let price_eth = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(price)
            .map(|amount| amount.into_u256())
            .ok_or(ExitError::InvalidRange)?;

        Ok(price_eth)
    }

    #[precompile::public("getMovingAlphaPrice(uint16)")]
    #[precompile::view]
    fn get_moving_alpha_price(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        let moving_alpha_price: U96F32 =
            pallet_subtensor::Pallet::<R>::get_moving_alpha_price(netuid.into());
        let price = moving_alpha_price.saturating_mul(U96F32::from_num(1_000_000_000));
        let price: SubstrateBalance = price.saturating_to_num::<u64>().into();
        let price_eth = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(price)
            .map(|amount| amount.into_u256())
            .ok_or(ExitError::InvalidRange)?;

        Ok(price_eth)
    }

    #[precompile::public("getTaoInPool(uint16)")]
    #[precompile::view]
    fn get_tao_in_pool(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::SubnetTAO::<R>::get(NetUid::from(netuid)).to_u64())
    }

    #[precompile::public("getAlphaInPool(uint16)")]
    #[precompile::view]
    fn get_alpha_in_pool(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::SubnetAlphaIn::<R>::get(NetUid::from(netuid)).into())
    }

    #[precompile::public("getAlphaOutPool(uint16)")]
    #[precompile::view]
    fn get_alpha_out_pool(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::SubnetAlphaOut::<R>::get(NetUid::from(netuid)).into())
    }

    #[precompile::public("getAlphaIssuance(uint16)")]
    #[precompile::view]
    fn get_alpha_issuance(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::Pallet::<R>::get_alpha_issuance(netuid.into()).into())
    }

    #[precompile::public("getTaoWeight()")]
    #[precompile::view]
    fn get_tao_weight(_handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        let tao_weight = pallet_subtensor::TaoWeight::<R>::get();
        Ok(U256::from(tao_weight))
    }

    #[precompile::public("getCKBurn()")]
    #[precompile::view]
    fn get_ck_burn(_handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        let ck_burn = pallet_subtensor::CKBurn::<R>::get();
        Ok(U256::from(ck_burn))
    }

    #[precompile::public("simSwapTaoForAlpha(uint16,uint64)")]
    #[precompile::view]
    fn sim_swap_tao_for_alpha(
        _handle: &mut impl PrecompileHandle,
        netuid: u16,
        tao: u64,
    ) -> EvmResult<U256> {
        let order = pallet_subtensor::GetAlphaForTao::<R>::with_amount(tao);
        let swap_result =
            <pallet_subtensor_swap::Pallet<R> as SwapHandler>::sim_swap(netuid.into(), order)
                .map_err(|e| PrecompileFailure::Error {
                    exit_status: ExitError::Other(Into::<&'static str>::into(e).into()),
                })?;
        Ok(U256::from(swap_result.amount_paid_out.to_u64()))
    }

    #[precompile::public("simSwapAlphaForTao(uint16,uint64)")]
    #[precompile::view]
    fn sim_swap_alpha_for_tao(
        _handle: &mut impl PrecompileHandle,
        netuid: u16,
        alpha: u64,
    ) -> EvmResult<U256> {
        let order = pallet_subtensor::GetTaoForAlpha::<R>::with_amount(alpha);
        let swap_result =
            <pallet_subtensor_swap::Pallet<R> as SwapHandler>::sim_swap(netuid.into(), order)
                .map_err(|e| PrecompileFailure::Error {
                    exit_status: ExitError::Other(Into::<&'static str>::into(e).into()),
                })?;
        Ok(U256::from(swap_result.amount_paid_out.to_u64()))
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

    #[precompile::public("getTaoInEmission(uint16)")]
    #[precompile::view]
    fn get_tao_in_emission(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        Ok(U256::from(
            pallet_subtensor::SubnetTaoInEmission::<R>::get(NetUid::from(netuid)).to_u64(),
        ))
    }

    #[precompile::public("getAlphaInEmission(uint16)")]
    #[precompile::view]
    fn get_alpha_in_emission(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        Ok(U256::from(
            pallet_subtensor::SubnetAlphaInEmission::<R>::get(NetUid::from(netuid)).to_u64(),
        ))
    }

    #[precompile::public("getAlphaOutEmission(uint16)")]
    #[precompile::view]
    fn get_alpha_out_emission(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        Ok(U256::from(
            pallet_subtensor::SubnetAlphaOutEmission::<R>::get(NetUid::from(netuid)).to_u64(),
        ))
    }

    #[precompile::public("getSumAlphaPrice()")]
    #[precompile::view]
    fn get_sum_alpha_price(_handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        let netuids = pallet_subtensor::NetworksAdded::<R>::iter()
            .filter(|(netuid, _)| *netuid != NetUid::ROOT)
            .collect::<Vec<_>>();

        let mut sum_alpha_price: U96F32 = U96F32::from_num(0);
        for (netuid, _) in netuids {
            let price = <pallet_subtensor_swap::Pallet<R> as SwapHandler>::current_alpha_price(
                netuid.into(),
            );

            if price < U96F32::from_num(1) {
                sum_alpha_price = sum_alpha_price.saturating_add(price);
            }
        }

        let price = sum_alpha_price.saturating_mul(U96F32::from_num(1_000_000_000));
        let price: SubstrateBalance = price.saturating_to_num::<u64>().into();
        let price_eth = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(price)
            .map(|amount| amount.into_u256())
            .ok_or(ExitError::InvalidRange)?;

        Ok(price_eth)
    }
}
