#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use core::marker::PhantomData;
use frame_benchmarking::{BenchmarkError, account, v2::*};
use pallet_subtensor::{Pallet as Subtensor, root_registered::EmaValueProvider, *};
use sp_std::vec::Vec;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};

use super::{STAKE_CHUNK_SUBNETS, STAKE_VALUE_HOTKEYS, StakeValueProgress, StakeValueProvider};
use crate::{AccountId, Runtime};

pub trait Config: frame_system::Config {}

pub struct Pallet<T: Config>(PhantomData<T>);

impl Config for Runtime {}

const FIRST_BENCHMARK_NETUID: u16 = 1024;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn stake_ema_provider_step() -> Result<(), BenchmarkError> {
        let (coldkey, progress) = prepare_stake_value_state();

        #[block]
        {
            let _ = StakeValueProvider::step(&coldkey, progress);
        }

        Ok(())
    }

    fn seed_swap_reserves(netuid: NetUid) {
        SubnetTAO::<Runtime>::insert(netuid, TaoBalance::from(150_000_000_000_u64));
        SubnetAlphaIn::<Runtime>::insert(netuid, AlphaBalance::from(100_000_000_000_u64));
    }

    fn add_balance_to_coldkey_account(coldkey: &AccountId, tao: TaoBalance) {
        let credit = Subtensor::<Runtime>::mint_tao(tao);
        let _ = Subtensor::<Runtime>::spend_tao(coldkey, credit, tao).unwrap();
    }

    fn prepare_stake_value_state() -> (AccountId, StakeValueProgress) {
        let coldkey: AccountId = account("StakeValueColdkey", 0, 0);
        add_balance_to_coldkey_account(&coldkey, TaoBalance::from(1_000_000_000_u64));

        let mut hotkeys: Vec<AccountId> = Vec::with_capacity(STAKE_VALUE_HOTKEYS as usize);
        for hotkey_index in 0..STAKE_VALUE_HOTKEYS {
            hotkeys.push(account("StakeValueHotkey", hotkey_index, 0));
        }
        OwnedHotkeys::<Runtime>::insert(&coldkey, hotkeys.clone());

        let mut first_netuid = None;
        for subnet_index in 0..STAKE_CHUNK_SUBNETS {
            let netuid = NetUid::from(FIRST_BENCHMARK_NETUID.saturating_add(subnet_index as u16));
            if first_netuid.is_none() {
                first_netuid = Some(netuid);
            }

            Subtensor::<Runtime>::init_new_network(netuid, 1);
            SubtokenEnabled::<Runtime>::insert(netuid, true);
            seed_swap_reserves(netuid);

            for hotkey in &hotkeys {
                TotalHotkeyAlpha::<Runtime>::insert(
                    hotkey.clone(),
                    netuid,
                    AlphaBalance::from(1_000_000_000_u64),
                );
            }
        }

        let netuids = Subtensor::<Runtime>::get_all_subnet_netuids();
        let subnet_offset = netuids
            .iter()
            .position(|netuid| Some(*netuid) == first_netuid)
            .unwrap_or_default() as u32;

        (
            coldkey,
            StakeValueProgress {
                subnet_offset,
                accumulated_tao: 0,
            },
        )
    }
}
