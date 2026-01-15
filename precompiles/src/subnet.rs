use core::marker::PhantomData;

use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::ConstU32;
use frame_support::traits::IsSubType;
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, PrecompileHandle};
use pallet_rate_limiting::{RateLimitKind, RateLimitTarget};
use precompile_utils::{EvmResult, prelude::BoundedString};
use sp_core::H256;
use sp_runtime::traits::{AsSystemOriginSigner, Dispatchable, SaturatedConversion};
use sp_std::vec;
use subtensor_runtime_common::{Currency, NetUid, rate_limiting};

use crate::{PrecompileExt, PrecompileHandleExt};

pub struct SubnetPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for SubnetPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_admin_utils::Config
        + pallet_rate_limiting::Config<
            LimitScope = NetUid,
            GroupId = subtensor_runtime_common::rate_limiting::GroupId,
            RuntimeCall = <R as frame_system::Config>::RuntimeCall,
        > + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + From<pallet_rate_limiting::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2051;
}

#[precompile_utils::precompile]
impl<R> SubnetPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_admin_utils::Config
        + pallet_rate_limiting::Config<
            LimitScope = NetUid,
            GroupId = subtensor_runtime_common::rate_limiting::GroupId,
            RuntimeCall = <R as frame_system::Config>::RuntimeCall,
        > + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_admin_utils::Call<R>>
        + From<pallet_rate_limiting::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>,
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
        let identity = pallet_subtensor::SubnetIdentityOfV3 {
            subnet_name: subnet_name.into(),
            github_repo: github_repo.into(),
            subnet_contact: subnet_contact.into(),
            subnet_url: subnet_url.into(),
            discord: discord.into(),
            description: description.into(),
            logo_url: vec![],
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

    #[precompile::public(
        "registerNetwork(bytes32,string,string,string,string,string,string,string,string)"
    )]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn register_network_with_identity_v2(
        handle: &mut impl PrecompileHandle,
        hotkey: H256,
        subnet_name: BoundedString<ConstU32<256>>,
        github_repo: BoundedString<ConstU32<1024>>,
        subnet_contact: BoundedString<ConstU32<1024>>,
        subnet_url: BoundedString<ConstU32<1024>>,
        discord: BoundedString<ConstU32<256>>,
        description: BoundedString<ConstU32<1024>>,
        additional: BoundedString<ConstU32<1024>>,
        logo_url: BoundedString<ConstU32<1024>>,
    ) -> EvmResult<()> {
        let hotkey = R::AccountId::from(hotkey.0);
        let identity = pallet_subtensor::SubnetIdentityOfV3 {
            subnet_name: subnet_name.into(),
            github_repo: github_repo.into(),
            subnet_contact: subnet_contact.into(),
            subnet_url: subnet_url.into(),
            discord: discord.into(),
            description: description.into(),
            logo_url: logo_url.into(),
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
        Ok(pallet_subtensor::Pallet::<R>::get_serving_rate_limit(
            NetUid::from(netuid),
        ))
    }

    #[precompile::public("setServingRateLimit(uint16,uint64)")]
    #[precompile::payable]
    fn set_serving_rate_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        serving_rate_limit: u64,
    ) -> EvmResult<()> {
        let call = pallet_rate_limiting::Call::<R>::set_rate_limit {
            target: RateLimitTarget::Group(subtensor_runtime_common::rate_limiting::GROUP_SERVE),
            scope: Some(netuid.into()),
            limit: RateLimitKind::Exact(serving_rate_limit.saturated_into()),
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getMinDifficulty(uint16)")]
    #[precompile::view]
    fn get_min_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MinDifficulty::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setMinDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_min_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_min_difficulty {
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::MaxDifficulty::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setMaxDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_max_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        max_difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_max_difficulty {
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::WeightsVersionKey::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setWeightsVersionKey(uint16,uint64)")]
    #[precompile::payable]
    fn set_weights_version_key(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        weights_version_key: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_weights_version_key {
            netuid: netuid.into(),
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
        let target = RateLimitTarget::Group(rate_limiting::GROUP_WEIGHTS_SUBNET);
        let scope = Some(NetUid::from(netuid));
        let limit =
            pallet_rate_limiting::Pallet::<R>::resolved_limit(&target, &scope).unwrap_or_default();
        Ok(limit.saturated_into())
    }

    #[precompile::public("setWeightsSetRateLimit(uint16,uint64)")]
    #[precompile::payable]
    fn set_weights_set_rate_limit(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        weights_set_rate_limit: u64,
    ) -> EvmResult<()> {
        let call = pallet_rate_limiting::Call::<R>::set_rate_limit {
            target: RateLimitTarget::Group(rate_limiting::GROUP_WEIGHTS_SUBNET),
            scope: Some(netuid.into()),
            limit: RateLimitKind::Exact(weights_set_rate_limit.saturated_into()),
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getAdjustmentAlpha(uint16)")]
    #[precompile::view]
    fn get_adjustment_alpha(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::AdjustmentAlpha::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setAdjustmentAlpha(uint16,uint64)")]
    #[precompile::payable]
    fn set_adjustment_alpha(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        adjustment_alpha: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_adjustment_alpha {
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::Pallet::<R>::get_max_weight_limit(
            NetUid::from(netuid),
        ))
    }

    #[precompile::public("getImmunityPeriod(uint16)")]
    #[precompile::view]
    fn get_immunity_period(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::ImmunityPeriod::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setImmunityPeriod(uint16,uint16)")]
    #[precompile::payable]
    fn set_immunity_period(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        immunity_period: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_immunity_period {
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::MinAllowedWeights::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setMinAllowedWeights(uint16,uint16)")]
    #[precompile::payable]
    fn set_min_allowed_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        min_allowed_weights: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_min_allowed_weights {
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::Kappa::<R>::get(NetUid::from(netuid)))
    }

    #[precompile::public("setKappa(uint16,uint16)")]
    #[precompile::payable]
    fn set_kappa(handle: &mut impl PrecompileHandle, netuid: u16, kappa: u16) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_kappa {
            netuid: netuid.into(),
            kappa,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getRho(uint16)")]
    #[precompile::view]
    fn get_rho(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Rho::<R>::get(NetUid::from(netuid)))
    }

    #[precompile::public("getAlphaSigmoidSteepness(uint16)")]
    #[precompile::view]
    fn get_alpha_sigmoid_steepness(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::AlphaSigmoidSteepness::<R>::get(NetUid::from(netuid)) as u16)
    }

    #[precompile::public("setRho(uint16,uint16)")]
    #[precompile::payable]
    fn set_rho(handle: &mut impl PrecompileHandle, netuid: u16, rho: u16) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_rho {
            netuid: netuid.into(),
            rho,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("setAlphaSigmoidSteepness(uint16,uint16)")]
    #[precompile::payable]
    fn set_alpha_sigmoid_steepness(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        steepness: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_alpha_sigmoid_steepness {
            netuid: netuid.into(),
            steepness: (steepness as i16),
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getActivityCutoff(uint16)")]
    #[precompile::view]
    fn get_activity_cutoff(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::ActivityCutoff::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setActivityCutoff(uint16,uint16)")]
    #[precompile::payable]
    fn set_activity_cutoff(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        activity_cutoff: u16,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_activity_cutoff {
            netuid: netuid.into(),
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
            NetUid::from(netuid),
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
            netuid: netuid.into(),
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
            NetUid::from(netuid),
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
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::MinBurn::<R>::get(NetUid::from(netuid)).to_u64())
    }

    #[precompile::public("setMinBurn(uint16,uint64)")]
    #[precompile::payable]
    fn set_min_burn(
        _handle: &mut impl PrecompileHandle,
        _netuid: u16,
        _min_burn: u64,
    ) -> EvmResult<()> {
        // DEPRECATED. The subnet owner cannot set the min burn anymore.
        Ok(())
    }

    #[precompile::public("getMaxBurn(uint16)")]
    #[precompile::view]
    fn get_max_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::MaxBurn::<R>::get(NetUid::from(netuid)).to_u64())
    }

    #[precompile::public("setMaxBurn(uint16,uint64)")]
    #[precompile::payable]
    fn set_max_burn(
        _handle: &mut impl PrecompileHandle,
        _netuid: u16,
        _max_burn: u64,
    ) -> EvmResult<()> {
        // DEPRECATED. The subnet owner cannot set the max burn anymore.
        Ok(())
    }

    #[precompile::public("getDifficulty(uint16)")]
    #[precompile::view]
    fn get_difficulty(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::Difficulty::<R>::get(NetUid::from(netuid)))
    }

    #[precompile::public("setDifficulty(uint16,uint64)")]
    #[precompile::payable]
    fn set_difficulty(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        difficulty: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_difficulty {
            netuid: netuid.into(),
            difficulty,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getBondsMovingAverage(uint16)")]
    #[precompile::view]
    fn get_bonds_moving_average(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::BondsMovingAverage::<R>::get(
            NetUid::from(netuid),
        ))
    }

    #[precompile::public("setBondsMovingAverage(uint16,uint64)")]
    #[precompile::payable]
    fn set_bonds_moving_average(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        bonds_moving_average: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_bonds_moving_average {
            netuid: netuid.into(),
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
            NetUid::from(netuid),
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
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::LiquidAlphaOn::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setLiquidAlphaEnabled(uint16,bool)")]
    #[precompile::payable]
    fn set_liquid_alpha_enabled(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        enabled: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_liquid_alpha_enabled {
            netuid: netuid.into(),
            enabled,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getYuma3Enabled(uint16)")]
    #[precompile::view]
    fn get_yuma3_enabled(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<bool> {
        Ok(pallet_subtensor::Yuma3On::<R>::get(NetUid::from(netuid)))
    }

    #[precompile::public("getBondsResetEnabled(uint16)")]
    #[precompile::view]
    fn get_bonds_reset_enabled(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<bool> {
        Ok(pallet_subtensor::BondsResetOn::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("setYuma3Enabled(uint16,bool)")]
    #[precompile::payable]
    fn set_yuma3_enabled(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        enabled: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_yuma3_enabled {
            netuid: netuid.into(),
            enabled,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("setBondsResetEnabled(uint16,bool)")]
    #[precompile::payable]
    fn set_bonds_reset_enabled(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        enabled: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_bonds_reset_enabled {
            netuid: netuid.into(),
            enabled,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("getAlphaValues(uint16)")]
    #[precompile::view]
    fn get_alpha_values(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<(u16, u16)> {
        Ok(pallet_subtensor::AlphaValues::<R>::get(NetUid::from(
            netuid,
        )))
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
            netuid: netuid.into(),
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
        Ok(pallet_subtensor::RevealPeriodEpochs::<R>::get(
            NetUid::from(netuid),
        ))
    }

    #[precompile::public("setCommitRevealWeightsInterval(uint16,uint64)")]
    #[precompile::payable]
    fn set_commit_reveal_weights_interval(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        interval: u64,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_commit_reveal_weights_interval {
            netuid: netuid.into(),
            interval,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public("toggleTransfers(uint16,bool)")]
    #[precompile::payable]
    fn toggle_transfers(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        toggle: bool,
    ) -> EvmResult<()> {
        let call = pallet_admin_utils::Call::<R>::sudo_set_toggle_transfer {
            netuid: netuid.into(),
            toggle,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }
}
