use crate::precompiles::{get_method_id, get_pubkey, get_slice, try_dispatch_runtime_call};
use crate::{Runtime, RuntimeCall};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, ExitError, HashedAddressMapping, PrecompileFailure, PrecompileHandle,
    PrecompileResult,
};
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::AccountId32;
use sp_std::vec;
pub const SUBNET_PRECOMPILE_INDEX: u64 = 2051;
// bytes with max lenght 1K
pub const MAX_SINGLE_PARAMETER_SIZE: usize = 1024;
// three bytes with max lenght 1K
pub const MAX_PARAMETER_SIZE: usize = 3 * MAX_SINGLE_PARAMETER_SIZE;

// ss58 public key i.e., the contract sends funds it received to the destination address from the
// method parameter.
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
            id if id == get_method_id("registerNetwork(bytes32,bytes,bytes,bytes)") => {
                Self::register_network(handle, &method_input)
            }
            id if id == get_method_id("registerNetwork(bytes32)") => {
                Self::register_network(handle, &method_input)
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
                let (hotkey, subnet_name, github_repo, subnet_contact) =
                    Self::parse_register_network_parameters(data)?;

                let identity: pallet_subtensor::SubnetIdentityOf =
                    pallet_subtensor::SubnetIdentityOf {
                        subnet_name,
                        github_repo,
                        subnet_contact,
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

        // Dispatch the register_network call
        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    fn parse_register_network_parameters(
        data: &[u8],
    ) -> Result<(AccountId32, vec::Vec<u8>, vec::Vec<u8>, vec::Vec<u8>), PrecompileFailure> {
        let (pubkey, dynamic_params) = get_pubkey(data)?;
        let dynamic_data_len = dynamic_params.len();

        let mut buf = [0_u8; 4];
        // get all start point for three data items: name, repo and contact
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

        // get name
        buf.copy_from_slice(get_slice(
            data,
            subnet_name_start + 28,
            subnet_name_start + 32,
        )?);
        let subnet_name_len: usize = u32::from_be_bytes(buf) as usize;

        if subnet_name_len > MAX_SINGLE_PARAMETER_SIZE {
            log::error!("the length of subnet nae as {} is too big", subnet_name_len);
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

        Ok((pubkey, name_vec, repo_vec, contact_vec))
    }
}
