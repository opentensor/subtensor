//! Runtime API definition for the MEV Shield.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use sp_runtime::traits::Block as BlockT;

sp_api::decl_runtime_apis! {
    pub trait ShieldApi {
        fn try_decrypt_extrinsic(uxt: <Block as BlockT>::Extrinsic) -> Option<<Block as BlockT>::Extrinsic>;
    }
}
