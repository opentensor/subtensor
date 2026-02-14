//! Runtime API definition for the shield pallet.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::Encode;
use sp_core::sr25519::Public;
use sp_runtime::traits::Block as BlockT;

sp_api::decl_runtime_apis! {
    pub trait MevShieldApi<Nonce>
    where Nonce: Encode
    {
        fn build_announce_extrinsic(
            next_public_key: Vec<u8>,
            nonce: Nonce,
            aura_pub: Public,
        ) -> Option<<Block as BlockT>::Extrinsic>;
    }
}
