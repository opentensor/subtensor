use crate::keys::sr25519_to_ed25519;
use crate::opaque::SessionKeys;
use frame_support::WeakBoundedVec;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::OnRuntimeUpgrade;
use pallet_aura;
use pallet_babe;
use pallet_staking::ValidatorPrefs;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_babe::AuthorityId as BabeAuthorityId;
use sp_consensus_babe::BabeAuthorityWeight;
use sp_runtime::AccountId32;
use sp_runtime::traits::OpaqueKeys;
use sp_runtime::traits::Saturating;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;

use crate::*;

const INITIAL_STAKE: u64 = UNITS;

pub struct Migration<T>(sp_std::marker::PhantomData<T>);

impl<T> OnRuntimeUpgrade for Migration<T>
where
    T: frame_system::Config
        + pallet_babe::Config
        + pallet_grandpa::Config
        + pallet_aura::Config<AuthorityId = AuraId>
        + pallet_staking::Config<AccountId = AccountId32, CurrencyBalance = Balance>
        + pallet_session::Config<ValidatorId = AccountId32, Keys = opaque::SessionKeys>
        + pallet_bags_list::Config<VoterBagsListInstance>,
{
    fn on_runtime_upgrade() -> Weight {
        // Nothing to do if we have already migrated.
        //
        // This check is critical for the runtime upgrade to be idempotent!
        let babe_authorities = pallet_babe::Authorities::<T>::get();
        if !babe_authorities.len().is_zero() {
            return T::DbWeight::get().reads(babe_authorities.len() as u64);
        }

        // IMPORTANT: These steps depend on each other.
        //
        // **Do not rearange them!
        Migration::<T>::pallet_babe_runtime_upgrade();
        Migration::<T>::pallet_session_runtime_upgrade();
        Migration::<T>::pallet_staking_runtime_upgrade();

        // Brick the Aura pallet so no new Aura blocks can be produced after this runtime upgrade.
        let _ = pallet_aura::Authorities::<T>::take();

        T::DbWeight::get().reads(0)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
        Migration::<T>::pallet_babe_pre_upgrade()?;
        Migration::<T>::pallet_session_pre_upgrade()?;
        Migration::<T>::pallet_staking_pre_upgrade()?;
        todo!()
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_pre_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
        Migration::<T>::pallet_babe_post_upgrade(Default::default())?;
        Migration::<T>::pallet_session_post_upgrade(Default::default())?;
        Migration::<T>::pallet_staking_post_upgrade(Default::default())?;
        todo!()
    }
}

