use crate::precompiles::{
    get_method_id, get_pubkey, get_slice, parse_netuid, try_dispatch_runtime_call,
};
use crate::{Runtime, RuntimeCall};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, ExitError, ExitSucceed, HashedAddressMapping, PrecompileFailure,
    PrecompileHandle, PrecompileOutput, PrecompileResult,
};

use sp_core::U256;
use sp_runtime::{traits::BlakeTwo256, AccountId32, Vec};
use sp_std::vec;

pub const SUBNET_PRECOMPILE_INDEX: u64 = 2051;
// bytes with max lenght 1K
pub const MAX_SINGLE_PARAMETER_SIZE: usize = 1024;
// seven bytes with max lenght 1K
pub const MAX_PARAMETER_SIZE: usize = 7 * MAX_SINGLE_PARAMETER_SIZE;
// ss58 public key i.e., the contract sends funds it received to the destination address from the
#[allow(dead_code)]
const CONTRACT_ADDRESS_SS58: [u8; 32] = [
    0x3a, 0x86, 0x18, 0xfb, 0xbb, 0x1b, 0xbc, 0x47, 0x86, 0x64, 0xff, 0x53, 0x46, 0x18, 0x0c, 0x35,
    0xd0, 0x9f, 0xac, 0x26, 0xf2, 0x02, 0x70, 0x85, 0xb3, 0x1c, 0x56, 0xc1, 0x06, 0x3c, 0x1c, 0xd3,
];
pub struct SubnetPrecompile;

