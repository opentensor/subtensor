use core::marker::PhantomData;

use pallet_admin_utils::DeprecationRegistryStorage;
use pallet_evm::PrecompileHandle;
use precompile_utils::{EvmResult, prelude::{Address, UnboundedBytes}, solidity::Codec};
use sp_core::{H160, H256};

use crate::PrecompileExt;

pub(crate) struct DeprecationRegistryPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for DeprecationRegistryPrecompile<R>
where
    R: frame_system::Config + pallet_admin_utils::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
{
    const INDEX: u64 = 2064;
}

#[precompile_utils::precompile]
impl<R> DeprecationRegistryPrecompile<R>
where
    R: frame_system::Config + pallet_admin_utils::Config + pallet_evm::Config,
{
    /// Returns deprecation info for a specific precompile function.
    /// The selector is passed as bytes32 with the 4-byte selector left-aligned
    /// (matching Solidity's bytes4 ABI encoding padded to 32 bytes).
    #[precompile::public("getDeprecationInfo(address,bytes32)")]
    #[precompile::view]
    fn get_deprecation_info(
        _handle: &mut impl PrecompileHandle,
        precompile: Address,
        selector_h256: H256,
    ) -> EvmResult<DeprecationInfoResult> {
        // Convert H160 address to u64 INDEX (low bytes)
        let precompile_h160: H160 = precompile.into();
        let index = precompile_h160.to_low_u64_be();

        // Extract 4-byte selector from left-aligned H256 (Solidity bytes4 is left-aligned)
        let selector_bytes = selector_h256.as_bytes();
        let selector: [u8; 4] = [
            selector_bytes[0],
            selector_bytes[1],
            selector_bytes[2],
            selector_bytes[3],
        ];

        match DeprecationRegistryStorage::<R>::get(index, selector) {
            Some(info) => {
                let new_precompile = H160::from_slice(&info.new_precompile);
                // Left-align the 4-byte new_selector into H256
                let mut new_selector_h256 = [0u8; 32];
                new_selector_h256[..4].copy_from_slice(&info.new_selector);
                let message: UnboundedBytes = info.message.into_inner().into();

                Ok(DeprecationInfoResult {
                    is_deprecated: true,
                    new_precompile: Address(new_precompile),
                    new_selector: H256::from(new_selector_h256),
                    message,
                })
            }
            None => Ok(DeprecationInfoResult {
                is_deprecated: false,
                new_precompile: Address(H160::zero()),
                new_selector: H256::zero(),
                message: UnboundedBytes::from(&b""[..]),
            }),
        }
    }
}

#[derive(Codec)]
struct DeprecationInfoResult {
    is_deprecated: bool,
    new_precompile: Address,
    new_selector: H256,
    message: UnboundedBytes,
}
