#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

#[freeze_struct("3a4fd213b5de5eb6")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SimSwapResult {
    pub tao_amount: TaoCurrency,
    pub alpha_amount: AlphaCurrency,
    pub tao_fee: TaoCurrency,
    pub alpha_fee: AlphaCurrency,
}

sp_api::decl_runtime_apis! {
    pub trait SwapRuntimeApi {
        fn current_alpha_price(netuid: NetUid) -> u64;
        fn sim_swap_tao_for_alpha(netuid: NetUid, tao: TaoCurrency) -> SimSwapResult;
        fn sim_swap_alpha_for_tao(netuid: NetUid, alpha: AlphaCurrency) -> SimSwapResult;
    }
}
