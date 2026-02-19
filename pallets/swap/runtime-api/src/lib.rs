#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use scale_info::prelude::vec::Vec;
use serde::{Deserialize, Serialize};
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};

#[freeze_struct("ee2ba1ec4ee58ae6")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SimSwapResult {
    pub tao_amount: TaoCurrency,
    pub alpha_amount: AlphaCurrency,
    pub tao_fee: TaoCurrency,
    pub alpha_fee: AlphaCurrency,
    pub tao_slippage: TaoCurrency,
    pub alpha_slippage: AlphaCurrency,
}

#[freeze_struct("d7bbb761fc2b2eac")]
#[derive(Decode, Deserialize, Encode, PartialEq, Eq, Clone, Debug, Serialize, TypeInfo)]
pub struct SubnetPrice {
    pub netuid: NetUid,
    pub price: u64,
}

sp_api::decl_runtime_apis! {
    pub trait SwapRuntimeApi {
        fn current_alpha_price(netuid: NetUid) -> u64;
        fn current_alpha_price_all() -> Vec<SubnetPrice>;
        fn sim_swap_tao_for_alpha(netuid: NetUid, tao: TaoCurrency) -> SimSwapResult;
        fn sim_swap_alpha_for_tao(netuid: NetUid, alpha: AlphaCurrency) -> SimSwapResult;
    }
}
