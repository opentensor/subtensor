use super::*;
use frame_support::traits::{fungible::Mutate, tokens::Preservation};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;

impl<T: Config> Pallet<T> {
    pub fn do_create_subnet_lending_pool(
        origin: T::RuntimeOrigin,
        initial_deposit: u64,
        max_lending_cap: u64,
        emissions_share: u64,
    ) -> dispatch::DispatchResult {
        let creator_coldkey = ensure_signed(origin)?;

        // Ensure we have reached the maximum number of lending pools
        ensure!(
            NextLendingPoolId::<T>::get() < T::LendingPoolsLimit::get(),
            Error::<T>::LendingPoolsLimitReached
        );
        // Ensure the initial deposit is above the minimum required to create a lending pool.
        ensure!(
            initial_deposit >= T::LendingPoolMinInitialDeposit::get(),
            Error::<T>::LendingPoolInitialDepositTooLow
        );
        // Ensure the max lending cap is at least superior to the initial deposit.
        ensure!(
            max_lending_cap > initial_deposit,
            Error::<T>::LendingPoolLendingCapInferiorToInitialDeposit
        );
        // Ensure the max lending cap is not greater than the maximum allowed.
        ensure!(
            max_lending_cap <= T::LendingPoolMaxLendingCap::get(),
            Error::<T>::LendingPoolLendingCapTooHigh
        );
        // Ensure the emisions share is at a minimum of some value.
        ensure!(
            emissions_share >= T::LendingPoolMinEmissionsShare::get(),
            Error::<T>::LendingPoolEmissionsShareTooLow
        );
        // Ensure the emissions share is not greater than 100%.
        ensure!(
            emissions_share <= 100,
            Error::<T>::LendingPoolEmissionsShareTooHigh
        );
        // Ensure creator coldkey contains the initial deposit.
        ensure!(
            Self::get_coldkey_balance(&creator_coldkey) >= initial_deposit,
            Error::<T>::LendingPoolNotEnoughBalanceToPayInitialDeposit
        );

        // Get the next pool id and increment it.
        let pool_id = NextLendingPoolId::<T>::get();
        NextLendingPoolId::<T>::mutate(|id| *id = id.saturating_add(1));

        // Derive the pool coldkey and hotkey.
        let pool_coldkey = Self::get_lending_pool_coldkey(pool_id);
        let _pool_hotkey = Self::get_lending_pool_hotkey(pool_id);

        LendingPools::<T>::insert(
            pool_id,
            LendingPool {
                creator: creator_coldkey.clone(),
                initial_deposit,
                max_lending_cap,
                emissions_share,
            },
        );

        // Transfer the initial deposit from the creator coldkey to the pool coldkey.
        T::Currency::transfer(
            &creator_coldkey,
            &pool_coldkey,
            initial_deposit,
            Preservation::Expendable,
        )?;

        // Add initial deposit to individual pool contributions.
        LendingPoolIndividualContributions::<T>::mutate(pool_id, creator_coldkey, |contribution| {
            *contribution = contribution.saturating_add(initial_deposit);
        });
        // Add initial deposit to total pool contributions.
        LendingPoolTotalContributions::<T>::mutate(pool_id, |total| {
            *total = total.saturating_add(initial_deposit);
        });

        Ok(())
    }

    pub fn get_lending_pool_coldkey(pool_id: u32) -> T::AccountId {
        let entropy = (b"subtensor/lending_pool/cold/", pool_id).using_encoded(blake2_256);
        let key = T::AccountId::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .expect("infinite length input; no invalid inputs for type; qed");

        key
    }

    pub fn get_lending_pool_hotkey(pool_id: u32) -> T::AccountId {
        let entropy = (b"subtensor/lending_pool/hot/", pool_id).using_encoded(blake2_256);
        let key = T::AccountId::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .expect("infinite length input; no invalid inputs for type; qed");

        key
    }
}

// fn edit_lending_proposal_cut() {}

// fn edit_lending_proposal_cap() {}

// fn edit_lending_proposal_end() {}

// maximum of pools for a specific user?
// // - minimum contribution bound
// // - if not already contributed, add as lender
// // - if already contributed, add to lending amount
// fn participate_to_lending_proposal(origin: (), cut: (), cap: (), end: ()) {}

// // The owner of the proposal can call this extrinsic to finalize the
// // proposal, it will be checked if the pooled fund are enough to register
// // for a subnet, then it will register the subnet
// fn finalize_lending_proposal() {}

// // When emission are received by the lend pool, distribute the cut of the subnet owner
// // to the lenders by sending the alpha to the ema price so lenders receive TAO.
// fn hook_on_emission() {}

// // When on lend end, transfer ownership of the subnet to the subnet operator.
// fn hook_on_lease_end() {}
