//! Runtime API definition for the shield pallet.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use sp_runtime::traits::Block as BlockT;

sp_api::decl_runtime_apis! {
    pub trait MevShieldApi {
        fn try_decrypt_extrinsic(
            uxt: <Block as BlockT>::Extrinsic,
            curr_sk_bytes: Vec<u8>,
        ) -> Option<<Block as BlockT>::Extrinsic>;
    }
}
