use super::*;
use crate::alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use substrate_fixed::types::U96F32;

impl<T: Config> Pallet<T> {
    pub fn get_subnet_block_emissions(
        subnets: &[NetUid],
        block_emission: U96F32,
    ) -> BTreeMap<NetUid, U96F32> {
        // Filter out subnets with no first emission block number.
        let subnets_to_emit_to: Vec<NetUid> = subnets
            .to_owned()
            .clone()
            .into_iter()
            .filter(|netuid| FirstEmissionBlockNumber::<T>::get(*netuid).is_some())
            .collect();
        log::debug!("Subnets to emit to: {subnets_to_emit_to:?}");

        // Get sum of alpha moving prices
        let total_moving_prices = subnets_to_emit_to
            .iter()
            .map(|netuid| Self::get_moving_alpha_price(*netuid))
            .fold(U96F32::saturating_from_num(0.0), |acc, ema| {
                acc.saturating_add(ema)
            });
        log::debug!("total_moving_prices: {total_moving_prices:?}");

        // Get subnet TAO emissions.
        subnets_to_emit_to
            .into_iter()
            .map(|netuid| {
                let moving_price = Self::get_moving_alpha_price(netuid);
                log::debug!("moving_price_i: {moving_price:?}");

                let share = block_emission
                    .saturating_mul(moving_price)
                    .checked_div(total_moving_prices)
                    .unwrap_or(U96F32::from_num(0));

                (netuid, share)
            })
            .collect::<BTreeMap<NetUid, U96F32>>()
    }
}
