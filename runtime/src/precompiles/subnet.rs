use frame_support::traits::ConstU32;
use frame_system::RawOrigin;
use pallet_evm::PrecompileHandle;
use precompile_utils::{prelude::BoundedString, EvmResult};
use sp_core::H256;

use crate::precompiles::{parse_pubkey, PrecompileExt, PrecompileHandleExt};
use crate::Runtime;

pub struct SubnetPrecompile;

impl PrecompileExt for SubnetPrecompile {
    const INDEX: u64 = 2051;
    const ADDRESS_SS58: [u8; 32] = [
        0x3a, 0x86, 0x18, 0xfb, 0xbb, 0x1b, 0xbc, 0x47, 0x86, 0x64, 0xff, 0x53, 0x46, 0x18, 0x0c,
        0x35, 0xd0, 0x9f, 0xac, 0x26, 0xf2, 0x02, 0x70, 0x85, 0xb3, 0x1c, 0x56, 0xc1, 0x06, 0x3c,
        0x1c, 0xd3,
    ];
}

#[precompile_utils::precompile]
impl SubnetPrecompile {
    #[precompile::public("registerNetwork(bytes32)")]
    #[precompile::payable]
    fn register_network(handle: &mut impl PrecompileHandle, hotkey: H256) -> EvmResult<()> {
        let (hotkey, _) = parse_pubkey(hotkey.as_bytes())?;
        let call = pallet_subtensor::Call::<Runtime>::register_network_with_identity {
            hotkey,
            identity: None,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
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
        let (hotkey, _) = parse_pubkey(hotkey.as_bytes())?;
        let identity = pallet_subtensor::SubnetIdentityOfV2 {
            subnet_name: subnet_name.into(),
            github_repo: github_repo.into(),
            subnet_contact: subnet_contact.into(),
            subnet_url: subnet_url.into(),
            discord: discord.into(),
            description: description.into(),
            additional: additional.into(),
        };

        let call = pallet_subtensor::Call::<Runtime>::register_network_with_identity {
            hotkey,
            identity: Some(identity),
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getServingRateLimit(uint16)")]
    #[precompile::view]
    fn get_serving_rate_limit(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::ServingRateLimit::<Runtime>::get(netuid))
    }

    #[precompile::public("setServingRateLimit(uint16,uint64)")]
    #[precompile::payable]
    fn set_serving_rate_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        serving_rate_limit: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_serving_rate_limit {
            netuid,
            serving_rate_limit,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getMinDifficulty(uint16)")]
    #[precompile::view]
    fn get_min_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MinDifficulty::<Runtime>::get(netuid))
    }

    #[precompile::public("setMinDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_min_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_min_difficulty {
            netuid,
            min_difficulty,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getMaxDifficulty(uint16)")]
    #[precompile::view]
    fn get_max_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MaxDifficulty::<Runtime>::get(netuid))
    }

    #[precompile::public("setMaxDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_max_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        max_difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_max_difficulty {
            netuid,
            max_difficulty,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getWeightsVersionKey(uint16)")]
    #[precompile::view]
    fn get_weights_version_key(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::WeightsVersionKey::<Runtime>::get(netuid))
    }

    #[precompile::public("setWeightsVersionKey(uint16,uint64)")]
    #[precompile::payable]
    fn set_weights_version_key(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        weights_version_key: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_weights_version_key {
            netuid,
            weights_version_key,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getWeightsSetRateLimit(uint16)")]
    #[precompile::view]
    fn get_weights_set_rate_limit(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::WeightsSetRateLimit::<Runtime>::get(
            netuid,
        ))
    }

    #[precompile::public("setWeightsSetRateLimit(uint16,uint64)")]
    #[precompile::payable]
    fn set_weights_set_rate_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        weights_set_rate_limit: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_weights_set_rate_limit {
            netuid,
            weights_set_rate_limit,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getAdjustmentAlpha(uint16)")]
    #[precompile::view]
    fn get_adjustment_alpha(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::AdjustmentAlpha::<Runtime>::get(netuid))
    }

    #[precompile::public("setAdjustmentAlpha(uint16,uint64)")]
    #[precompile::payable]
    fn set_adjustment_alpha(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        adjustment_alpha: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_adjustment_alpha {
            netuid,
            adjustment_alpha,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getMaxWeightLimit(uint16)")]
    #[precompile::view]
    fn get_max_weight_limit(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::MaxWeightsLimit::<Runtime>::get(netuid))
    }

    #[precompile::public("setMaxWeightLimit(uint16,uint16)")]
    #[precompile::payable]
    fn set_max_weight_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        max_weight_limit: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_max_weight_limit {
            netuid,
            max_weight_limit,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getImmunityPeriod(uint16)")]
    #[precompile::view]
    fn get_immunity_period(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::ImmunityPeriod::<Runtime>::get(netuid))
    }

    #[precompile::public("setImmunityPeriod(uint16,uint16)")]
    #[precompile::payable]
    fn set_immunity_period(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        immunity_period: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_immunity_period {
            netuid,
            immunity_period,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getMinAllowedWeights(uint16)")]
    #[precompile::view]
    fn get_min_allowed_weights(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::MinAllowedWeights::<Runtime>::get(netuid))
    }

    #[precompile::public("setMinAllowedWeights(uint16,uint16)")]
    #[precompile::payable]
    fn set_min_allowed_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_allowed_weights: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_min_allowed_weights {
            netuid,
            min_allowed_weights,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getKappa(uint16)")]
    #[precompile::view]
    fn get_kappa(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Kappa::<Runtime>::get(netuid))
    }

    #[precompile::public("setKappa(uint16,uint16)")]
    #[precompile::payable]
    fn set_kappa(handle: &mut impl PrecompileHandle, netuid: u16, kappa: u16) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_kappa { netuid, kappa };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getRho(uint16)")]
    #[precompile::view]
    fn get_rho(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Rho::<Runtime>::get(netuid))
    }

    #[precompile::public("setRho(uint16,uint16)")]
    #[precompile::payable]
    fn set_rho(handle: &mut impl PrecompileHandle, netuid: u16, rho: u16) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_rho { netuid, rho };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getActivityCutoff(uint16)")]
    #[precompile::view]
    fn get_activity_cutoff(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::ActivityCutoff::<Runtime>::get(netuid))
    }

    #[precompile::public("setActivityCutoff(uint16,uint16)")]
    #[precompile::payable]
    fn set_activity_cutoff(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        activity_cutoff: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_activity_cutoff {
            netuid,
            activity_cutoff,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getNetworkRegistrationAllowed(uint16)")]
    #[precompile::view]
    fn get_network_registration_allowed(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::NetworkRegistrationAllowed::<Runtime>::get(netuid))
    }

    #[precompile::public("setNetworkRegistrationAllowed(uint16,bool)")]
    #[precompile::payable]
    fn set_network_registration_allowed(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        registration_allowed: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_network_registration_allowed {
            netuid,
            registration_allowed,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getNetworkPowRegistrationAllowed(uint16)")]
    #[precompile::view]
    fn get_network_pow_registration_allowed(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::NetworkPowRegistrationAllowed::<Runtime>::get(netuid))
    }

    #[precompile::public("setNetworkPowRegistrationAllowed(uint16,bool)")]
    #[precompile::payable]
    fn set_network_pow_registration_allowed(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        registration_allowed: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_network_pow_registration_allowed {
            netuid,
            registration_allowed,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getMinBurn(uint16)")]
    #[precompile::view]
    fn get_min_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MinBurn::<Runtime>::get(netuid))
    }

    #[precompile::public("setMinBurn(uint16,uint64)")]
    #[precompile::payable]
    fn set_min_burn(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_burn: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_min_burn { netuid, min_burn };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getMaxBurn(uint16)")]
    #[precompile::view]
    fn get_max_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MaxBurn::<Runtime>::get(netuid))
    }

    #[precompile::public("setMaxBurn(uint16,uint64)")]
    #[precompile::payable]
    fn set_max_burn(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        max_burn: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_max_burn { netuid, max_burn };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getDifficulty(uint16)")]
    #[precompile::view]
    fn get_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::Difficulty::<Runtime>::get(netuid))
    }

    #[precompile::public("setDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_difficulty { netuid, difficulty };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getBondsMovingAverage(uint16)")]
    #[precompile::view]
    fn get_bonds_moving_average(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::BondsMovingAverage::<Runtime>::get(netuid))
    }

    #[precompile::public("setBondsMovingAverage(uint16,uint64)")]
    #[precompile::payable]
    fn set_bonds_moving_average(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        bonds_moving_average: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_bonds_moving_average {
            netuid,
            bonds_moving_average,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getCommitRevealWeightsEnabled(uint16)")]
    #[precompile::view]
    fn get_commit_reveal_weights_enabled(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::CommitRevealWeightsEnabled::<Runtime>::get(netuid))
    }

    #[precompile::public("setCommitRevealWeightsEnabled(uint16,bool)")]
    #[precompile::payable]
    fn set_commit_reveal_weights_enabled(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        enabled: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_commit_reveal_weights_enabled {
            netuid,
            enabled,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getLiquidAlphaEnabled(uint16)")]
    #[precompile::view]
    fn get_liquid_alpha_enabled(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<bool> {
        Ok(pallet_subtensor::LiquidAlphaOn::<Runtime>::get(netuid))
    }

    #[precompile::public("setLiquidAlphaEnabled(uint16,bool)")]
    #[precompile::payable]
    fn set_liquid_alpha_enabled(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        enabled: bool,
    ) -> EvmResult<()> {
        let call =
            pallet_admin_utils::Call::<Runtime>::sudo_set_liquid_alpha_enabled { netuid, enabled };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getAlphaValues(uint16)")]
    #[precompile::view]
    fn get_alpha_values(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<(u16, u16)> {
        Ok(pallet_subtensor::AlphaValues::<Runtime>::get(netuid))
    }

    #[precompile::public("setAlphaValues(uint16,uint16,uint16)")]
    #[precompile::payable]
    fn set_alpha_values(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        alpha_low: u16,
        alpha_high: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_alpha_values {
            netuid,
            alpha_low,
            alpha_high,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("getCommitRevealWeightsInterval(uint16)")]
    #[precompile::view]
    fn get_commit_reveal_weights_interval(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<u64> {
        Ok(pallet_subtensor::RevealPeriodEpochs::<Runtime>::get(netuid))
    }

    #[precompile::public("setCommitRevealWeightsInterval(uint16,uint64)")]
    #[precompile::payable]
    fn set_commit_reveal_weights_interval(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        interval: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<Runtime>::sudo_set_commit_reveal_weights_interval {
            netuid,
            interval,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }
}
