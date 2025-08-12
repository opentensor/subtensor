#![cfg_attr(not(feature = "std"), no_std)]

use subtensor_swap_interface::SwapResult;

sp_api::decl_runtime_apis! {
    pub trait SwapRuntimeApi {
        fn current_alpha_price(netuid: u16) -> u64;
        fn sim_swap_tao_for_alpha(netuid: u16, tao: u64) -> SwapResult;
        fn sim_swap_alpha_for_tao(netuid: u16, alpha: u64) -> SwapResult;
    }
}
