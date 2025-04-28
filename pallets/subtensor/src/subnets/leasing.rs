use super::*;
use frame_support::{
    dispatch::RawOrigin,
    pallet_prelude::*,
    traits::{Defensive, fungible::*, tokens::Preservation},
};
use frame_system::pallet_prelude::*;
use sp_core::blake2_256;
use sp_runtime::{Percent, traits::TrailingZeroInput};
use substrate_fixed::types::{U64F64, U96F32};

pub type LeaseId = u32;

// #[freeze_struct("1941771e0ae01e2e")]
#[derive(Encode, Decode, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub struct SubnetLease<AccountId, BlockNumber> {
    pub beneficiary: AccountId,
    pub coldkey: AccountId,
    pub hotkey: AccountId,
    pub emissions_share: Percent,
    pub end_block: Option<BlockNumber>,
    pub netuid: u16,
}

pub type SubnetLeaseOf<T> = SubnetLease<<T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

impl<T: Config> Pallet<T> {
    pub fn do_register_leased_network(
        origin: T::RuntimeOrigin,
        emissions_share: Percent,
        end_block: Option<BlockNumberFor<T>>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        let (crowdloan_id, crowdloan) = Self::get_crowdloan_being_finalized()?;

        ensure!(who == crowdloan.creator, Error::<T>::InvalidBeneficiary);

        // Initialize the lease id, coldkey and hotkey and keep track of them
        let lease_id = Self::get_next_lease_id()?;
        let lease_coldkey = Self::lease_coldkey(lease_id);
        let lease_hotkey = Self::lease_hotkey(lease_id);
        frame_system::Pallet::<T>::inc_providers(&lease_coldkey);
        frame_system::Pallet::<T>::inc_providers(&lease_hotkey);

        // Transfer money from crowdloan account to leased network coldkey
        <T as Config>::Currency::transfer(
            &crowdloan.funds_account,
            &lease_coldkey,
            crowdloan.raised,
            Preservation::Expendable,
        )?;

        // Register the network
        Self::do_register_network(
            RawOrigin::Signed(lease_coldkey.clone()).into(),
            &lease_hotkey,
            1,
            None,
        )?;

        // Retrieve the network id
        let netuid = Self::find_lease_netuid(&lease_coldkey).unwrap();

        // Create the subnet lease
        SubnetLeases::<T>::insert(
            lease_id,
            SubnetLease {
                beneficiary: who.clone(),
                coldkey: lease_coldkey.clone(),
                hotkey: lease_hotkey.clone(),
                emissions_share,
                end_block,
                netuid,
            },
        );
        SubnetUidToLeaseId::<T>::insert(netuid, lease_id);

        // Enable the beneficiary to operate the subnet through a proxy
        T::ProxyInterface::add_lease_beneficiary_proxy(&lease_coldkey, &who)?;

        // Compute the share to the lease of each contributor to the crowdloan except for
        // the beneficiary which will be computed as the dividends are distributed
        let contributions = pallet_crowdloan::Contributions::<T>::iter_prefix(crowdloan_id)
            .into_iter()
            .filter(|(contributor, _)| contributor != &who);
        for (contributor, amount) in contributions {
            let share: U64F64 = U64F64::from(amount).saturating_div(U64F64::from(crowdloan.raised));
            SubnetLeaseShares::<T>::insert(lease_id, contributor, share);
        }

        Self::deposit_event(Event::SubnetLeaseCreated {
            beneficiary: who,
            lease_id,
            netuid,
            end_block,
        });

        Ok(())
    }

    pub fn do_terminate_lease(origin: T::RuntimeOrigin, lease_id: LeaseId) -> DispatchResult {
        let who = ensure_signed(origin)?;
        let now = frame_system::Pallet::<T>::block_number();

        // Ensure the lease exists and the beneficiary is the caller
        let lease = SubnetLeases::<T>::get(lease_id).ok_or(Error::<T>::LeaseDoesNotExist)?;
        ensure!(lease.beneficiary == who, Error::<T>::InvalidBeneficiary);

        // Ensure the lease has an end block and we are past it
        let end_block = lease.end_block.ok_or(Error::<T>::LeaseHasNoEndBlock)?;
        ensure!(end_block >= now, Error::<T>::LeaseHasNotEnded);

        // Transfer ownership to the beneficiary
        Self::set_subnet_owner_hotkey(lease.netuid, &lease.beneficiary);

        // Stop tracking the lease coldkey and hotkey
        let _ = frame_system::Pallet::<T>::dec_providers(&lease.coldkey).defensive();
        let _ = frame_system::Pallet::<T>::dec_providers(&lease.hotkey).defensive();

        // Remove the lease, its contributors and accumulated dividends from storage
        let _ = SubnetLeaseShares::<T>::clear_prefix(
            lease_id,
            T::MaxContributorsPerLeaseToRemove::get(),
            None,
        );
        AccumulatedLeaseDividends::<T>::remove(lease_id);
        SubnetLeases::<T>::remove(lease_id);

        // Remove the beneficiary proxy
        T::ProxyInterface::remove_lease_beneficiary_proxy(&lease.coldkey, &lease.beneficiary)?;

        Self::deposit_event(Event::SubnetLeaseTerminated {
            beneficiary: lease.beneficiary,
            netuid: lease.netuid,
        });

        // TODO: Refund the weights for the difference between max contributors to refund and the real
        // number of contributors that were refunded
        Ok(())
    }

