#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
    pub trait SwapRuntimeApi {
        fn current_alpha_price(netuid: u16) -> u64;
    }
}
