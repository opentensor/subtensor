//! This file defines abstraction for subnet leasing.
//!
//! It is used to register a new leased network through a crowdloan using the `register_leased_network` extrinsic
//! as a call parameter to the crowdloan pallet `create` extrinsic. A new subnet will be registered
//! paying the lock cost using the crowdloan funds and a proxy will be created for the beneficiary
//! to operate the subnet.
//!
//! The crowdloan's contributions are used to compute the share of the emissions that the contributors
//! will receive as dividends. The leftover cap is refunded to the contributors and the beneficiary.
//!
//! The lease can have a defined end block, after which the lease will be terminated and the subnet
//! will be transferred to the beneficiary. In case the lease is perpetual, the lease will never be
//! terminated and emissions will continue to be distributed to the contributors.
//!
//! The lease can be terminated by the beneficiary after the end block has passed (if any) and the subnet
//! ownership will be transferred to the beneficiary.

use super::*;
use frame_support::{
    dispatch::RawOrigin,
    traits::{Defensive, fungible::*, tokens::Preservation},
};
use frame_system::pallet_prelude::*;
use sp_core::blake2_256;
use sp_runtime::{Percent, Saturating, traits::TrailingZeroInput};
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, NetUid};

pub type LeaseId = u32;

pub type CurrencyOf<T> = <T as Config>::Currency;

pub type BalanceOf<T> =
    <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[freeze_struct("eec95dad4d06b2d5")]
#[derive(Encode, Decode, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo, Clone)]
pub struct SubnetLease<AccountId, BlockNumber, Balance> {
    /// The beneficiary of the lease, able to operate the subnet through
    /// a proxy and taking ownership of the subnet at the end of the lease (if defined).
    pub beneficiary: AccountId,
    /// The coldkey of the lease.
    pub coldkey: AccountId,
    /// The hotkey of the lease.
    pub hotkey: AccountId,
    /// The share of the emissions that the contributors will receive.
    pub emissions_share: Percent,
    /// The block at which the lease will end. If not defined, the lease is perpetual.
    pub end_block: Option<BlockNumber>,
    /// The netuid of the subnet that the lease is for.
    pub netuid: NetUid,
    /// The cost of the lease including the network registration and proxy.
    pub cost: Balance,
}

pub type SubnetLeaseOf<T> =
    SubnetLease<<T as frame_system::Config>::AccountId, BlockNumberFor<T>, BalanceOf<T>>;

