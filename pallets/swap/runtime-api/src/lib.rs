#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::U96F32;

sp_api::decl_runtime_apis! {
    pub trait SwapRuntimeApi {
        fn current_alpha_price(netuid: u16) -> U96F32;
    }
}
