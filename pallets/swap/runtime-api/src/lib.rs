#![cfg_attr(not(feature = "std"), no_std)]

use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};
use subtensor_swap_interface::SwapResult;

sp_api::decl_runtime_apis! {
    pub trait SwapRuntimeApi {
        fn current_alpha_price(netuid: NetUid) -> u64;
        fn sim_swap_tao_for_alpha(netuid: NetUid, tao: TaoCurrency) -> SwapResult;
        fn sim_swap_alpha_for_tao(netuid: NetUid, alpha: AlphaCurrency) -> SwapResult;
    }
}
