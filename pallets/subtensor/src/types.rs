#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
#[allow(unused_imports)]
use alloc::vec::Vec;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::hexdisplay::AsBytesRef;
use sp_core::Bytes;
use sp_runtime::codec::{Decode, Encode, MaxEncodedLen};

/// Wrapper for Bytes that implements TypeInfo
/// Needed as Bytes doesnt implement it anymore , and the node can't serialize Vec<u8>
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct TensorBytes(pub Bytes);

impl TypeInfo for TensorBytes {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("TensorBytes", module_path!()))
            .composite(
                scale_info::build::Fields::unnamed()
                    .field(|f| f.ty::<Vec<u8>>().type_name("Bytes")),
            )
    }
}

impl AsRef<[u8]> for TensorBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsBytesRef for TensorBytes {
    fn as_bytes_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for TensorBytes {
    fn from(bytes: Vec<u8>) -> Self {
        TensorBytes(sp_core::Bytes(bytes))
    }
}

#[derive(PartialEq)]
pub enum SubnetType {
    STAO,
    DTAO
}

/// Subnet type transtion state
/// 
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
pub struct SubnetTransition<AccountId> {
    pub substake_current_key: Option<(AccountId, AccountId, u16)>,
    pub coldkey: AccountId,
    pub hotkey: AccountId,
    pub initial_total_tao: u64,
    pub initial_alpha_per_tao: u64,
}
