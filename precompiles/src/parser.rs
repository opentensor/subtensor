extern crate alloc;

use frame_system::RawOrigin;
use pallet_evm::{ExitError, PrecompileFailure};
use sp_core::U256;
use sp_std::vec;

/// Takes a slice from bytes with PrecompileFailure as Error
pub(crate) fn parse_slice(data: &[u8], from: usize, to: usize) -> Result<&[u8], PrecompileFailure> {
    let maybe_slice = data.get(from..to);
    if let Some(slice) = maybe_slice {
        Ok(slice)
    } else {
        log::error!(
            "fail to get slice from data, {:?}, from {}, to {}",
            &data,
            from,
            to
        );
        Err(PrecompileFailure::Error {
            exit_status: ExitError::InvalidRange,
        })
    }
}

pub(crate) fn parse_pubkey<A: From<[u8; 32]>>(
    data: &[u8],
) -> Result<(A, vec::Vec<u8>), PrecompileFailure> {
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(parse_slice(data, 0, 32)?);

    Ok((
        pubkey.into(),
        data.get(32..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()),
    ))
}

pub(crate) fn contract_to_origin<A: From<[u8; 32]>>(
    contract: &[u8; 32],
) -> Result<RawOrigin<A>, PrecompileFailure> {
    let (account_id, _) = parse_pubkey::<A>(contract)?;
    Ok(RawOrigin::Signed(account_id))
}

pub(crate) fn try_u16_from_u256(value: U256) -> Result<u16, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("the value is outside of u16 bounds".into()),
    })
}
