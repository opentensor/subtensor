use crate::keys::sr25519_to_ed25519;
use crate::opaque::SessionKeys;
use frame_election_provider_support::ElectionProviderBase;
use frame_support::WeakBoundedVec;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::OnRuntimeUpgrade;
use pallet_aura;
use pallet_babe;
use pallet_staking::ValidatorPrefs;
use sp_consensus_babe::AuthorityId as BabeAuthorityId;
use sp_consensus_babe::BabeAuthorityWeight;
use sp_runtime::AccountId32;
use sp_runtime::traits::OpaqueKeys;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;

pub(crate) fn pos_upgrade() -> Weight {
    // Initialize weight counter
    // TODO: Compute weight correctly
    let weight = <Runtime as frame_system::Config>::DbWeight::get().reads(1);

    // Nothing to do if we have already migrated to Babe.
    //
    // This check is critical for the runtime upgrade to be idempotent!
    if !pallet_babe::Authorities::<Runtime>::get().len().is_zero() {
        return weight;
    }

    // IMPORTANT: These steps depend on each other.
    //
    // **Do not rearange them!
    initialize_pallet_babe();
    initialize_pallet_session();
    initialize_pallet_staking();

    // Brick the Aura pallet so no new Aura blocks can be produced after this runtime upgrade.
    let _ = pallet_aura::Authorities::<Runtime>::take();

    weight
}

fn initialize_pallet_staking() {
    let authorities = pallet_babe::Authorities::<Runtime>::get()
        .into_iter()
        .map(|a| AccountId32::new(a.0.into_inner().into()))
        .collect::<Vec<_>>();
    let validator_count = authorities.len() as u32;
    let stakers = authorities
        .iter()
        .map(|x| {
            (
                x.clone(),
                x.clone(),
                UNITS,
                pallet_staking::StakerStatus::<AccountId32>::Validator,
            )
        })
        .collect::<Vec<_>>();
    let invulnerables = authorities.clone();
    let force_era = pallet_staking::Forcing::NotForcing;
    let slash_reward_fraction = Perbill::from_percent(10);

    pallet_staking::ValidatorCount::<Runtime>::put(validator_count);
    pallet_staking::MinimumValidatorCount::<Runtime>::put(1);
    pallet_staking::Invulnerables::<Runtime>::put(&invulnerables);
    pallet_staking::ForceEra::<Runtime>::put(force_era);
    pallet_staking::CanceledSlashPayout::<Runtime>::put(0);
    pallet_staking::SlashRewardFraction::<Runtime>::put(slash_reward_fraction);
    pallet_staking::MinNominatorBond::<Runtime>::put(10);
    pallet_staking::MinValidatorBond::<Runtime>::put(10);
    pallet_staking::MaxValidatorsCount::<Runtime>::put(25);
    pallet_staking::MaxNominatorsCount::<Runtime>::put(100);
    let era: sp_staking::EraIndex = 0;
    pallet_staking::CurrentEra::<Runtime>::set(Some(era));
    pallet_staking::ActiveEra::<Runtime>::set(Some(pallet_staking::ActiveEraInfo {
        index: era,
        start: None,
    }));

    for &(ref account, _, balance, ref status) in &stakers {
        log::info!(
            "inserting genesis staker: {:?} => {:?} => {:?}",
            account,
            balance,
            status
        );
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
        }
        match <pallet_staking::Pallet<Runtime>>::bond(
            RawOrigin::Signed(account.clone()).into(),
            balance,
            pallet_staking::RewardDestination::Staked,
        ) {
            Ok(_) => {}
            Err(_e) => {
                todo!()
            }
        };
        match <pallet_staking::Pallet<Runtime>>::validate(
            RawOrigin::Signed(account.clone()).into(),
            ValidatorPrefs {
                commission: Perbill::from_percent(1),
                blocked: false,
            },
        ) {
            Ok(_) => {}
            Err(_e) => {
                todo!()
            }
        };
        assert!(
            pallet_staking::ValidatorCount::<Runtime>::get()
                <= <<Runtime as pallet_staking::Config>::ElectionProvider as ElectionProviderBase>::MaxWinners::get()
        );
    }

    // // all voters are reported to the `VoterList`.
    // assert_eq!(
    //     <Runtime as pallet_staking::Config>::VoterList::count(),
    //     pallet_staking::Nominators::<Runtime>::count()
    //         + pallet_staking::Validators::<Runtime>::count(),
    //     "not all genesis stakers were inserted into sorted list provider, something is wrong."
    // );
}

fn initialize_pallet_babe() {
    let authorities: Vec<(BabeAuthorityId, BabeAuthorityWeight)> =
        pallet_aura::Authorities::<Runtime>::get()
            .into_iter()
            .map(|aura| {
                // BabeAuthorityId and AuraId are both sr25519::Public, so can convert between them
                // easily.
                (BabeAuthorityId::from(aura.into_inner()), 1)
            })
            .collect::<Vec<_>>();
    let bounded_authorities =
        WeakBoundedVec::<_, <Runtime as pallet_babe::Config>::MaxAuthorities>::try_from(
            authorities.to_vec(),
        )
        .expect("Initial number of authorities should be lower than Runtime::MaxAuthorities");

    log::info!("Set {} into bounded authorites", bounded_authorities.len());
    pallet_babe::SegmentIndex::<Runtime>::put(0);
    pallet_babe::Authorities::<Runtime>::put(&bounded_authorities);
    pallet_babe::NextAuthorities::<Runtime>::put(&bounded_authorities);
    pallet_babe::EpochConfig::<Runtime>::put(BABE_GENESIS_EPOCH_CONFIG);
}

fn initialize_pallet_session() {
    let babe_authorities = pallet_babe::Authorities::<Runtime>::get()
        .into_iter()
        .map(|a| a.0)
        .collect::<Vec<_>>();

    log::info!(
        "Initializing pallet_session with authorities: {:?}",
        babe_authorities
    );

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

    pallet_session::CurrentIndex::<Runtime>::put(0);
    pallet_session::Validators::<Runtime>::put(
        keys.iter()
            .map(|(account, _)| account.clone())
            .collect::<Vec<_>>(),
    );
    let key_ids = <Runtime as pallet_session::Config>::Keys::key_ids();
    for (account, session_keys) in keys.iter() {
        pallet_session::NextKeys::<Runtime>::insert(account, session_keys);

        for id in key_ids.iter() {
            pallet_session::KeyOwner::<Runtime>::insert((id, session_keys.get_raw(*id)), account);
        }
    }
    pallet_session::QueuedKeys::<Runtime>::put(keys);
}

use crate::*;

pub struct Migration;

impl OnRuntimeUpgrade for Migration {
    fn on_runtime_upgrade() -> Weight {
        pos_upgrade()
    }
}
