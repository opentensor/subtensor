use pallet_evm::{
    AddressMapping, ExitError, HashedAddressMapping, PrecompileFailure, PrecompileHandle,
    PrecompileResult,
};

use crate::precompiles::{
    get_method_id, get_pubkey, get_single_u8, get_slice, parse_netuid, try_dispatch_runtime_call,
};
use crate::{Runtime, RuntimeCall};
use frame_system::RawOrigin;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::AccountId32;
use sp_std::vec;
use sp_std::vec::Vec;

pub const NEURON_PRECOMPILE_INDEX: u64 = 2052;
// max paramter lenght 4K
pub const MAX_PARAMETER_SIZE: usize = 4 * 1024;
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

        if method_input.len() > MAX_PARAMETER_SIZE {
            log::error!(
                "method parameter data length as {} is too long",
                method_input.len()
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        match method_id {
            id if id == get_method_id("setWeights(uint16,uint16[],uint16[],uint64)") => {
                Self::set_weights(handle, &method_input)
            }
            id if id == get_method_id("commitWeights(uint16,uint256)") => {
                Self::commit_weights(handle, &method_input)
            }
            id if id
                == get_method_id("revealWeights(uint16,uint16[],uint16[],uint16[],uint64)") =>
            {
                Self::reveal_weights(handle, &method_input)
            }

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

    pub fn set_weights(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, dests, weights, version_key) = Self::parse_netuid_dests_weights(data)?;
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::set_weights {
            netuid,
            dests,
            weights,
            version_key,
        });
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    pub fn commit_weights(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, commit_hash) = Self::parse_netuid_commit_hash(data)?;

        let call =
            RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::commit_weights {
                netuid,
                commit_hash,
            });
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );
        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    pub fn reveal_weights(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, uids, values, salt, version_key) =
            Self::parse_netuid_dests_weights_salt(data)?;
        let call =
            RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::reveal_weights {
                netuid,
                uids,
                values,
                salt,
                version_key,
            });
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );
        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
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

    fn parse_netuid_dests_weights(
        data: &[u8],
    ) -> Result<(u16, Vec<u16>, Vec<u16>, u64), PrecompileFailure> {
        let data_len = data.len();
        if data_len < 4 * 32 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        // get the neuron amount in sebnet
        let subnet_size = pallet_subtensor::Pallet::<Runtime>::get_subnetwork_n(netuid) as usize;

        let mut first_position_vec = [0u8; 2];
        first_position_vec.copy_from_slice(get_slice(data, 62, 64)?);
        let first_position = u16::from_be_bytes(first_position_vec) as usize;

        if first_position > data_len {
            log::error!("position for uids data as {} is too large", first_position);
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut second_position_vec = [0u8; 2];
        second_position_vec.copy_from_slice(get_slice(data, 94, 96)?);
        let second_position = u16::from_be_bytes(second_position_vec) as usize;

        if second_position > data_len {
            log::error!("position for uids data as {} is too large", first_position);
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut version_key_vec = [0u8; 8];
        version_key_vec.copy_from_slice(get_slice(data, 120, 128)?);
        let version_key = u64::from_be_bytes(version_key_vec);

        let mut dests = vec![];
        let mut weights = vec![];

        let mut dests_len_vec = [0u8; 2];
        dests_len_vec.copy_from_slice(get_slice(data, first_position + 30, first_position + 32)?);
        let dests_len = u16::from_be_bytes(dests_len_vec) as usize;

        if dests_len > subnet_size {
            log::error!(
                "uids len as {} in set weight is more than neurons {} in subnet {}",
                dests_len,
                subnet_size,
                netuid
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        for i in 0..dests_len {
            let mut tmp_vec = [0u8; 2];
            let from = first_position
                .saturating_add(62)
                .saturating_add(i.saturating_mul(32));
            let to = from.saturating_add(2);
            tmp_vec.copy_from_slice(get_slice(data, from, to)?);
            let dest = u16::from_be_bytes(tmp_vec);
            dests.push(dest);
        }

        let mut weights_len_vec = [0u8; 2];
        weights_len_vec.copy_from_slice(get_slice(
            data,
            second_position + 30,
            second_position + 32,
        )?);
        let weights_len = u16::from_be_bytes(weights_len_vec) as usize;

        if weights_len > subnet_size {
            log::error!(
                "weights len as {} in set weight is more than neurons {} in subnet {}",
                weights_len,
                subnet_size,
                netuid
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        for i in 0..weights_len {
            let mut tmp_vec = [0u8; 2];
            let from = second_position
                .saturating_add(62)
                .saturating_add(i.saturating_mul(32));
            let to = from.saturating_add(2);
            tmp_vec.copy_from_slice(get_slice(data, from, to)?);
            let weight = u16::from_be_bytes(tmp_vec);
            weights.push(weight);
        }

        Ok((netuid, dests, weights, version_key))
    }

    fn parse_netuid_commit_hash(data: &[u8]) -> Result<(u16, H256), PrecompileFailure> {
        if data.len() < 2 * 32 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);
        let commit_hash = H256::from_slice(get_slice(data, 32, 64)?);

        Ok((netuid, commit_hash))
    }

    fn parse_netuid_dests_weights_salt(
        data: &[u8],
    ) -> Result<(u16, Vec<u16>, Vec<u16>, Vec<u16>, u64), PrecompileFailure> {
        let data_len = data.len();
        if data_len < 5 * 32 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let netuid = parse_netuid(data, 30)?;

        // get the neuron amount in sebnet
        let subnet_size = pallet_subtensor::Pallet::<Runtime>::get_subnetwork_n(netuid) as usize;

        let mut first_position_vec = [0u8; 2];
        first_position_vec.copy_from_slice(get_slice(data, 62, 64)?);
        let first_position = u16::from_be_bytes(first_position_vec) as usize;

        if first_position > data_len {
            log::error!("position for uids data as {} is too large", first_position);
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut second_position_vec = [0u8; 2];
        second_position_vec.copy_from_slice(get_slice(data, 94, 96)?);
        let second_position = u16::from_be_bytes(second_position_vec) as usize;

        if second_position > data_len {
            log::error!(
                "position for values data as {} is too large",
                first_position
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut third_position_vec = [0u8; 2];
        third_position_vec.copy_from_slice(get_slice(data, 126, 128)?);
        let third_position = u16::from_be_bytes(third_position_vec) as usize;

        if third_position > data_len {
            log::error!("position for salt data as {} is too large", first_position);
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut version_key_vec = [0u8; 8];
        version_key_vec.copy_from_slice(get_slice(data, 152, 160)?);
        let version_key = u64::from_be_bytes(version_key_vec);

        let mut uids = vec![];
        let mut values = vec![];
        let mut salt = vec![];

        let mut uids_len_vec = [0u8; 2];
        uids_len_vec.copy_from_slice(get_slice(data, first_position + 30, first_position + 32)?);
        let uids_len = u16::from_be_bytes(uids_len_vec) as usize;

        if uids_len > subnet_size {
            log::error!(
                "uids len as {} in reveal weight is more than neurons {} in subnet {}",
                uids_len,
                subnet_size,
                netuid
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        for i in 0..uids_len {
            let mut tmp_vec = [0u8; 2];
            let from = first_position
                .saturating_add(62)
                .saturating_add(i.saturating_mul(32));
            let to = from.saturating_add(2);
            tmp_vec.copy_from_slice(get_slice(data, from, to)?);
            let uid = u16::from_be_bytes(tmp_vec);
            uids.push(uid);
        }

        let mut values_len_vec = [0u8; 2];
        values_len_vec.copy_from_slice(get_slice(
            data,
            second_position + 30,
            second_position + 32,
        )?);
        let values_len = u16::from_be_bytes(values_len_vec) as usize;

        if values_len > subnet_size {
            log::error!(
                "values len as {} in reveal weight is more than neurons {} in subnet {}",
                values_len,
                subnet_size,
                netuid
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        for i in 0..values_len {
            let mut tmp_vec = [0u8; 2];
            let from = second_position
                .saturating_add(62)
                .saturating_add(i.saturating_mul(32));
            let to = from.saturating_add(2);
            tmp_vec.copy_from_slice(get_slice(data, from, to)?);
            let value = u16::from_be_bytes(tmp_vec);
            values.push(value);
        }

        let mut salt_len_vec = [0u8; 2];
        salt_len_vec.copy_from_slice(get_slice(data, third_position + 30, third_position + 32)?);
        let salt_len = u16::from_be_bytes(salt_len_vec) as usize;

        if salt_len > subnet_size {
            log::error!(
                "salt len as {} in reveal weight is more than neurons {} in subnet {}",
                salt_len,
                subnet_size,
                netuid
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        for i in 0..salt_len {
            let mut tmp_vec = [0u8; 2];
            let from = third_position
                .saturating_add(62)
                .saturating_add(i.saturating_mul(32));
            let to = from.saturating_add(2);
            tmp_vec.copy_from_slice(get_slice(data, from, to)?);
            let value = u16::from_be_bytes(tmp_vec);
            salt.push(value);
        }

        Ok((netuid, uids, values, salt, version_key))
    }

    fn parse_serve_axon_parameters(
        data: &[u8],
    ) -> Result<(u16, u32, u128, u16, u8, u8, u8, u8), PrecompileFailure> {
        if data.len() < 256 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let netuid = parse_netuid(data, 30)?;

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
        let data_len = data.len();
        if data_len < 288 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let netuid = parse_netuid(data, 30)?;

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

        if len_position > data_len {
            log::error!(
                "the start position of certificate as {} is bigger than whole data len {}",
                len_position,
                data_len
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut len_vec = [0u8; 2];
        len_vec.copy_from_slice(get_slice(data, len_position + 30, len_position + 32)?);
        let vec_len = u16::from_be_bytes(len_vec) as usize;

        let vec_result = get_slice(
            data,
            len_position + 32,
            len_position.saturating_add(32).saturating_add(vec_len),
        )?
        .to_vec();

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

        let netuid = parse_netuid(data, 30)?;

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