impl<T: Config> Pallet<T> {
    /// Register a new leased network through a crowdloan. A new subnet will be registered
    /// paying the lock cost using the crowdloan funds and a proxy will be created for the beneficiary
    /// to operate the subnet.
    ///
    /// The crowdloan's contributions are used to compute the share of the emissions that the contributors
    /// will receive as dividends.
    ///
    /// The leftover cap is refunded to the contributors and the beneficiary.
    pub fn do_register_leased_network(
        origin: T::RuntimeOrigin,
        emissions_share: Percent,
        end_block: Option<BlockNumberFor<T>>,
    ) -> DispatchResultWithPostInfo {
        let who = ensure_signed(origin)?;
        let now = frame_system::Pallet::<T>::block_number();

        // Ensure the origin is the creator of the crowdloan
        let (crowdloan_id, crowdloan) = Self::get_crowdloan_being_finalized()?;
        ensure!(
            who == crowdloan.creator,
            Error::<T>::ExpectedBeneficiaryOrigin
        );

        if let Some(end_block) = end_block {
            ensure!(end_block > now, Error::<T>::LeaseCannotEndInThePast);
        }

        let (lease_id, lease) =
            Self::initialize_lease(&crowdloan, &who, emissions_share, end_block)?;

        let origin = RawOrigin::Signed(lease.coldkey.clone()).into();
        let mechid = 1;
        Self::do_register_network(origin, &lease.hotkey, mechid, None)?;

        let netuid = Self::find_lease_netuid(&lease.coldkey)?;
        let leftover_cap =
            Self::finalize_lease_creation(&crowdloan, lease_id, lease.clone(), netuid)?;

        // Get all the contributions to the crowdloan except for the beneficiary
        // because its share will be computed as the dividends are distributed
        let contributions = pallet_crowdloan::Contributions::<T>::iter_prefix(crowdloan_id)
            .into_iter()
            .filter(|(contributor, _)| contributor != &who);

        let mut refunded_cap = 0u64;
        for (contributor, amount) in contributions {
            // Compute the share of the contributor to the lease
            let share: U64F64 = U64F64::from(amount).saturating_div(U64F64::from(crowdloan.raised));
            SubnetLeaseShares::<T>::insert(lease_id, &contributor, share);

            // Refund the unused part of the cap to the contributor relative to their share
            let contributor_refund = share
                .saturating_mul(U64F64::from(leftover_cap))
                .floor()
                .saturating_to_num::<u64>();
            <T as Config>::Currency::transfer(
                &lease.coldkey,
                &contributor,
                contributor_refund,
                Preservation::Expendable,
            )?;
            refunded_cap = refunded_cap.saturating_add(contributor_refund);
        }

        // Refund what's left after refunding the contributors to the beneficiary
        let beneficiary_refund = leftover_cap.saturating_sub(refunded_cap);
        <T as Config>::Currency::transfer(
            &lease.coldkey,
            &who,
            beneficiary_refund,
            Preservation::Expendable,
        )?;

        Self::deposit_event(Event::SubnetLeaseCreated {
            beneficiary: who,
            lease_id,
            netuid,
            end_block,
        });

        if crowdloan.contributors_count < T::MaxContributors::get() {
            // We have less contributors than the max allowed, so we need to refund the difference
            Ok(
                Some(SubnetLeasingWeightInfo::<T>::do_register_leased_network(
                    crowdloan.contributors_count,
                ))
                .into(),
            )
        } else {
            // We have the max number of contributors, so we don't need to refund anything
            Ok(().into())
        }
    }

    /// Announces a subnet sale into a lease using the beneficiary coldkey and the min sale price.
    ///
    /// The caller must be the subnet owner and the crowdloan's raised amount must be greater
    /// than or equal to the min sale price.
    pub fn do_announce_subnet_sale_into_lease(
        origin: OriginFor<T>,
        netuid: NetUid,
        beneficiary: T::AccountId,
        min_sale_price: TaoCurrency,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        let now = frame_system::Pallet::<T>::block_number();

        let (crowdloan_id, crowdloan) = Self::get_crowdloan_being_finalized()?;
        ensure!(
            crowdloan.raised >= min_sale_price.to_u64(),
            Error::<T>::SubnetMinSalePriceNotMet
        );

        ensure!(
            !ColdkeySwapAnnouncements::<T>::contains_key(&who),
            Error::<T>::ColdkeySwapAnnouncementAlreadyExists
        );
        ensure!(
            !SubnetSaleIntoLeaseAnnouncements::<T>::contains_key(&who),
            Error::<T>::SubnetSaleIntoLeaseAnnouncementAlreadyExists
        );

        ensure!(
            SubnetOwner::<T>::get(netuid) == who,
            Error::<T>::NotSubnetOwner
        );
        let owners = SubnetOwner::<T>::iter().collect::<Vec<_>>();
        ensure!(
            owners.iter().filter(|(_, owner)| owner == &who).count() == 1,
            Error::<T>::TooManySubnetsOwned
        );

        SubnetSaleIntoLeaseAnnouncements::<T>::insert(
            who,
            (now, beneficiary.clone(), netuid, crowdloan_id),
        );

        Self::deposit_event(Event::SubnetSaleIntoLeaseAnnounced {
            beneficiary,
            netuid,
            min_sale_price,
        });
        Ok(())
    }