    pub fn distribute_leased_network_dividends(lease_id: LeaseId, owner_cut_alpha: u64) {
        // Ensure the lease exists
        let lease = match SubnetLeases::<T>::get(lease_id) {
            Some(lease) => lease,
            None => {
                log::error!(
                    "Lease {} doesn't exists so we can't distribute dividends",
                    lease_id
                );
                return;
            }
        };

        // Ensure the lease has not ended
        let now = frame_system::Pallet::<T>::block_number();
        if lease.end_block.is_some_and(|end_block| end_block <= now) {
            return;
        }

        // Get the actual amount of alpha to distribute from the owner's cut,
        // we voluntarily round up to favor the contributors
        let current_contributors_cut_alpha = lease.emissions_share.mul_ceil(owner_cut_alpha);

        // Get the total amount of alpha to distribute from the contributors
        // including the dividends accumulated so far
        let total_contributors_cut_alpha = AccumulatedLeaseDividends::<T>::get(lease_id)
            .saturating_add(current_contributors_cut_alpha);

        // Ensure trading is enabled for the subnet else we accumulate the dividends for later
        // distribution
        if let Err(_) = Self::ensure_subtoken_enabled(lease.netuid) {
            log::error!(
                "Subtoken is not enabled for subnet {} so we can't distribute lease dividends",
                lease.netuid
            );
            AccumulatedLeaseDividends::<T>::mutate(lease_id, |v| {
                *v = v.saturating_add(current_contributors_cut_alpha)
            });
            return;
        }

        // Check if we can safely swap the accumulated alpha dividends for tao else we accumulate
        // the dividends for later distribution
        if let Some(tao_equivalent) =
            Self::sim_swap_alpha_for_tao(lease.netuid, total_contributors_cut_alpha)
        {
            if tao_equivalent < DefaultMinStake::<T>::get() {
                AccumulatedLeaseDividends::<T>::mutate(lease_id, |v| {
                    *v = v.saturating_add(current_contributors_cut_alpha)
                });
                return;
            }
        }

        // Unstake the contributors cut from the subnet as tao to the lease coldkey
        let fee = Self::calculate_staking_fee(
            Some((&lease.hotkey, lease.netuid)),
            &lease.coldkey,
            None,
            &lease.coldkey,
            U96F32::saturating_from_num(total_contributors_cut_alpha),
        );
        let tao_unstaked = Self::unstake_from_subnet(
            &lease.hotkey,
            &lease.coldkey,
            lease.netuid,
            total_contributors_cut_alpha,
            fee,
        );

        // Distribute the contributors cut to the contributors and accumulate the tao
        // distributed so far to obtain how much tao is left to distribute to the beneficiary
        let mut tao_distributed = 0u64;
        for (contributor, share) in SubnetLeaseShares::<T>::iter_prefix(lease_id) {
            let tao_for_contributor = share
                .saturating_mul(U64F64::from(tao_unstaked))
                .floor()
                .to_num::<u64>();
            Self::add_balance_to_coldkey_account(&contributor, tao_for_contributor);
            tao_distributed = tao_distributed.saturating_add(tao_for_contributor);
        }

        // Distribute the leftover tao to the beneficiary
        let beneficiary_cut_tao = tao_unstaked.saturating_sub(tao_distributed);
        Self::add_balance_to_coldkey_account(&lease.beneficiary, beneficiary_cut_tao);

        // Reset the accumulated dividends
        AccumulatedLeaseDividends::<T>::insert(lease_id, 0);
    }

    fn lease_coldkey(lease_id: LeaseId) -> T::AccountId {
        let entropy = ("leasing/coldkey", lease_id).using_encoded(blake2_256);
        Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .expect("infinite length input; no invalid inputs for type; qed")
    }

    fn lease_hotkey(lease_id: LeaseId) -> T::AccountId {
        let entropy = ("leasing/hotkey", lease_id).using_encoded(blake2_256);
        Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .expect("infinite length input; no invalid inputs for type; qed")
    }

    fn get_next_lease_id() -> Result<LeaseId, Error<T>> {
        let lease_id = NextSubnetLeaseId::<T>::get();

        // Increment the lease id
        let next_lease_id = lease_id.checked_add(1).ok_or(Error::<T>::Overflow)?;
        NextSubnetLeaseId::<T>::put(next_lease_id);

        Ok(lease_id)
    }

    fn find_lease_netuid(lease_coldkey: &T::AccountId) -> Option<u16> {
        SubnetOwner::<T>::iter()
            .find(|(_, coldkey)| coldkey == lease_coldkey)
            .map(|(netuid, _)| netuid)
    }

    fn get_crowdloan_being_finalized() -> Result<
        (
            pallet_crowdloan::CrowdloanId,
            pallet_crowdloan::CrowdloanInfoOf<T>,
        ),
        pallet_crowdloan::Error<T>,
    > {
        let crowdloan_id = pallet_crowdloan::CurrentCrowdloanId::<T>::get()
            .ok_or(pallet_crowdloan::Error::<T>::InvalidCrowdloanId)?;
        let crowdloan = pallet_crowdloan::Crowdloans::<T>::get(crowdloan_id)
            .ok_or(pallet_crowdloan::Error::<T>::InvalidCrowdloanId)?;
        Ok((crowdloan_id, crowdloan))
    }
}
