#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};

#[freeze_struct("b70069c0a57a7ac8")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SimSwapResult {
    pub tao_amount: TaoBalance,
    pub alpha_amount: AlphaBalance,
    pub tao_fee: TaoBalance,
    pub alpha_fee: AlphaBalance,
}

sp_api::decl_runtime_apis! {
    pub trait SwapRuntimeApi {
        fn current_alpha_price(netuid: NetUid) -> u64;
        fn sim_swap_tao_for_alpha(netuid: NetUid, tao: TaoBalance) -> SimSwapResult;
        fn sim_swap_alpha_for_tao(netuid: NetUid, alpha: AlphaBalance) -> SimSwapResult;
    }
}