impl<T> Migration<T>
where
    T: frame_system::Config
        + pallet_babe::Config
        + pallet_grandpa::Config
        + pallet_aura::Config<AuthorityId = AuraId>
        + pallet_staking::Config<AccountId = AccountId32, CurrencyBalance = Balance>
        + pallet_session::Config<ValidatorId = AccountId32, Keys = opaque::SessionKeys>
        + pallet_bags_list::Config<VoterBagsListInstance>,
{
    #[cfg(feature = "try-runtime")]
    fn pallet_babe_pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
        let authorities = pallet_aura::Authorities::<T>::get();
        Ok(authorities.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn pallet_babe_post_upgrade(pre_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
        use sp_std::collections::btree_set::BTreeSet;

        let pre_aura_authorities: Vec<AuraId> =
            Decode::decode(&mut &pre_state[..]).map_err(|_| "Failed to decode pre-state")?;
        let expected_authorities: Vec<(BabeAuthorityId, BabeAuthorityWeight)> =
            pre_aura_authorities
                .into_iter()
                .map(|aura| (BabeAuthorityId::from(aura.into_inner()), 1))
                .collect::<Vec<_>>();

        // Check `pallet_babe::Authorities` and `pallet_babe::NextAuthorities`
        let actual_authorities: BTreeSet<(BabeAuthorityId, BabeAuthorityWeight)> =
            pallet_babe::Authorities::<T>::get().into_iter().collect();
        let actual_next_authorities: BTreeSet<(BabeAuthorityId, BabeAuthorityWeight)> =
            pallet_babe::NextAuthorities::<T>::get()
                .into_iter()
                .collect();
        for (authority, weight) in expected_authorities.iter() {
            if !actual_authorities.contains(&(authority.clone(), *weight)) {
                return Err("Authorities not initialized correctly".into());
            }
            if !actual_next_authorities.contains(&(authority.clone(), *weight)) {
                return Err("NextAuthorities not initialized correctly".into());
            }
        }

        if pallet_babe::SegmentIndex::<T>::get() != 0 {
            return Err("SegmentIndex does not match expected value.".into());
        }
        if pallet_babe::EpochConfig::<T>::get() != Some(BABE_GENESIS_EPOCH_CONFIG) {
            return Err("EpochConfig does not match expected value.".into());
        }
        Ok(())
    }

    fn pallet_babe_runtime_upgrade() -> Weight {
        let authorities: Vec<(BabeAuthorityId, BabeAuthorityWeight)> =
            pallet_aura::Authorities::<T>::get()
                .into_iter()
                .map(|aura| (BabeAuthorityId::from(aura.into_inner()), 1))
                .collect::<Vec<_>>();
        let bounded_authorities =
            WeakBoundedVec::<_, <T as pallet_babe::Config>::MaxAuthorities>::try_from(
                authorities.to_vec(),
            )
            .expect("Initial number of authorities should be lower than T::MaxAuthorities");

        log::info!("Set {} into bounded authorites", bounded_authorities.len());
        pallet_babe::Authorities::<T>::put(&bounded_authorities);
        pallet_babe::NextAuthorities::<T>::put(&bounded_authorities);
        pallet_babe::SegmentIndex::<T>::put(0);
        pallet_babe::EpochConfig::<T>::put(BABE_GENESIS_EPOCH_CONFIG);

        let reads = authorities.len();
        let writes = (authorities.len() * 2) + 2;
        T::DbWeight::get().reads_writes(reads as u64, writes as u64)
    }

    #[cfg(feature = "try-runtime")]
    fn pallet_session_pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
        use sp_std::collections::btree_set::BTreeSet;

        let aura_authorities: Vec<AuraId> =
            pallet_aura::Authorities::<T>::get().into_iter().collect();
        let grandpa_authorities: BTreeSet<GrandpaId> = pallet_grandpa::Authorities::<T>::get()
            .into_iter()
            .map(|(account, _)| account)
            .collect();
        Ok((aura_authorities, grandpa_authorities).encode())
    }

    #[cfg(feature = "try-runtime")]
    fn pallet_session_post_upgrade(pre_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
        use sp_std::collections::btree_set::BTreeSet;

        let (aura_authorities, mut grandpa_authorities): (Vec<AuraId>, BTreeSet<GrandpaId>) =
            Decode::decode(&mut &pre_state[..])
                .expect("Failed to decode pallet_session_post_upgrade state");

        let expected_keys: Vec<(AccountId32, SessionKeys)> = aura_authorities
            .into_iter()
            .map(|aura_authority| {
                let babe_authority = BabeAuthorityId::from(aura_authority.into_inner());
                let keys = SessionKeys {
                    babe: babe_authority.clone(),
                    grandpa: sr25519_to_ed25519(babe_authority.clone())
                        .expect("Failed to map Babe ID to Grandpa ID")
                        .into(),
                };
                let account = AccountId32::new(babe_authority.into_inner().into());
                (account, keys)
            })
            .collect();

        if pallet_session::QueuedKeys::<T>::get() != expected_keys {
            return Err("QueuedKeys does not match expected value.".into());
        }
        if pallet_session::CurrentIndex::<T>::get() != 0 {
            return Err("CurrentIndex does not match expected value.".into());
        }
        if pallet_session::Validators::<T>::get()
            != expected_keys
                .iter()
                .map(|(account, _)| account.clone())
                .collect::<Vec<_>>()
        {
            return Err("Validators does not match expected value.".into());
        }
        let key_ids = <T as pallet_session::Config>::Keys::key_ids();
        for (account, session_keys) in expected_keys.iter() {
            if pallet_session::NextKeys::<T>::get(account) != Some(session_keys.clone()) {
                return Err("NextKeys does not match expected value.".into());
            }

            for id in key_ids.iter() {
                if pallet_session::KeyOwner::<T>::get((id, session_keys.get_raw(*id)))
                    != Some(account.clone())
                {
                    return Err("KeyOwner does not match expected value.".into());
                }
            }
        }

        // Check every grandpa key was migrated exactly once. This check is important to ensure
        // there are no incorrect entires in our `sr25519_to_ed25519` mapping.
        for (_, session_keys) in expected_keys.iter() {
            if grandpa_authorities.take(&session_keys.grandpa).is_none() {
                return Err("All Grandpa keys were not migrated exactly once".into());
            }
        }
        if !grandpa_authorities.is_empty() {
            return Err("Not all grandpa keys were migrated".into());
        }

        Ok(())
    }

    fn pallet_session_runtime_upgrade() -> Weight {
        let mut reads = 0u64;
        let mut writes = 0u64;

        let babe_authorities = pallet_babe::Authorities::<T>::get()
            .into_iter()
            .map(|a| a.0)
            .collect::<Vec<_>>();
        reads.saturating_accrue(babe_authorities.len() as u64);

        log::info!("Initializing pallet_session with authorities: {babe_authorities:?}");

        let keys: Vec<(AccountId32, SessionKeys)> = babe_authorities
            .into_iter()
            .map(|babe_id| {
                let keys = SessionKeys {
                    babe: babe_id.clone(),
                    grandpa: sr25519_to_ed25519(babe_id.clone())
                        .expect("Failed to map Babe ID to Grandpa ID")
                        .into(),
                };
                let account = AccountId32::new(babe_id.into_inner().into());
                log::info!(
                    "Built SessionKeys Account: {:?} Keys: {:?}",
                    &account,
                    &keys,
                );
                (account, keys)
            })
            .collect();

        pallet_session::CurrentIndex::<T>::put(0);
        writes.saturating_inc();
        pallet_session::Validators::<T>::put(
            keys.iter()
                .map(|(account, _)| account.clone())
                .collect::<Vec<_>>(),
        );
        writes.saturating_accrue(keys.len() as u64);

        let key_ids = <T as pallet_session::Config>::Keys::key_ids();
        for (account, session_keys) in keys.iter() {
            pallet_session::NextKeys::<T>::insert(account, session_keys);
            writes.saturating_inc();

            for id in key_ids.iter() {
                pallet_session::KeyOwner::<T>::insert((id, session_keys.get_raw(*id)), account);
                writes.saturating_inc();
            }
        }
        writes.saturating_accrue(keys.len() as u64);
        pallet_session::QueuedKeys::<T>::put(keys);

        T::DbWeight::get().reads_writes(reads, writes)
    }

    #[cfg(feature = "try-runtime")]
    fn pallet_staking_pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
        let expected_stakers = pallet_aura::Authorities::<T>::get()
            .into_iter()
            .map(|aura| AccountId32::from(aura.into_inner()))
            .collect::<Vec<AccountId32>>();
        Ok(expected_stakers.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn pallet_staking_post_upgrade(pre_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
        use frame_support::ensure;

        let expected_stakers: Vec<AccountId32> =
            Decode::decode(&mut &pre_state[..]).map_err(|_| "Failed to decode pre-state")?;
        let expected_validator_count = expected_stakers.len();
        let expected_invulnerables = expected_stakers.clone();
        if pallet_staking::ValidatorCount::<T>::get() != expected_validator_count as u32 {
            return Err("ValidatorCount count does not match expected value.".into());
        }
        if pallet_staking::MinimumValidatorCount::<T>::get() != 1u32 {
            return Err("MinimumValidatorCount does not match expected value.".into());
        }
        if pallet_staking::Invulnerables::<T>::get() != expected_invulnerables {
            return Err("Invulnerables does not match expected value.".into());
        }
        if pallet_staking::ForceEra::<T>::get() != pallet_staking::Forcing::NotForcing {
            return Err("ForceEra does not match expected value.".into());
        }
        if pallet_staking::CanceledSlashPayout::<T>::get() != 0u64 {
            return Err("CanceledSlashPayout does not match expected value.".into());
        }
        if pallet_staking::SlashRewardFraction::<T>::get() != Perbill::from_percent(10) {
            return Err("SlashRewardFraction does not match expected value.".into());
        }
        if pallet_staking::MinNominatorBond::<T>::get() != 10u64 {
            return Err("MinNominatorBond does not match expected value.".into());
        }
        if pallet_staking::MinValidatorBond::<T>::get() != 10u64 {
            return Err("MinValidatorBond does not match expected value.".into());
        }
        if pallet_staking::MaxValidatorsCount::<T>::get() != Some(MaxAuthorities::get()) {
            return Err("MaxValidatorsCount does not match expected value.".into());
        }
        if pallet_staking::MaxNominatorsCount::<T>::get() != None {
            return Err("MaxNominatorsCount does not match expected value.".into());
        }
        if pallet_staking::CurrentEra::<T>::get() != Some(0u32) {
            return Err("CurrentEra does not match expected value.".into());
        }
        match pallet_staking::ActiveEra::<T>::get() {
            Some(active_era) if active_era.index.is_zero() && active_era.start.is_none() => {
                // ActiveEra matches expected value.
            }
            _ => {
                return Err("ActiveEra does not match expected value.".into());
            }
        }

        use sp_staking::StakingAccount;
        for staker in expected_stakers.iter() {
            use frame_election_provider_support::SortedListProvider as _;

            let ledger = pallet_staking::Pallet::<T>::ledger(StakingAccount::Stash(staker.clone()))
                .map_err(|_| return "Expected staker stash not found in ledger.")?;

            ensure!(
                pallet_staking::Bonded::<T>::get(staker.clone()) == Some(staker.clone()),
                "Stash does not match controller for staker"
            );
            ensure!(
                ledger.total >= INITIAL_STAKE,
                "Staker has insufficient total balance in ledger."
            );
            ensure!(
                ledger.active >= INITIAL_STAKE,
                "Staker has insufficient active balance in ledger."
            );
            ensure!(
                ledger.unlocking.is_empty(),
                "Staker has unlocking balance which is not expected."
            );
            ensure!(
                pallet_staking::Validators::<T>::contains_key(staker.clone()),
                "Expected staker to be in Validator list"
            );
            ensure!(
                pallet_bags_list::Pallet::<T, VoterBagsListInstance>::contains(&staker),
                "Expected staker to be in voter list"
            );
        }

        Ok(())
    }

    fn pallet_staking_runtime_upgrade() -> Weight {
        let mut reads = 0u64;
        let mut writes = 0u64;

        let authorities = pallet_babe::Authorities::<T>::get()
            .into_iter()
            .map(|a| AccountId32::new(a.0.into_inner().into()))
            .collect::<Vec<_>>();
        reads.saturating_accrue(authorities.len() as u64);

        let validator_count = authorities.len() as u32;
        let stakers = authorities
            .iter()
            .map(|x| {
                (
                    x.clone(),
                    x.clone(),
                    INITIAL_STAKE,
                    pallet_staking::StakerStatus::<AccountId32>::Validator,
                )
            })
            .collect::<Vec<_>>();
        let invulnerables = authorities.clone();
        let force_era = pallet_staking::Forcing::NotForcing;
        let slash_reward_fraction = Perbill::from_percent(10);

        pallet_staking::ValidatorCount::<T>::put(validator_count);
        pallet_staking::MinimumValidatorCount::<T>::put(1);
        pallet_staking::Invulnerables::<T>::put(&invulnerables);
        pallet_staking::ForceEra::<T>::put(force_era);
        pallet_staking::CanceledSlashPayout::<T>::put(0);
        pallet_staking::SlashRewardFraction::<T>::put(slash_reward_fraction);
        pallet_staking::MinNominatorBond::<T>::put(10);
        pallet_staking::MinValidatorBond::<T>::put(10);
        pallet_staking::MaxValidatorsCount::<T>::put(MaxAuthorities::get());
        let era: sp_staking::EraIndex = 0;
        pallet_staking::CurrentEra::<T>::set(Some(era));
        pallet_staking::ActiveEra::<T>::set(Some(pallet_staking::ActiveEraInfo {
            index: era,
            start: None,
        }));
        writes.saturating_accrue(11u64);

        for &(ref account, _, balance, ref status) in &stakers {
            log::info!("inserting genesis staker: {account:?} => {balance:?} => {status:?}");
            if Balances::usable_balance(account) < balance {
                use frame_support::traits::fungible::Mutate;
                log::warn!(
                    "Account {:?} does not have enough balance to bond ({:?} < {:?}). Topping it up with bond amount.",
                    account,
                    Balances::usable_balance(account),
                    balance
                );
                // If the account does not have enough balance, we top it up with the bond amount.
                let _ = Balances::mint_into(account, balance);
                writes.saturating_inc();
            }
            if let Err(e) = <pallet_staking::Pallet<T>>::bond(
                RawOrigin::Signed(account.clone()).into(),
                balance,
                pallet_staking::RewardDestination::Staked,
            ) {
                log::error!(
                    "Failed to bond {account:?} with balance {balance:?} and status {status:?}: {e:?}"
                );
            };
            writes.saturating_inc();
            if let Err(e) = <pallet_staking::Pallet<T>>::validate(
                RawOrigin::Signed(account.clone()).into(),
                ValidatorPrefs {
                    commission: Perbill::from_percent(1),
                    blocked: false,
                },
            ) {
                log::error!("Failed to set {account:?} as validator: {e:?}");
            };
            writes.saturating_inc();
        }

        T::DbWeight::get().reads_writes(reads, writes)
    }
}