    /// Settles a subnet sale into a lease.
    ///
    /// The caller must be the subnet owner and the announcement must be older than the coldkey swap announcement delay.
    ///
    /// The coldkey swap will`` be performed and a new lease will be created with the beneficiary coldkey
    /// to operate the subnet through a proxy.
    ///
    /// The crowdloan's contributions are used to compute the share of the emissions that the contributors
    /// will receive as dividends.
    ///
    /// The leftover cap is paid to the seller.
    pub fn do_settle_subnet_sale_into_lease(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;

        let (when, beneficiary, netuid, crowdloan_id) =
            SubnetSaleIntoLeaseAnnouncements::<T>::take(who.clone())
                .ok_or(Error::<T>::SubnetSaleIntoLeaseAnnouncementNotFound)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let delay = ColdkeySwapAnnouncementDelay::<T>::get();
        ensure!(
            now >= when.saturating_add(delay),
            Error::<T>::SubnetLeaseIntoSaleSettledTooEarly
        );

        let crowdloan = pallet_crowdloan::Crowdloans::<T>::get(crowdloan_id)
            .ok_or(pallet_crowdloan::Error::<T>::InvalidCrowdloanId)?;

        let emissions_share = Percent::from_percent(100);
        let end_block = None;
        let (lease_id, lease) =
            Self::initialize_lease(&crowdloan, &beneficiary, emissions_share, end_block)?;

        Self::do_swap_coldkey(&who, &lease.coldkey)?;

        let leftover_cap =
            Self::finalize_lease_creation(&crowdloan, lease_id, lease.clone(), netuid)?;

        // Get all the contributions to the crowdloan except the seller because he doesn't get any shares
        // and the beneficiary because its share will be computed as the dividends are distributed
        let contributions = pallet_crowdloan::Contributions::<T>::iter_prefix(crowdloan_id)
            .into_iter()
            .filter(|(contributor, _)| contributor != &who && contributor != &beneficiary);

        for (contributor, amount) in contributions {
            // Compute the share of the contributor to the lease
            let denominator = U64F64::from(crowdloan.raised.saturating_sub(crowdloan.deposit));
            let share: U64F64 = U64F64::from(amount).saturating_div(denominator);
            SubnetLeaseShares::<T>::insert(lease_id, &contributor, share);
        }

        <T as Config>::Currency::transfer(
            &lease.coldkey,
            &who,
            leftover_cap,
            Preservation::Expendable,
        )?;

        Self::deposit_event(Event::SubnetLeaseCreated {
            beneficiary: beneficiary.clone(),
            lease_id,
            netuid,
            end_block,
        });
        Self::deposit_event(Event::SubnetSaleIntoLeaseSettled {
            beneficiary,
            netuid,
        });
        Ok(())
    }

    pub fn do_cancel_subnet_sale_into_lease(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;

        let (_, _, netuid, crowdloan_id) = SubnetSaleIntoLeaseAnnouncements::<T>::take(who.clone())
            .ok_or(Error::<T>::SubnetSaleIntoLeaseAnnouncementNotFound)?;

        pallet_crowdloan::Crowdloans::<T>::try_mutate(crowdloan_id, |crowdloan| match crowdloan {
            Some(crowdloan) => {
                // Unmark the crowdloan as finalized to allow the contributions to be refunded
                crowdloan.finalized = false;
                Ok::<_, DispatchError>(())
            }
            None => Err(pallet_crowdloan::Error::<T>::InvalidCrowdloanId.into()),
        })?;

        Self::deposit_event(Event::SubnetSaleIntoLeaseCancelled { netuid });
        Ok(())
    }

