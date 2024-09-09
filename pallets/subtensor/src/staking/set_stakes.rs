use super::*;
use std::collections::HashMap;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    pub fn do_set_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        stakes: Vec<(u16, u64)>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin.clone())?;
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );
        let mut total_stake: u64 = 0;
        stakes
            .iter()
            .for_each(|stake| total_stake = total_stake.saturating_add(stake.1));

        if total_stake != u64::MAX {
            return Error::<T>::NotEnoughBalanceToStake.into();
        }

        let netuids = Self::get_all_subnet_netuids();

        // netuid, current alpha, current tao
        let mut set_stake_parameters: HashMap<u16, (I96F32, I96F32)> = HashMap::new();
        let mut total_tao = I96F32::from_num(0);

        for netuid in netuids.iter() {
            let mechid: u16 = SubnetMechanism::<T>::get(netuid);
            let alpha_64 = Alpha::<T>::get((hotkey.clone(), coldkey.clone(), netuid));
            if alpha_64 == 0 {
                continue;
            }
            let alpha = I96F32::from_num(alpha_64);
            let tao;

            if mechid == 1 {
                // Step 4: Dynamic mechanism
                // Step 4a: Get current TAO in the subnet
                let subnet_tao: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(netuid));
                // Step 4b: Get current alpha in the subnet
                let subnet_alpha: I96F32 = I96F32::from_num(SubnetAlphaIn::<T>::get(netuid));
                // Step 4c: Calculate constant product k
                let k: I96F32 = subnet_alpha.saturating_mul(subnet_tao);
                // Step 4d: Calculate TAO unstaked using constant product formula
                tao = subnet_tao.saturating_sub(
                    k.checked_div(subnet_alpha.saturating_add(alpha))
                        .unwrap_or(I96F32::from_num(0)),
                );
                // Step 4e: Calculate new subnet alpha
                // new_subnet_alpha = subnet_alpha.saturating_add(float_alpha_unstaked);
            } else {
                // Step 5: Stable mechanism
                // Step 5a: TAO unstaked is equal to alpha unstaked
                tao = alpha;
                // Step 5b: New subnet alpha is always zero in stable mechanism
                // new_subnet_alpha = I96F32::from_num(0.0);
            }

            total_tao = total_tao.saturating_add(tao);

            set_stake_parameters.insert(*netuid, (alpha, tao));

            // let tao_unstaked_u64: u64 =
            //     tao_unstaked.min(I96F32::from_num(u64::MAX)).to_num::<u64>();
        }

        let stake_map = stakes
            .iter()
            .map(|(netuid, propotion)| {
                let tao = total_tao
                    .saturating_mul(I96F32::from_num(*propotion))
                    .saturating_div(I96F32::from_num(u64::MAX));
                (*netuid, tao)
            })
            .collect::<HashMap<u16, I96F32>>();

        for netuid in netuids.iter() {

            let (more_stake, amount) = match (set_stake_parameters.get(netuid), stake_map.get(netuid)) {
                (Some(parameter), Some(stake)) => {
					// will stake more tao
                    if parameter.1 < *stake {

                        // stake more
                    } else if parameter.1 > *stake {
                        if let Some(stake_to_be_added) =
						parameter.1.saturating_sub(stake).checked_to_num::<u64>()
                        {
                            // stake some
                            Self::do_add_stake(
                                origin.clone(),
                                hotkey.clone(),
                                *netuid,
                                stake_to_be_added,
                            )?;
                        } else {
                            log::error!("can not convert I96F32 to u64 in do_set_stake");
                        }
                    }
                }
                (Some(parameter), None) => {
                    // unstake all
                }
                (None, Some(stake)) => {
                    // stake all
                    if let Some(stake_to_be_added) = stake.checked_to_num::<u64>() {
                        // stake some
                        Self::do_add_stake(
                            origin.clone(),
                            hotkey.clone(),
                            *netuid,
                            stake_to_be_added,
                        )?;
                    } else {
                        log::error!("can not convert I96F32 to u64 in do_set_stake");
                    }
                }
                _ => (true, 0)
            };

			if amount != 0 {
				let mechid: u16 = SubnetMechanism::<T>::get(netuid);
				if more_stake {
					Self::do_add_stake(
						origin.clone(),
						hotkey.clone(),
						*netuid,
						amount,
					)?;
				} else {

					let tao_unstaked = if mechid == 1 {
						// Step 4: Dynamic mechanism
						// Step 4a: Get current TAO in the subnet
						let subnet_tao: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(netuid));
						// Step 4b: Get current alpha in the subnet
						let subnet_alpha: I96F32 =
							I96F32::from_num(SubnetAlphaIn::<T>::get(netuid));
						// Step 4c: Calculate constant product k
						let k: I96F32 = subnet_alpha.saturating_mul(subnet_tao);
						// Step 4d: Calculate TAO unstaked using constant product formula
						tao_unstaked = subnet_tao.saturating_sub(
							k.checked_div(subnet_alpha.saturating_add(float_alpha_unstaked))
								.unwrap_or(I96F32::from_num(0)),
						);
						// Step 4e: Calculate new subnet alpha
						new_subnet_alpha = subnet_alpha.saturating_add(float_alpha_unstaked);
					} else {
						// Step 5: Stable mechanism
						// Step 5a: TAO unstaked is equal to alpha unstaked
						tao_unstaked = float_alpha_unstaked;
						// Step 5b: New subnet alpha is always zero in stable mechanism
						new_subnet_alpha = I96F32::from_num(0.0);
					}

				}
			}
        }

        // Self::do_set_stake(origin, hotkey, set_stakes)

        // get all current stake in all netuids

        // compute the gap between current staked tao and new tao

        // for each netuid, call add_stake or remove_stake
        Ok(())
    }
}
