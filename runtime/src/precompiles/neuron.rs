use pallet_evm::{
    AddressMapping, ExitError, HashedAddressMapping, PrecompileFailure, PrecompileHandle,
    PrecompileResult,
};

use crate::precompiles::{
    get_method_id, get_pubkey, get_single_u8, get_slice, parse_netuid, try_dispatch_runtime_call,
};
use crate::{Runtime, RuntimeCall};
use frame_system::RawOrigin;
use sp_runtime::{traits::BlakeTwo256, AccountId32};
use sp_std::vec;
pub const NEURON_PRECOMPILE_INDEX: u64 = 2052;

// ss58 public key i.e., the contract sends funds it received to the destination address from the
// method parameter.
#[allow(dead_code)]
const CONTRACT_ADDRESS_SS58: [u8; 32] = [
    0xbc, 0x46, 0x35, 0x79, 0xbc, 0x99, 0xf9, 0xee, 0x7c, 0x59, 0xed, 0xee, 0x20, 0x61, 0xa3, 0x09,
    0xd2, 0x1e, 0x68, 0xd5, 0x39, 0xb6, 0x40, 0xec, 0x66, 0x46, 0x90, 0x30, 0xab, 0x74, 0xc1, 0xdb,
];
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
                hotkey,
            });
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        // Dispatch the register_network call
        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
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
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        // Dispatch the register_network call
        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
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
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        // Dispatch the register_network call
        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
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
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        // Dispatch the register_network call
        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn parse_netuid_hotkey_parameter(data: &[u8]) -> Result<(u16, AccountId32), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let netuid = parse_netuid(data, 30)?;

        let (hotkey, _) = get_pubkey(get_slice(data, 32, 64)?)?;

        Ok((netuid, hotkey))
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

        let ip_type = get_single_u8(data, 159)?;
        let protocol = get_single_u8(data, 191)?;
        let placeholder1 = get_single_u8(data, 223)?;
        let placeholder2 = get_single_u8(data, 255)?;
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

        let ip_type = get_single_u8(data, 159)?;
        let protocol = get_single_u8(data, 191)?;
        let placeholder1 = get_single_u8(data, 223)?;
        let placeholder2 = get_single_u8(data, 255)?;

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

        let ip_type = get_single_u8(data, 159)?;
        Ok((netuid, version, ip, port, ip_type))
    }
}