    /// Terminate a lease.
    ///
    /// The beneficiary can terminate the lease after the end block has passed and get the subnet ownership.
    /// The subnet is transferred to the beneficiary and the lease is removed from storage.
    pub fn do_terminate_lease(
        origin: T::RuntimeOrigin,
        lease_id: LeaseId,
        hotkey: T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let who = ensure_signed(origin)?;
        let now = frame_system::Pallet::<T>::block_number();

        // Ensure the lease exists and the beneficiary is the caller
        let lease = SubnetLeases::<T>::get(lease_id).ok_or(Error::<T>::LeaseDoesNotExist)?;
        ensure!(
            lease.beneficiary == who,
            Error::<T>::ExpectedBeneficiaryOrigin
        );

        // Ensure the lease has an end block and we are past it
        let end_block = lease.end_block.ok_or(Error::<T>::LeaseHasNoEndBlock)?;
        ensure!(now >= end_block, Error::<T>::LeaseHasNotEnded);

        // Transfer ownership to the beneficiary
        ensure!(
            Self::coldkey_owns_hotkey(&lease.beneficiary, &hotkey),
            Error::<T>::BeneficiaryDoesNotOwnHotkey
        );
        SubnetOwner::<T>::insert(lease.netuid, lease.beneficiary.clone());
        Self::set_subnet_owner_hotkey(lease.netuid, &hotkey);

        // Stop tracking the lease coldkey and hotkey
        let _ = frame_system::Pallet::<T>::dec_providers(&lease.coldkey).defensive();
        let _ = frame_system::Pallet::<T>::dec_providers(&lease.hotkey).defensive();

        // Remove the lease, its contributors and accumulated dividends from storage
        let clear_result =
            SubnetLeaseShares::<T>::clear_prefix(lease_id, T::MaxContributors::get(), None);
        AccumulatedLeaseDividends::<T>::remove(lease_id);
        SubnetLeases::<T>::remove(lease_id);

        // Remove the beneficiary proxy
        T::ProxyInterface::remove_lease_beneficiary_proxy(&lease.coldkey, &lease.beneficiary)?;

        Self::deposit_event(Event::SubnetLeaseTerminated {
            beneficiary: lease.beneficiary,
            netuid: lease.netuid,
        });

        if clear_result.unique < T::MaxContributors::get() {
            // We have cleared less than the max number of shareholders, so we need to refund the difference
            Ok(Some(SubnetLeasingWeightInfo::<T>::do_terminate_lease(
                clear_result.unique,
            ))
            .into())
        } else {
            // We have cleared the max number of shareholders, so we don't need to refund anything
            Ok(().into())
        }
    }

