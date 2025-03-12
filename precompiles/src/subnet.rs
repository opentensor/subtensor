use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::ConstU32;
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, PrecompileHandle};
use precompile_utils::{EvmResult, prelude::BoundedString};
use sp_core::H256;
use sp_runtime::traits::Dispatchable;

use crate::{PrecompileExt, PrecompileHandleExt};

pub struct SubnetPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for SubnetPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_admin_utils::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2051;
}

#[precompile_utils::precompile]
impl<R> SubnetPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_admin_utils::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    #[precompile::public("registerNetwork(bytes32)")]
    #[precompile::payable]
    fn register_network(handle: &mut impl PrecompileHandle, hotkey: H256) -> EvmResult<()> {
        let hotkey = R::AccountId::from(hotkey.0);
        let call = pallet_subtensor::Call::<R>::register_network_with_identity {
            hotkey,
            identity: None,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public(
        "registerNetwork(bytes32,string,string,string,string,string,string,string)"
    )]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn register_network_with_identity(
        handle: &mut impl PrecompileHandle,
        hotkey: H256,
        subnet_name: BoundedString<ConstU32<256>>,
        github_repo: BoundedString<ConstU32<1024>>,
        subnet_contact: BoundedString<ConstU32<1024>>,
        subnet_url: BoundedString<ConstU32<1024>>,
        discord: BoundedString<ConstU32<256>>,
        description: BoundedString<ConstU32<1024>>,
        additional: BoundedString<ConstU32<1024>>,
    ) -> EvmResult<()> {
        let hotkey = R::AccountId::from(hotkey.0);
        let identity = pallet_subtensor::SubnetIdentityOfV2 {
            subnet_name: subnet_name.into(),
            github_repo: github_repo.into(),
            subnet_contact: subnet_contact.into(),
            subnet_url: subnet_url.into(),
            discord: discord.into(),
            description: description.into(),
            additional: additional.into(),
        };

        let call = pallet_subtensor::Call::<R>::register_network_with_identity {
            hotkey,
            identity: Some(identity),
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getServingRateLimit(uint16)")]
    #[precompile::view]
    fn get_serving_rate_limit(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::ServingRateLimit::<R>::get(netuid))
    }

    #[precompile::public("setServingRateLimit(uint16,uint64)")]
    #[precompile::payable]
    fn set_serving_rate_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        serving_rate_limit: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_serving_rate_limit {
            netuid,
            serving_rate_limit,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getMinDifficulty(uint16)")]
    #[precompile::view]
    fn get_min_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MinDifficulty::<R>::get(netuid))
    }

    #[precompile::public("setMinDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_min_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_min_difficulty {
            netuid,
            min_difficulty,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getMaxDifficulty(uint16)")]
    #[precompile::view]
    fn get_max_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MaxDifficulty::<R>::get(netuid))
    }

    #[precompile::public("setMaxDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_max_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        max_difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_max_difficulty {
            netuid,
            max_difficulty,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getWeightsVersionKey(uint16)")]
    #[precompile::view]
    fn get_weights_version_key(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::WeightsVersionKey::<R>::get(netuid))
    }

    #[precompile::public("setWeightsVersionKey(uint16,uint64)")]
    #[precompile::payable]
    fn set_weights_version_key(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        weights_version_key: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_weights_version_key {
            netuid,
            weights_version_key,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getWeightsSetRateLimit(uint16)")]
    #[precompile::view]
    fn get_weights_set_rate_limit(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::WeightsSetRateLimit::<R>::get(netuid))
    }

    #[precompile::public("setWeightsSetRateLimit(uint16,uint64)")]
    #[precompile::payable]
    fn set_weights_set_rate_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        weights_set_rate_limit: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_weights_set_rate_limit {
            netuid,
            weights_set_rate_limit,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getAdjustmentAlpha(uint16)")]
    #[precompile::view]
    fn get_adjustment_alpha(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::AdjustmentAlpha::<R>::get(netuid))
    }

    #[precompile::public("setAdjustmentAlpha(uint16,uint64)")]
    #[precompile::payable]
    fn set_adjustment_alpha(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        adjustment_alpha: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_adjustment_alpha {
            netuid,
            adjustment_alpha,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getMaxWeightLimit(uint16)")]
    #[precompile::view]
    fn get_max_weight_limit(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::MaxWeightsLimit::<R>::get(netuid))
    }

    #[precompile::public("setMaxWeightLimit(uint16,uint16)")]
    #[precompile::payable]
    fn set_max_weight_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        max_weight_limit: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_max_weight_limit {
            netuid,
            max_weight_limit,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getImmunityPeriod(uint16)")]
    #[precompile::view]
    fn get_immunity_period(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::ImmunityPeriod::<R>::get(netuid))
    }

    #[precompile::public("setImmunityPeriod(uint16,uint16)")]
    #[precompile::payable]
    fn set_immunity_period(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        immunity_period: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_immunity_period {
            netuid,
            immunity_period,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getMinAllowedWeights(uint16)")]
    #[precompile::view]
    fn get_min_allowed_weights(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::MinAllowedWeights::<R>::get(netuid))
    }

    #[precompile::public("setMinAllowedWeights(uint16,uint16)")]
    #[precompile::payable]
    fn set_min_allowed_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_allowed_weights: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_min_allowed_weights {
            netuid,
            min_allowed_weights,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getKappa(uint16)")]
    #[precompile::view]
    fn get_kappa(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Kappa::<R>::get(netuid))
    }

    #[precompile::public("setKappa(uint16,uint16)")]
    #[precompile::payable]
    fn set_kappa(handle: &mut impl PrecompileHandle, netuid: u16, kappa: u16) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_kappa { netuid, kappa };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getRho(uint16)")]
    #[precompile::view]
    fn get_rho(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Rho::<R>::get(netuid))
    }

    #[precompile::public("setRho(uint16,uint16)")]
    #[precompile::payable]
    fn set_rho(handle: &mut impl PrecompileHandle, netuid: u16, rho: u16) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_rho { netuid, rho };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getActivityCutoff(uint16)")]
    #[precompile::view]
    fn get_activity_cutoff(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::ActivityCutoff::<R>::get(netuid))
    }

    #[precompile::public("setActivityCutoff(uint16,uint16)")]
    #[precompile::payable]
    fn set_activity_cutoff(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        activity_cutoff: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_activity_cutoff {
            netuid,
            activity_cutoff,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getNetworkRegistrationAllowed(uint16)")]
    #[precompile::view]
    fn get_network_registration_allowed(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::NetworkRegistrationAllowed::<R>::get(
            netuid,
        ))
    }

    #[precompile::public("setNetworkRegistrationAllowed(uint16,bool)")]
    #[precompile::payable]
    fn set_network_registration_allowed(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        registration_allowed: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_network_registration_allowed {
            netuid,
            registration_allowed,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getNetworkPowRegistrationAllowed(uint16)")]
    #[precompile::view]
    fn get_network_pow_registration_allowed(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::NetworkPowRegistrationAllowed::<R>::get(
            netuid,
        ))
    }

    #[precompile::public("setNetworkPowRegistrationAllowed(uint16,bool)")]
    #[precompile::payable]
    fn set_network_pow_registration_allowed(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        registration_allowed: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_network_pow_registration_allowed {
            netuid,
            registration_allowed,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getMinBurn(uint16)")]
    #[precompile::view]
    fn get_min_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MinBurn::<R>::get(netuid))
    }

    #[precompile::public("setMinBurn(uint16,uint64)")]
    #[precompile::payable]
    fn set_min_burn(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_burn: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_min_burn { netuid, min_burn };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getMaxBurn(uint16)")]
    #[precompile::view]
    fn get_max_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MaxBurn::<R>::get(netuid))
    }

    #[precompile::public("setMaxBurn(uint16,uint64)")]
    #[precompile::payable]
    fn set_max_burn(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        max_burn: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_max_burn { netuid, max_burn };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getDifficulty(uint16)")]
    #[precompile::view]
    fn get_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::Difficulty::<R>::get(netuid))
    }

    #[precompile::public("setDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_difficulty { netuid, difficulty };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getBondsMovingAverage(uint16)")]
    #[precompile::view]
    fn get_bonds_moving_average(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::BondsMovingAverage::<R>::get(netuid))
    }

    #[precompile::public("setBondsMovingAverage(uint16,uint64)")]
    #[precompile::payable]
    fn set_bonds_moving_average(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        bonds_moving_average: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_bonds_moving_average {
            netuid,
            bonds_moving_average,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getCommitRevealWeightsEnabled(uint16)")]
    #[precompile::view]
    fn get_commit_reveal_weights_enabled(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::CommitRevealWeightsEnabled::<R>::get(
            netuid,
        ))
    }

    #[precompile::public("setCommitRevealWeightsEnabled(uint16,bool)")]
    #[precompile::payable]
    fn set_commit_reveal_weights_enabled(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        enabled: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_commit_reveal_weights_enabled {
            netuid,
            enabled,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getLiquidAlphaEnabled(uint16)")]
    #[precompile::view]
    fn get_liquid_alpha_enabled(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<bool> {
        Ok(pallet_subtensor::LiquidAlphaOn::<R>::get(netuid))
    }

    #[precompile::public("setLiquidAlphaEnabled(uint16,bool)")]
    #[precompile::payable]
    fn set_liquid_alpha_enabled(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        enabled: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_liquid_alpha_enabled { netuid, enabled };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getAlphaValues(uint16)")]
    #[precompile::view]
    fn get_alpha_values(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<(u16, u16)> {
        Ok(pallet_subtensor::AlphaValues::<R>::get(netuid))
    }

    #[precompile::public("setAlphaValues(uint16,uint16,uint16)")]
    #[precompile::payable]
    fn set_alpha_values(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        alpha_low: u16,
        alpha_high: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_alpha_values {
            netuid,
            alpha_low,
            alpha_high,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getCommitRevealWeightsInterval(uint16)")]
    #[precompile::view]
    fn get_commit_reveal_weights_interval(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<u64> {
        Ok(pallet_subtensor::RevealPeriodEpochs::<R>::get(netuid))
    }

    #[precompile::public("setCommitRevealWeightsInterval(uint16,uint64)")]
    #[precompile::payable]
    fn set_commit_reveal_weights_interval(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        interval: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_commit_reveal_weights_interval {
            netuid,
            interval,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }
}
