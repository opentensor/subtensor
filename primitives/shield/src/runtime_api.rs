//! Runtime API definition for the MEV Shield.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use crate::ShieldedTransaction;
use sp_runtime::traits::Block as BlockT;

type ExtrinsicOf<Block> = <Block as BlockT>::Extrinsic;

sp_api::decl_runtime_apis! {
    pub trait ShieldApi {
        fn try_unshield_tx(uxt: ExtrinsicOf<Block>) -> Option<ShieldedTransaction>;
        fn try_decrypt_shielded_tx(shielded_tx: ShieldedTransaction) -> Option<ExtrinsicOf<Block>>;
    }
}
