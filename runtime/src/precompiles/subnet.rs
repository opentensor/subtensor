use crate::precompiles::{get_method_id, get_pubkey, get_slice, try_dispatch_runtime_call};
use crate::{Runtime, RuntimeCall};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, ExitError, HashedAddressMapping, PrecompileFailure, PrecompileHandle,
    PrecompileResult,
};
use sp_runtime::AccountId32;
use sp_runtime::{traits::BlakeTwo256, Vec};
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

        // Dispatch the register_network call
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
}