    /// Hook used when the subnet owner's cut is distributed to split the amount into dividends
    /// for the contributors and the beneficiary in shares relative to their initial contributions.
    /// It accumulates dividends to be distributed later when the interval for distribution is reached.
    /// Distribution is made in alpha and stake to the contributor coldkey and lease hotkey.
    pub fn distribute_leased_network_dividends(lease_id: LeaseId, owner_cut_alpha: AlphaCurrency) {
        // Ensure the lease exists
        let Some(lease) = SubnetLeases::<T>::get(lease_id) else {
            log::debug!("Lease {lease_id} doesn't exists so we can't distribute dividends");
            return;
        };

        // Ensure the lease has not ended
        let now = frame_system::Pallet::<T>::block_number();
        if lease.end_block.is_some_and(|end_block| end_block <= now) {
            return;
        }

        // Get the actual amount of alpha to distribute from the owner's cut,
        // we voluntarily round up to favor the contributors
        let current_contributors_cut_alpha =
            lease.emissions_share.mul_ceil(owner_cut_alpha.to_u64());

        // Get the total amount of alpha to distribute from the contributors
        // including the dividends accumulated so far
        let total_contributors_cut_alpha = AccumulatedLeaseDividends::<T>::get(lease_id)
            .saturating_add(current_contributors_cut_alpha.into());

        // Ensure the distribution interval is not zero
        let rem = now
            .into()
            .checked_rem(T::LeaseDividendsDistributionInterval::get().into());
        if rem.is_none() {
            // This should never happen but we check it anyway
            log::error!("LeaseDividendsDistributionInterval must be greater than 0");
            AccumulatedLeaseDividends::<T>::set(lease_id, total_contributors_cut_alpha);
            return;
        } else if rem.is_some_and(|rem| rem > 0u32.into()) {
            // This is not the time to distribute dividends, so we accumulate the dividends
            AccumulatedLeaseDividends::<T>::set(lease_id, total_contributors_cut_alpha);
            return;
        }

        // We use a storage layer to ensure the distribution is atomic.
        if let Err(err) = frame_support::storage::with_storage_layer(|| {
            let mut alpha_distributed = AlphaCurrency::ZERO;

            // Distribute the contributors cut to the contributors and accumulate the alpha
            // distributed so far to obtain how much alpha is left to distribute to the beneficiary
            for (contributor, share) in SubnetLeaseShares::<T>::iter_prefix(lease_id) {
                let alpha_for_contributor = share
                    .saturating_mul(U64F64::from(total_contributors_cut_alpha.to_u64()))
                    .ceil()
                    .saturating_to_num::<u64>();

                Self::transfer_stake_within_subnet(
                    &lease.coldkey,
                    &lease.hotkey,
                    &contributor,
                    &lease.hotkey,
                    lease.netuid,
                    alpha_for_contributor.into(),
                )?;
                alpha_distributed = alpha_distributed.saturating_add(alpha_for_contributor.into());

                Self::deposit_event(Event::SubnetLeaseDividendsDistributed {
                    lease_id,
                    contributor,
                    alpha: alpha_for_contributor.into(),
                });
            }

            // Distribute the leftover alpha to the beneficiary
            let beneficiary_cut_alpha =
                total_contributors_cut_alpha.saturating_sub(alpha_distributed);
            Self::transfer_stake_within_subnet(
                &lease.coldkey,
                &lease.hotkey,
                &lease.beneficiary,
                &lease.hotkey,
                lease.netuid,
                beneficiary_cut_alpha.into(),
            )?;
            Self::deposit_event(Event::SubnetLeaseDividendsDistributed {
                lease_id,
                contributor: lease.beneficiary.clone(),
                alpha: beneficiary_cut_alpha.into(),
            });

            // Reset the accumulated dividends
            AccumulatedLeaseDividends::<T>::insert(lease_id, AlphaCurrency::ZERO);

            Ok::<(), DispatchError>(())
        }) {
            log::debug!("Couldn't distributing dividends for lease {lease_id}: {err:?}");
            AccumulatedLeaseDividends::<T>::set(lease_id, total_contributors_cut_alpha);
        };
    }

    // Get the crowdloan being finalized from the crowdloan pallet when the call is executed,
    // and the current crowdloan ID is exposed to us.
    pub(crate) fn get_crowdloan_being_finalized() -> Result<
        (
            pallet_crowdloan::CrowdloanId,
            pallet_crowdloan::CrowdloanInfoOf<T>,
        ),
        pallet_crowdloan::Error<T>,
    > {
        let crowdloan_id = pallet_crowdloan::CurrentCrowdloanId::<T>::get()
            .ok_or(pallet_crowdloan::Error::<T>::CurrentCrowdloanIdNotSet)?;
        let crowdloan = pallet_crowdloan::Crowdloans::<T>::get(crowdloan_id)
            .ok_or(pallet_crowdloan::Error::<T>::InvalidCrowdloanId)?;
        Ok((crowdloan_id, crowdloan))
    }

    fn lease_coldkey(lease_id: LeaseId) -> Result<T::AccountId, DispatchError> {
        let entropy = ("leasing/coldkey", lease_id).using_encoded(blake2_256);
        T::AccountId::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .map_err(|_| Error::<T>::InvalidValue.into())
    }

    fn lease_hotkey(lease_id: LeaseId) -> Result<T::AccountId, DispatchError> {
        let entropy = ("leasing/hotkey", lease_id).using_encoded(blake2_256);
        T::AccountId::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
            .map_err(|_| Error::<T>::InvalidValue.into())
    }