impl SubnetPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();
        if txdata.len() > MAX_PARAMETER_SIZE {
            log::error!("the length of subnet call is {} ", txdata.len());
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata
            .get(4..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

        match method_id {
            id if id
                == get_method_id(
                    "registerNetwork(bytes32,bytes,bytes,bytes,bytes,bytes,bytes,bytes)",
                ) =>
            {
                Self::register_network(handle, &method_input)
            }
            id if id == get_method_id("registerNetwork(bytes32)") => {
                Self::register_network(handle, &method_input)
            }

            id if id == get_method_id("getServingRateLimit(uint16)") => {
                Self::get_serving_rate_limit(&method_input)
            }
            id if id == get_method_id("setServingRateLimit(uint16,uint64)") => {
                Self::set_serving_rate_limit(handle, &method_input)
            }

            id if id == get_method_id("getMinDifficulty(uint16)") => {
                Self::get_min_difficulty(&method_input)
            }
            id if id == get_method_id("setMinDifficulty(uint16,uint64)") => {
                Self::set_min_difficulty(handle, &method_input)
            }

            id if id == get_method_id("getMaxDifficulty(uint16)") => {
                Self::get_max_difficulty(&method_input)
            }
            id if id == get_method_id("setMaxDifficulty(uint16,uint64)") => {
                Self::set_max_difficulty(handle, &method_input)
            }

            id if id == get_method_id("getWeightsVersionKey(uint16)") => {
                Self::get_weights_version_key(&method_input)
            }
            id if id == get_method_id("setWeightsVersionKey(uint16,uint64)") => {
                Self::set_weights_version_key(handle, &method_input)
            }

            id if id == get_method_id("getWeightsSetRateLimit(uint16)") => {
                Self::get_weights_set_rate_limit(&method_input)
            }
            id if id == get_method_id("setWeightsSetRateLimit(uint16,uint64)") => {
                Self::set_weights_set_rate_limit(handle, &method_input)
            }

            id if id == get_method_id("getAdjustmentAlpha(uint16)") => {
                Self::get_adjustment_alpha(&method_input)
            }
            id if id == get_method_id("setAdjustmentAlpha(uint16,uint64)") => {
                Self::set_adjustment_alpha(handle, &method_input)
            }

            id if id == get_method_id("getMaxWeightLimit(uint16)") => {
                Self::get_max_weight_limit(&method_input)
            }
            id if id == get_method_id("setMaxWeightLimit(uint16,uint64)") => {
                Self::set_max_weight_limit(handle, &method_input)
            }

            id if id == get_method_id("getImmunityPeriod(uint16)") => {
                Self::get_immunity_period(&method_input)
            }
            id if id == get_method_id("setImmunityPeriod(uint16,uint64)") => {
                Self::set_immunity_period(handle, &method_input)
            }

            id if id == get_method_id("getMinAllowedWeights(uint16)") => {
                Self::get_min_allowed_weights(&method_input)
            }
            id if id == get_method_id("setMinAllowedWeights(uint16,uint16)") => {
                Self::set_min_allowed_weights(handle, &method_input)
            }

            id if id == get_method_id("getKappa(uint16)") => Self::get_kappa(&method_input),
            id if id == get_method_id("setKappa(uint16,uint16)") => {
                Self::set_kappa(handle, &method_input)
            }

            id if id == get_method_id("getRho(uint16)") => Self::get_rho(&method_input),
            id if id == get_method_id("setRho(uint16,uint16)") => {
                Self::set_rho(handle, &method_input)
            }

            id if id == get_method_id("getActivityCutoff(uint16)") => {
                Self::get_activity_cutoff(&method_input)
            }
            id if id == get_method_id("setActivityCutoff(uint16,uint16)") => {
                Self::set_activity_cutoff(handle, &method_input)
            }

            id if id == get_method_id("getNetworkRegistrationAllowed(uint16)") => {
                Self::get_network_registration_allowed(&method_input)
            }
            id if id == get_method_id("setNetworkRegistrationAllowed(uint16,bool)") => {
                Self::set_network_registration_allowed(handle, &method_input)
            }

            id if id == get_method_id("getNetworkPowRegistrationAllowed(uint16)") => {
                Self::get_network_pow_registration_allowed(&method_input)
            }
            id if id == get_method_id("setNetworkPowRegistrationAllowed(uint16,bool)") => {
                Self::set_network_pow_registration_allowed(handle, &method_input)
            }

            id if id == get_method_id("getMinBurn(uint16)") => Self::get_min_burn(&method_input),
            id if id == get_method_id("setMinBurn(uint16,uint64)") => {
                Self::set_min_burn(handle, &method_input)
            }

            id if id == get_method_id("getMaxBurn(uint16)") => Self::get_max_burn(&method_input),
            id if id == get_method_id("setMaxBurn(uint16,uint64)") => {
                Self::set_max_burn(handle, &method_input)
            }

            id if id == get_method_id("getDifficulty(uint16)") => {
                Self::get_difficulty(&method_input)
            }
            id if id == get_method_id("setDifficulty(uint16,uint64)") => {
                Self::set_difficulty(handle, &method_input)
            }

            id if id == get_method_id("getBondsMovingAverage(uint16)") => {
                Self::get_bonds_moving_average(&method_input)
            }
            id if id == get_method_id("setBondsMovingAverage(uint16,uint64)") => {
                Self::set_bonds_moving_average(handle, &method_input)
            }

            id if id == get_method_id("getCommitRevealWeightsEnabled(uint16)") => {
                Self::get_commit_reveal_weights_enabled(&method_input)
            }
            id if id == get_method_id("setCommitRevealWeightsEnabled(uint16,bool)") => {
                Self::set_commit_reveal_weights_enabled(handle, &method_input)
            }

            id if id == get_method_id("getLiquidAlphaEnabled(uint16)") => {
                Self::get_liquid_alpha_enabled(&method_input)
            }
            id if id == get_method_id("setLiquidAlphaEnabled(uint16,bool)") => {
                Self::set_liquid_alpha_enabled(handle, &method_input)
            }

            id if id == get_method_id("getAlphaValues(uint16)") => {
                Self::get_alpha_values(&method_input)
            }
            id if id == get_method_id("setAlphaValues(uint16,uint16,uint16)") => {
                Self::set_alpha_values(handle, &method_input)
            }

            id if id == get_method_id("getCommitRevealWeightsInterval(uint16)") => {
                Self::get_commit_reveal_weights_interval(&method_input)
            }
            id if id == get_method_id("setCommitRevealWeightsInterval(uint16,uint64)") => {
                Self::set_commit_reveal_weights_interval(handle, &method_input)
            }
            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            }),
        }
    }

    fn register_network(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let call = match data.len() {
            32 => {
                let (hotkey, _) = get_pubkey(data)?;
                RuntimeCall::SubtensorModule(
                    pallet_subtensor::Call::<Runtime>::register_network_with_identity {
                        hotkey,
                        identity: None,
                    },
                )
            }
            33.. => {
                let (
                    hotkey,
                    subnet_name,
                    github_repo,
                    subnet_contact,
                    subnet_url,
                    discord,
                    description,
                    additional,
                ) = Self::parse_register_network_parameters(data)?;

                let identity: pallet_subtensor::SubnetIdentityOfV2 =
                    pallet_subtensor::SubnetIdentityOfV2 {
                        subnet_name,
                        github_repo,
                        subnet_contact,
                        subnet_url,
                        discord,
                        description,
                        additional,
                    };

                // Create the register_network callcle
                RuntimeCall::SubtensorModule(
                    pallet_subtensor::Call::<Runtime>::register_network_with_identity {
                        hotkey,
                        identity: Some(identity),
                    },
                )
            }
            _ => {
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::InvalidRange,
                });
            }
        };

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_serving_rate_limit(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::ServingRateLimit::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_serving_rate_limit(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, serving_rate_limit) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_serving_rate_limit {
                netuid,
                serving_rate_limit,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_min_difficulty(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::MinDifficulty::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_min_difficulty(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, min_difficulty) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_min_difficulty {
                netuid,
                min_difficulty,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_max_difficulty(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::MaxDifficulty::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_max_difficulty(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, max_difficulty) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_max_difficulty {
                netuid,
                max_difficulty,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_weights_version_key(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::WeightsVersionKey::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_weights_version_key(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, weights_version_key) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_weights_version_key {
                netuid,
                weights_version_key,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_weights_set_rate_limit(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::WeightsSetRateLimit::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_weights_set_rate_limit(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, weights_set_rate_limit) = Self::parse_netuid_u64_parameter(data)?;

        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_weights_set_rate_limit {
                netuid,
                weights_set_rate_limit,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_adjustment_alpha(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::AdjustmentAlpha::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_adjustment_alpha(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, adjustment_alpha) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_adjustment_alpha {
                netuid,
                adjustment_alpha,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_max_weight_limit(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::MaxWeightsLimit::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_max_weight_limit(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, max_weight_limit) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_max_weight_limit {
                netuid,
                max_weight_limit,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_immunity_period(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::ImmunityPeriod::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_immunity_period(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, immunity_period) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_immunity_period {
                netuid,
                immunity_period,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_min_allowed_weights(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::MinAllowedWeights::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_min_allowed_weights(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, min_allowed_weights) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_min_allowed_weights {
                netuid,
                min_allowed_weights,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_kappa(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::Kappa::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_kappa(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, kappa) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_kappa {
            netuid,
            kappa,
        });

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_rho(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::Rho::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_rho(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, rho) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_rho {
            netuid,
            rho,
        });

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_activity_cutoff(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::ActivityCutoff::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_activity_cutoff(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, activity_cutoff) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_activity_cutoff {
                netuid,
                activity_cutoff,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_network_registration_allowed(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::NetworkRegistrationAllowed::<Runtime>::get(netuid);

        let value_u256 = if value { U256::from(1) } else { U256::from(0) };
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_network_registration_allowed(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, registration_allowed) = Self::parse_netuid_bool_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_network_registration_allowed {
                netuid,
                registration_allowed,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_network_pow_registration_allowed(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::NetworkPowRegistrationAllowed::<Runtime>::get(netuid);

        let value_u256 = if value { U256::from(1) } else { U256::from(0) };
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_network_pow_registration_allowed(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, registration_allowed) = Self::parse_netuid_bool_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_network_pow_registration_allowed {
                netuid,
                registration_allowed,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_min_burn(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::MinBurn::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_min_burn(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, min_burn) = Self::parse_netuid_u64_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_min_burn {
                netuid,
                min_burn,
            });

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_max_burn(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::MaxBurn::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_max_burn(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, max_burn) = Self::parse_netuid_u64_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_max_burn {
                netuid,
                max_burn,
            });

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_difficulty(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::Difficulty::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_difficulty(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, difficulty) = Self::parse_netuid_u64_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_difficulty {
                netuid,
                difficulty,
            });

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_bonds_moving_average(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::BondsMovingAverage::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_bonds_moving_average(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, bonds_moving_average) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_bonds_moving_average {
                netuid,
                bonds_moving_average,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_commit_reveal_weights_enabled(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::CommitRevealWeightsEnabled::<Runtime>::get(netuid);

        let value_u256 = if value { U256::from(1) } else { U256::from(0) };
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_commit_reveal_weights_enabled(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, enabled) = Self::parse_netuid_bool_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_commit_reveal_weights_enabled {
                netuid,
                enabled,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_liquid_alpha_enabled(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::LiquidAlphaOn::<Runtime>::get(netuid);

        let value_u256 = if value { U256::from(1) } else { U256::from(0) };
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_liquid_alpha_enabled(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, enabled) = Self::parse_netuid_bool_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_liquid_alpha_enabled { netuid, enabled },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_alpha_values(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let (alpha_low, alpha_high) = pallet_subtensor::AlphaValues::<Runtime>::get(netuid);

        let mut value_u256 = U256::from(alpha_low);
        let mut result = [0_u8; 64];
        U256::to_big_endian(&value_u256, &mut result[0..]);

        value_u256 = U256::from(alpha_high);
        U256::to_big_endian(&value_u256, &mut result[32..]);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_alpha_values(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, alpha_low, alpha_high) = Self::parse_netuid_u16_u16_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_alpha_values {
                netuid,
                alpha_low,
                alpha_high,
            });

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn get_commit_reveal_weights_interval(data: &[u8]) -> PrecompileResult {
        let netuid = parse_netuid(data, 30)?;

        let value = pallet_subtensor::RevealPeriodEpochs::<Runtime>::get(netuid);

        let value_u256 = U256::from(value);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&value_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn set_commit_reveal_weights_interval(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, interval) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_commit_reveal_weights_interval {
                netuid,
                interval,
            },
        );

        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn parse_register_network_parameters(
        data: &[u8],
    ) -> Result<
        (
            AccountId32,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
        ),
        PrecompileFailure,
    > {
        let (pubkey, dynamic_params) = get_pubkey(data)?;
        let dynamic_data_len = dynamic_params.len();

        let mut buf = [0_u8; 4];
        // get all start points for the data items: name, repo, contact, url, discord, description, additional
        buf.copy_from_slice(get_slice(data, 60, 64)?);
        let subnet_name_start: usize = u32::from_be_bytes(buf) as usize;
        if subnet_name_start > dynamic_data_len {
            log::error!(
                "the start position of subnet name as {} is too big ",
                subnet_name_start
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        buf.copy_from_slice(get_slice(data, 92, 96)?);
        let github_repo_start: usize = u32::from_be_bytes(buf) as usize;
        if github_repo_start > dynamic_data_len {
            log::error!(
                "the start position of github repo as {} is too big ",
                github_repo_start
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        buf.copy_from_slice(get_slice(data, 124, 128)?);
        let subnet_contact_start: usize = u32::from_be_bytes(buf) as usize;
        if subnet_contact_start > dynamic_data_len {
            log::error!(
                "the start position of subnet contact as {} is too big ",
                subnet_contact_start
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        buf.copy_from_slice(get_slice(data, 156, 160)?);
        let subnet_url_start: usize = u32::from_be_bytes(buf) as usize;
        if subnet_url_start > dynamic_data_len {
            log::error!(
                "the start position of subnet_url as {} is too big ",
                subnet_url_start
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        buf.copy_from_slice(get_slice(data, 188, 192)?);
        let discord_start: usize = u32::from_be_bytes(buf) as usize;
        if discord_start > dynamic_data_len {
            log::error!(
                "the start position of discord as {} is too big ",
                discord_start
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        buf.copy_from_slice(get_slice(data, 220, 224)?);
        let description_start: usize = u32::from_be_bytes(buf) as usize;
        if description_start > dynamic_data_len {
            log::error!(
                "the start position of description as {} is too big ",
                description_start
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        buf.copy_from_slice(get_slice(data, 252, 256)?);
        let additional_start: usize = u32::from_be_bytes(buf) as usize;
        if additional_start > dynamic_data_len {
            log::error!(
                "the start position of additional as {} is too big ",
                additional_start
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        // get name
        buf.copy_from_slice(get_slice(
            data,
            subnet_name_start + 28,
            subnet_name_start + 32,
        )?);
        let subnet_name_len: usize = u32::from_be_bytes(buf) as usize;

        if subnet_name_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!(
                "the length of subnet name as {} is too big",
                subnet_name_len
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut name_vec = vec![0; subnet_name_len];
        name_vec.copy_from_slice(get_slice(
            data,
            subnet_name_start + 32,
            subnet_name_start + subnet_name_len + 32,
        )?);

        // get repo data
        buf.copy_from_slice(get_slice(
            data,
            github_repo_start + 28,
            github_repo_start + 32,
        )?);
        let github_repo_len: usize = u32::from_be_bytes(buf) as usize;
        if github_repo_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!(
                "the length of github repo as {} is too big",
                github_repo_len
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut repo_vec = vec![0; github_repo_len];
        repo_vec.copy_from_slice(get_slice(
            data,
            github_repo_start + 32,
            github_repo_start + github_repo_len + 32,
        )?);

        // get contact data
        buf.copy_from_slice(get_slice(
            data,
            subnet_contact_start + 28,
            subnet_contact_start + 32,
        )?);
        let subnet_contact_len: usize = u32::from_be_bytes(buf) as usize;
        if subnet_contact_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!(
                "the length of subnet contact as {} is too big",
                subnet_contact_len
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut contact_vec = vec![0; subnet_contact_len];
        contact_vec.copy_from_slice(get_slice(
            data,
            subnet_contact_start + 32,
            subnet_contact_start + subnet_contact_len + 32,
        )?);

        // get subnet_url
        buf.copy_from_slice(get_slice(
            data,
            subnet_url_start + 28,
            subnet_url_start + 32,
        )?);
        let subnet_url_len: usize = u32::from_be_bytes(buf) as usize;
        if subnet_url_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!("the length of subnet_url as {} is too big", subnet_url_len);
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut url_vec = vec![0; subnet_url_len];
        url_vec.copy_from_slice(get_slice(
            data,
            subnet_url_start + 32,
            subnet_url_start + subnet_url_len + 32,
        )?);

        // get discord
        buf.copy_from_slice(get_slice(data, discord_start + 28, discord_start + 32)?);
        let discord_len: usize = u32::from_be_bytes(buf) as usize;
        if discord_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!("the length of discord as {} is too big", discord_len);
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut discord_vec = vec![0; discord_len];
        discord_vec.copy_from_slice(get_slice(
            data,
            discord_start + 32,
            discord_start + discord_len + 32,
        )?);

        // get description
        buf.copy_from_slice(get_slice(
            data,
            description_start + 28,
            description_start + 32,
        )?);
        let description_len: usize = u32::from_be_bytes(buf) as usize;
        if description_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!(
                "the length of description as {} is too big",
                description_len
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut description_vec = vec![0; description_len];
        description_vec.copy_from_slice(get_slice(
            data,
            description_start + 32,
            description_start + description_len + 32,
        )?);

        // get additional
        buf.copy_from_slice(get_slice(
            data,
            additional_start + 28,
            additional_start + 32,
        )?);
        let additional_len: usize = u32::from_be_bytes(buf) as usize;
        if additional_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!("the length of additional as {} is too big", additional_len);
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut additional_vec = vec![0; additional_len];
        additional_vec.copy_from_slice(get_slice(
            data,
            additional_start + 32,
            additional_start + additional_len + 32,
        )?);

        Ok((
            pubkey,
            name_vec,
            repo_vec,
            contact_vec,
            url_vec,
            discord_vec,
            description_vec,
            additional_vec,
        ))
    }

    fn parse_netuid_u64_parameter(data: &[u8]) -> Result<(u16, u64), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let netuid = parse_netuid(data, 30)?;

        let mut parameter_vec = [0u8; 8];
        parameter_vec.copy_from_slice(get_slice(data, 56, 64)?);
        let parameter = u64::from_be_bytes(parameter_vec);

        Ok((netuid, parameter))
    }

    fn parse_netuid_u16_parameter(data: &[u8]) -> Result<(u16, u16), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let netuid = parse_netuid(data, 30)?;

        let mut parameter_vec = [0u8; 2];
        parameter_vec.copy_from_slice(get_slice(data, 62, 64)?);
        let parameter = u16::from_be_bytes(parameter_vec);

        Ok((netuid, parameter))
    }

    fn parse_netuid_u16_u16_parameter(data: &[u8]) -> Result<(u16, u16, u16), PrecompileFailure> {
        if data.len() < 96 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let netuid = parse_netuid(data, 30)?;

        let mut parameter_1_vec = [0u8; 2];
        parameter_1_vec.copy_from_slice(get_slice(data, 62, 64)?);
        let parameter_1 = u16::from_be_bytes(parameter_1_vec);

        let mut parameter_2_vec = [0u8; 2];
        parameter_2_vec.copy_from_slice(get_slice(data, 94, 96)?);
        let parameter_2 = u16::from_be_bytes(parameter_2_vec);

        Ok((netuid, parameter_1, parameter_2))
    }

    fn parse_netuid_bool_parameter(data: &[u8]) -> Result<(u16, bool), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let netuid = parse_netuid(data, 30)?;

        let mut parameter_vec = [0_u8];
        parameter_vec.copy_from_slice(get_slice(data, 63, 64)?);

        let parameter = parameter_vec[0] != 0;

        Ok((netuid, parameter))
    }
}
