use frame_support::traits::{fungible, OnRuntimeUpgrade};

use crate::*;

pub struct Migration;

impl OnRuntimeUpgrade for Migration {
    fn on_runtime_upgrade() -> Weight {
        // First, we need to initialize the TotalSubnetLocked
        let subnets_len = pallet_subtensor::SubnetLocked::<Runtime>::iter().count() as u64;
        let total_subnet_locked: u64 = pallet_subtensor::SubnetLocked::<Runtime>::iter()
            .fold(0, |acc, (_, v)| acc.saturating_add(v));
        pallet_subtensor::TotalSubnetLocked::<Runtime>::put(total_subnet_locked);

        // Now, we can rejig the total issuance
        let total_account_balances =
            <<Runtime as pallet_subtensor::Config>::Currency as fungible::Inspect<
                <Runtime as frame_system::Config>::AccountId,
            >>::total_issuance();
        let total_stake = pallet_subtensor::TotalStake::<Runtime>::get();
        let total_subnet_locked = pallet_subtensor::TotalSubnetLocked::<Runtime>::get();

        let prev_total_issuance = pallet_subtensor::TotalIssuance::<Runtime>::get();
        let new_total_issuance = total_account_balances
            .saturating_add(total_stake)
            .saturating_add(total_subnet_locked);
        pallet_subtensor::TotalIssuance::<Runtime>::put(new_total_issuance);

        log::info!(
            "Subtensor Pallet TI Rejigged: previously: {:?}, new: {:?}",
            prev_total_issuance,
            new_total_issuance
        );

        <Runtime as frame_system::Config>::DbWeight::get()
            .reads_writes(subnets_len.saturating_add(5), 1)
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        // These are usually checked anyway by try-runtime-cli, but just in case check them again
        // explicitly here.
        pallet_subtensor::Pallet::<Runtime>::check_accounting_invariants()?;
        Ok(())
    }
}