    fn get_next_lease_id() -> Result<LeaseId, Error<T>> {
        let lease_id = NextSubnetLeaseId::<T>::get();

        // Increment the lease id
        let next_lease_id = lease_id.checked_add(1).ok_or(Error::<T>::Overflow)?;
        NextSubnetLeaseId::<T>::put(next_lease_id);

        Ok(lease_id)
    }

    fn find_lease_netuid(lease_coldkey: &T::AccountId) -> Result<NetUid, Error<T>> {
        SubnetOwner::<T>::iter()
            .find(|(_, coldkey)| coldkey == lease_coldkey)
            .map(|(netuid, _)| netuid)
            .ok_or(Error::<T>::LeaseNetuidNotFound)
    }

    fn initialize_lease(
        crowdloan: &pallet_crowdloan::CrowdloanInfoOf<T>,
        beneficiary: &T::AccountId,
        emissions_share: Percent,
        end_block: Option<BlockNumberFor<T>>,
    ) -> Result<(LeaseId, SubnetLeaseOf<T>), DispatchError> {
        let lease_id = Self::get_next_lease_id()?;
        let lease_coldkey = Self::lease_coldkey(lease_id)?;
        let lease_hotkey = Self::lease_hotkey(lease_id)?;
        frame_system::Pallet::<T>::inc_providers(&lease_coldkey);
        frame_system::Pallet::<T>::inc_providers(&lease_hotkey);

        <T as Config>::Currency::transfer(
            &crowdloan.funds_account,
            &lease_coldkey,
            crowdloan.raised,
            Preservation::Expendable,
        )?;

        let lease = SubnetLease {
            coldkey: lease_coldkey,
            hotkey: lease_hotkey,
            beneficiary: beneficiary.clone(),
            emissions_share,
            end_block,
            netuid: Default::default(),
            cost: Default::default(),
        };

        Ok((lease_id, lease))
    }

    fn finalize_lease_creation(
        crowdloan: &pallet_crowdloan::CrowdloanInfoOf<T>,
        lease_id: LeaseId,
        lease: SubnetLeaseOf<T>,
        netuid: NetUid,
    ) -> Result<BalanceOf<T>, DispatchError> {
        // Enable the beneficiary to operate the subnet through a proxy
        T::ProxyInterface::add_lease_beneficiary_proxy(&lease.coldkey, &lease.beneficiary)?;

        // Get left leftover cap and compute the overall cost of the lease
        let leftover_cap = <T as Config>::Currency::balance(&lease.coldkey);
        let cost = crowdloan.raised.saturating_sub(leftover_cap);

        // The lease take should be 0% to allow all contributors to receive dividends entirely
        Self::delegate_hotkey(&lease.hotkey, 0);

        SubnetLeases::<T>::insert(
            lease_id,
            SubnetLease {
                netuid,
                cost,
                ..lease
            },
        );
        SubnetUidToLeaseId::<T>::insert(netuid, lease_id);

        Ok(leftover_cap)
    }
}

/// Weight functions needed for subnet leasing.
pub struct SubnetLeasingWeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> SubnetLeasingWeightInfo<T> {
    pub fn do_register_leased_network(k: u32) -> Weight {
        Weight::from_parts(301_560_714, 10079)
            .saturating_add(Weight::from_parts(26_884_006, 0).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(41_u64))
            .saturating_add(T::DbWeight::get().reads(2_u64.saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(55_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64.saturating_mul(k.into())))
            .saturating_add(Weight::from_parts(0, 2579).saturating_mul(k.into()))
    }

    pub fn do_terminate_lease(k: u32) -> Weight {
        Weight::from_parts(56_635_122, 6148)
            .saturating_add(Weight::from_parts(912_993, 0).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(6_u64))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(k.into())))
            .saturating_add(Weight::from_parts(0, 2529).saturating_mul(k.into()))
    }
}
