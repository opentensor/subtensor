use pallet_evm::{ExitError, PrecompileFailure, PrecompileHandle, PrecompileResult};

use crate::precompiles::{dispatch, get_method_id, get_slice};
use sp_std::vec;

use crate::{Runtime, RuntimeCall};
pub const NEURON_PRECOMPILE_INDEX: u64 = 2052;

// this is neuron smart contract's(0x0000000000000000000000000000000000000804) sr25519 address
pub const NEURON_CONTRACT_ADDRESS: &str = "5GKZiUUgTnWSz3BgiVBMehEKkLszsG4ZXnvgWpWFUFKqrqyn";

pub struct NeuronPrecompile;

impl NeuronPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata
            .get(4..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

        match method_id {
            id if id == get_method_id("burnedRegister(uint16,bytes32)") => {
                Self::burned_register(handle, &method_input)
            }
            id if id
                == get_method_id(
                    "serveAxon(uint16,uint32,uint128,uint16,uint8,uint8,uint8,uint8)",
                ) =>
            {
                Self::serve_axon(handle, &method_input)
            }
            id if id
                == get_method_id(
                    "serveAxonTls(uint16,uint32,uint128,uint16,uint8,uint8,uint8,uint8,bytes)",
                ) =>
            {
                Self::serve_axon_tls(handle, &method_input)
            }
            id if id == get_method_id("servePrometheus(uint16,uint32,uint128,uint16,uint8)") => {
                Self::serve_prometheus(handle, &method_input)
            }

            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            }),
        }
    }

    pub fn burned_register(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, hotkey) = Self::parse_netuid_hotkey_parameter(data)?;
        let call =
            RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::burned_register {
                netuid,
                hotkey: hotkey.into(),
            });
        dispatch(handle, call, NEURON_CONTRACT_ADDRESS)
    }

    pub fn serve_axon(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, version, ip, port, ip_type, protocol, placeholder1, placeholder2) =
            Self::parse_serve_axon_parameters(data)?;
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::serve_axon {
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
        });
        dispatch(handle, call, NEURON_CONTRACT_ADDRESS)
    }

    pub fn serve_axon_tls(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, version, ip, port, ip_type, protocol, placeholder1, placeholder2, certificate) =
            Self::parse_serve_axon_tls_parameters(data)?;
        let call =
            RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::serve_axon_tls {
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2,
                certificate,
            });
        dispatch(handle, call, NEURON_CONTRACT_ADDRESS)
    }

    pub fn serve_prometheus(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, version, ip, port, ip_type) = Self::parse_serve_prometheus_parameters(data)?;
        let call =
            RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::serve_prometheus {
                netuid,
                version,
                ip,
                port,
                ip_type,
            });
        dispatch(handle, call, NEURON_CONTRACT_ADDRESS)
    }

    fn parse_netuid_hotkey_parameter(data: &[u8]) -> Result<(u16, [u8; 32]), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut parameter = [0u8; 32];
        parameter.copy_from_slice(get_slice(data, 32, 64)?);

        Ok((netuid, parameter))
    }

    fn parse_serve_axon_parameters(
        data: &[u8],
    ) -> Result<(u16, u32, u128, u16, u8, u8, u8, u8), PrecompileFailure> {
        if data.len() < 256 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut version_vec = [0u8; 4];
        version_vec.copy_from_slice(get_slice(data, 60, 64)?);
        let version = u32::from_be_bytes(version_vec);

        let mut ip_vec = [0u8; 16];
        ip_vec.copy_from_slice(get_slice(data, 80, 96)?);
        let ip = u128::from_be_bytes(ip_vec);

        let mut port_vec = [0u8; 2];
        port_vec.copy_from_slice(get_slice(data, 126, 128)?);
        let port = u16::from_be_bytes(port_vec);

        let ip_type = data[159];
        let protocol = data[191];
        let placeholder1 = data[223];
        let placeholder2 = data[255];
        Ok((
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
        ))
    }

    fn parse_serve_axon_tls_parameters(
        data: &[u8],
    ) -> Result<(u16, u32, u128, u16, u8, u8, u8, u8, vec::Vec<u8>), PrecompileFailure> {
        if data.len() < 288 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut version_vec = [0u8; 4];
        version_vec.copy_from_slice(get_slice(data, 60, 64)?);
        let version = u32::from_be_bytes(version_vec);

        let mut ip_vec = [0u8; 16];
        ip_vec.copy_from_slice(get_slice(data, 80, 96)?);
        let ip = u128::from_be_bytes(ip_vec);

        let mut port_vec = [0u8; 2];
        port_vec.copy_from_slice(get_slice(data, 126, 128)?);
        let port = u16::from_be_bytes(port_vec);

        let ip_type = data[159];
        let protocol = data[191];
        let placeholder1 = data[223];
        let placeholder2 = data[255];

        let mut len_position_vec = [0u8; 2];
        len_position_vec.copy_from_slice(get_slice(data, 286, 288)?);
        let len_position = u16::from_be_bytes(len_position_vec) as usize;

        let mut len_vec = [0u8; 2];
        len_vec.copy_from_slice(get_slice(data, len_position + 30, len_position + 32)?);
        let vec_len = u16::from_be_bytes(len_vec) as usize;

        let vec_result = get_slice(data, len_position + 32, len_position + 32 + vec_len)?.to_vec();

        Ok((
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
            vec_result,
        ))
    }

    fn parse_serve_prometheus_parameters(
        data: &[u8],
    ) -> Result<(u16, u32, u128, u16, u8), PrecompileFailure> {
        if data.len() < 160 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut version_vec = [0u8; 4];
        version_vec.copy_from_slice(get_slice(data, 60, 64)?);
        let version = u32::from_be_bytes(version_vec);

        let mut ip_vec = [0u8; 16];
        ip_vec.copy_from_slice(get_slice(data, 80, 96)?);
        let ip = u128::from_be_bytes(ip_vec);

        let mut port_vec = [0u8; 2];
        port_vec.copy_from_slice(get_slice(data, 126, 128)?);
        let port = u16::from_be_bytes(port_vec);

        let ip_type = data[159];
        Ok((netuid, version, ip, port, ip_type))
    }
}
