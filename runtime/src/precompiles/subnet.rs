use crate::precompiles::{dispatch, get_method_id, get_slice};
use crate::{Runtime, RuntimeCall};
use pallet_evm::{ExitError, PrecompileFailure, PrecompileHandle, PrecompileResult};
use sp_std::vec;

pub const SUBNET_PRECOMPILE_INDEX: u64 = 2051;
// three bytes with max lenght 1K
pub const MAX_PARAMETER_SIZE: usize = 3 * 1024;

// this is staking smart contract's(0x0000000000000000000000000000000000000803) sr25519 address
pub const STAKING_CONTRACT_ADDRESS: &str = "5DPSUCb5mZFfizvBDSnRoAqmxV5Bmov2CS3xV773qU6VP1w2";

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
            id if id == get_method_id("registerNetwork(bytes,bytes,bytes)") => {
                Self::register_network(handle, &method_input)
            }
            id if id == get_method_id("registerNetwork()") => {
                Self::register_network(handle, &[0_u8; 0])
            }
            id if id == get_method_id("setServingRateLimit(uint16,uint64)") => {
                Self::set_serving_rate_limit(handle, &method_input)
            }
            id if id == get_method_id("setMinDifficulty(uint16,uint64)") => {
                Self::set_min_difficulty(handle, &method_input)
            }
            id if id == get_method_id("setMaxDifficulty(uint16,uint64)") => {
                Self::set_max_difficulty(handle, &method_input)
            }
            id if id == get_method_id("setWeightsVersionKey(uint16,uint64)") => {
                Self::set_weights_version_key(handle, &method_input)
            }
            id if id == get_method_id("setWeightsSetRateLimit(uint16,uint64)") => {
                Self::set_weights_set_rate_limit(handle, &method_input)
            }
            id if id == get_method_id("setServingRateLimit(uint16,uint64)") => {
                Self::set_adjustment_alpha(handle, &method_input)
            }
            id if id == get_method_id("setServingRateLimit(uint16,uint64)") => {
                Self::set_max_weight_limit(handle, &method_input)
            }
            id if id == get_method_id("setServingRateLimit(uint16,uint64)") => {
                Self::set_immunity_period(handle, &method_input)
            }
            id if id == get_method_id("setMinAllowedWeights(uint16,uint16)") => {
                Self::set_min_allowed_weights(handle, &method_input)
            }
            id if id == get_method_id("setKappa(uint16,uint16)") => {
                Self::set_kappa(handle, &method_input)
            }
            id if id == get_method_id("setRho(uint16,uint16)") => {
                Self::set_rho(handle, &method_input)
            }
            id if id == get_method_id("setActivityCutoff(uint16,uint16)") => {
                Self::set_activity_cutoff(handle, &method_input)
            }
            id if id == get_method_id("set_NetworkRegistrationAllowed(uint16,bool)") => {
                Self::set_network_registration_allowed(handle, &method_input)
            }
            id if id == get_method_id("setNetworkPowRegistrationAllowed(uint16,bool)") => {
                Self::set_network_pow_registration_allowed(handle, &method_input)
            }
            id if id == get_method_id("setMinBurn(uint16,uint64)") => {
                Self::set_min_burn(handle, &method_input)
            }
            id if id == get_method_id("setMaxBurn(uint16,uint64)") => {
                Self::set_max_burn(handle, &method_input)
            }
            id if id == get_method_id("setDifficulty(uint16,uint64)") => {
                Self::set_difficulty(handle, &method_input)
            }
            id if id == get_method_id("setBondsMovingAverage(uint16,uint64)") => {
                Self::set_bonds_moving_average(handle, &method_input)
            }
            id if id == get_method_id("setCommitRevealWeightsEnabled(uint16,bool)") => {
                Self::set_commit_reveal_weights_enabled(handle, &method_input)
            }
            id if id == get_method_id("setLiquidAlphaEnabled(uint16,bool)") => {
                Self::set_liquid_alpha_enabled(handle, &method_input)
            }
            id if id == get_method_id("setAlphaValues(uint16,uint16,uint16)") => {
                Self::set_alpha_values(handle, &method_input)
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
        let call = if data.is_empty() {
            RuntimeCall::SubtensorModule(
                pallet_subtensor::Call::<Runtime>::register_network_with_identity {
                    identity: None,
                },
            )
        } else {
            let (subnet_name, github_repo, subnet_contact) =
                Self::parse_register_network_parameters(data)?;

            let identity: pallet_subtensor::SubnetIdentityOf = pallet_subtensor::SubnetIdentityOf {
                subnet_name,
                github_repo,
                subnet_contact,
            };

            // Create the register_network callcle
            RuntimeCall::SubtensorModule(
                pallet_subtensor::Call::<Runtime>::register_network_with_identity {
                    identity: Some(identity),
                },
            )
        };

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_serving_rate_limit(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, serving_rate_limit) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_serving_rate_limit {
                netuid,
                serving_rate_limit,
            },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_min_difficulty(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, min_difficulty) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_min_difficulty {
                netuid,
                min_difficulty,
            },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_max_difficulty(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, max_difficulty) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_max_difficulty {
                netuid,
                max_difficulty,
            },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_adjustment_alpha(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, adjustment_alpha) = Self::parse_netuid_u64_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_adjustment_alpha {
                netuid,
                adjustment_alpha,
            },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_max_weight_limit(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, max_weight_limit) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_max_weight_limit {
                netuid,
                max_weight_limit,
            },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_immunity_period(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, immunity_period) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_immunity_period {
                netuid,
                immunity_period,
            },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_kappa(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, kappa) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_kappa {
            netuid,
            kappa,
        });

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_rho(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, rho) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_rho {
            netuid,
            rho,
        });

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_activity_cutoff(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, activity_cutoff) = Self::parse_netuid_u16_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_activity_cutoff {
                netuid,
                activity_cutoff,
            },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_min_burn(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, min_burn) = Self::parse_netuid_u64_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_min_burn {
                netuid,
                min_burn,
            });

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_max_burn(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, max_burn) = Self::parse_netuid_u64_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_max_burn {
                netuid,
                max_burn,
            });

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_difficulty(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, difficulty) = Self::parse_netuid_u64_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_difficulty {
                netuid,
                difficulty,
            });

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_liquid_alpha_enabled(
        handle: &mut impl PrecompileHandle,
        data: &[u8],
    ) -> PrecompileResult {
        let (netuid, enabled) = Self::parse_netuid_bool_parameter(data)?;
        let call = RuntimeCall::AdminUtils(
            pallet_admin_utils::Call::<Runtime>::sudo_set_liquid_alpha_enabled { netuid, enabled },
        );

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn set_alpha_values(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, alpha_low, alpha_high) = Self::parse_netuid_u16_u16_parameter(data)?;
        let call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::<Runtime>::sudo_set_alpha_values {
                netuid,
                alpha_low,
                alpha_high,
            });

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
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

        dispatch(handle, call, STAKING_CONTRACT_ADDRESS)
    }

    fn parse_register_network_parameters(
        data: &[u8],
    ) -> Result<(vec::Vec<u8>, vec::Vec<u8>, vec::Vec<u8>), PrecompileFailure> {
        let mut buf = [0_u8; 4];

        // get all start point for three data items: name, repo and contact
        buf.copy_from_slice(get_slice(data, 28, 32)?);
        let subnet_name_start: usize = u32::from_be_bytes(buf) as usize;

        buf.copy_from_slice(get_slice(data, 60, 64)?);
        let github_repo_start: usize = u32::from_be_bytes(buf) as usize;

        buf.copy_from_slice(get_slice(data, 92, 96)?);
        let subnet_contact_start: usize = u32::from_be_bytes(buf) as usize;

        // get name
        buf.copy_from_slice(get_slice(
            data,
            subnet_name_start + 28,
            subnet_name_start + 32,
        )?);
        let subnet_name_len: usize = u32::from_be_bytes(buf) as usize;

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

        let mut contact_vec = vec![0; subnet_contact_len];
        contact_vec.copy_from_slice(get_slice(
            data,
            subnet_contact_start + 32,
            subnet_contact_start + subnet_contact_len + 32,
        )?);

        Ok((name_vec, repo_vec, contact_vec))
    }

    fn parse_netuid_u64_parameter(data: &[u8]) -> Result<(u16, u64), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

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
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut parameter_vec = [0u8; 2];
        parameter_vec.copy_from_slice(get_slice(data, 62, 64)?);
        let parameter = u16::from_be_bytes(parameter_vec);

        Ok((netuid, parameter))
    }

    fn parse_netuid_u16_u16_parameter(data: &[u8]) -> Result<(u16, u16, u16), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

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
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut parameter_vec = [0_u8];
        parameter_vec.copy_from_slice(get_slice(data, 63, 64)?);

        let parameter = parameter_vec[0] != 0;

        Ok((netuid, parameter))
    }
}
