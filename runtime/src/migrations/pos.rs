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
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::crypto::Ss58Codec;
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
    let minimum_validator_count = 1;
    let _validator_count = authorities.len() as u32;
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

    pallet_staking::ValidatorCount::<Runtime>::put(11);
    pallet_staking::MinimumValidatorCount::<Runtime>::put(minimum_validator_count);
    pallet_staking::Invulnerables::<Runtime>::put(&invulnerables);
    pallet_staking::ForceEra::<Runtime>::put(force_era);
    pallet_staking::CanceledSlashPayout::<Runtime>::put(0);
    pallet_staking::SlashRewardFraction::<Runtime>::put(slash_reward_fraction);
    pallet_staking::MinNominatorBond::<Runtime>::put(10);
    pallet_staking::MinValidatorBond::<Runtime>::put(10);
    pallet_staking::MaxValidatorsCount::<Runtime>::put(20);
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
                grandpa: babe_to_grandpa_id(babe_id.clone())
                    .expect("Failed to map Babe ID to Grandpa ID"),
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

/// Grandpa keys are in a different encoding to Aura/Babe.
///
/// The pallet_session `KeyOwner` storage requires a mapping from Aura/Babe to
/// the grandpa key. We use this function to perform that mapping for all known keys.
///
/// The pub keys in this function were seeded from Alice, Bob, Charlie, Dave, Eve, Ferdie,
/// and known Bittensor devnet, testnet and finney authorities.
///
/// TODO: Double check these values.
fn babe_to_grandpa_id(babe: BabeAuthorityId) -> Option<GrandpaId> {
    let res = match babe.to_ss58check().as_str() {
        // Alice
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY" => {
            GrandpaId::from_ss58check("5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu").ok()
        }
        // Testnet Validator 1
        "5D5ABUyMsdmJdH7xrsz9vREq5eGXr5pXhHxix2dENQR62dEo" => {
            GrandpaId::from_ss58check("5H3qMjQjoeZxZ98jzDmoCwbz2sugd5fDN1wrr8Phf49zemKL").ok()
        }
        // Testnet Validator 2
        "5GbRc5sNDdhcPAU9suV2g9P5zyK1hjAQ9JHeeadY1mb8kXoM" => {
            GrandpaId::from_ss58check("5GbkysfaCjK3cprKPhi3CUwaB5xWpBwcfrkzs6FmqHxej8HZ").ok()
        }
        // Testnet Validator 3
        "5CoVWwBwXz2ndEChGcS46VfSTb3RMUZzZzAYdBKo263zDhEz" => {
            GrandpaId::from_ss58check("5HTLp4BvPp99iXtd8YTBZA1sMfzo8pd4mZzBJf7HYdCn2boU").ok()
        }
        // Testnet Validator 4
        "5EekcbqupwbgWqF8hWGY4Pczsxp9sbarjDehqk7bdyLhDCwC" => {
            GrandpaId::from_ss58check("5GAemcU4Pzyfe8DwLwDFx3aWzyg3FuqYUCCw2h4sdDZhyFvE").ok()
        }
        // Testnet Validator 5
        "5GgdEQyS5DZzUwKuyucEPEZLxFKGmasUFm1mqM3sx1MRC5RV" => {
            GrandpaId::from_ss58check("5EibpMomXmgekxcfs25SzFBpGWUsG9Lc8ALNjXN3TYH5Tube").ok()
        }
        // Testnet Validator 6
        "5Ek5JLCGk2PuoT1fS23GXiWYUT98HVUBERFQBu5g57sNf44x" => {
            GrandpaId::from_ss58check("5Gyrc6b2mx1Af6zWJYHdx3gwgtXgZvD9YkcG9uTUPYry4V2a").ok()
        }
        // Finney Validator 1
        "5EJUcFbe74FDQwPsZDbRVpdDxVZQQxjoGZA9ayJqJTbcRrGf" => {
            GrandpaId::from_ss58check("5GRcfchgXZjkCfqgNvfjicjJw3vVGF4Ahqon2w8RfjXwyzy4").ok()
        }
        // Finney Validator 2
        "5H5oVSbQxDSw1TohAvLvp9CTAua6PN4yHme19UrG4c1ojS8J" => {
            GrandpaId::from_ss58check("5FAEYaHLZmLRX4XFs2SBHbLhkysbSPrcTp51w6sQNaYLa7Tu").ok()
        }
        // Finney Validator 3
        "5CfBazEwCAsmscGj1J9rhXess9ZXZ5qYcuZvFWii9sxT977v" => {
            GrandpaId::from_ss58check("5F6LgDAenzchE5tPmFHKGueYy1rj85oB2yxvm1xyKLVvk4gy").ok()
        }
        // Finney Validator 4
        "5HZDvVFWH3ifx1Sx8Uaaa7oiT6U4fAKrR3LKy9r1zFnptc1z" => {
            GrandpaId::from_ss58check("5GJY6A1X8KNvqHcf42Cpr5HZzG95FZVJkTHJvnHSBGgshEWn").ok()
        }
        // Finney Validator 5
        "5H3v2VfQmsAAgj63EDaB1ZWmruTHHkJ4kci5wkt6SwMi2VW1" => {
            GrandpaId::from_ss58check("5FXVk1gEsNweTB6AvS5jAWCivXQHTcyCWXs21wHvRU5UTZtb").ok()
        }
        // Finney Validator 6
        "5CPhKdvHmMqRmMUrpFnvLc6GUcduVwpNHsPPEhnYQ7QXjPdz" => {
            GrandpaId::from_ss58check("5GAzG6PhVvpeoZVkKupa2uZDrhwsUmk5fCHgwq95cN9s3Dvi").ok()
        }
        // Finney Validator 7
        "5DZTjVhqVjHyhXLhommE4jqY9w1hJEKNQWJ8p6QnUWghRYS1" => {
            GrandpaId::from_ss58check("5HmGN73kkcHaKNJrSPAxwiwAiiCkztDZ1AYi4gkpv6jaWaxi").ok()
        }
        // Finney Validator 8
        "5ETyBUhi3uVCzsk4gyTmtf41nheH7wALqQQxbUkmRPNqEMGS" => {
            GrandpaId::from_ss58check("5Cq63ca5KM5qScJYmQi7PvFPhJ6Cxr6yw6Xg9dLYoRYg33rN").ok()
        }
        // Finney Validator 9
        "5DUSt6KiZWxA3tsiFkv3xYSNuox6PCfhyvqqM9x7N5kuHV2S" => {
            GrandpaId::from_ss58check("5FF1kun4rb5B7C3tqh23XPVDDUJ3UchnaXxJeXu1i5n8KNHp").ok()
        }
        // Finney Validator 10
        "5GgsDz9yixsdHxFu52SN37f6TrUtU2RwmGJejbHVmN1ERXL4" => {
            GrandpaId::from_ss58check("5EZiep2gMyV2cz9x54TQDb1cuyFYYcwGRGZ7J19Ua4YSAWCZ").ok()
        }
        // Finney Validator 11
        "5HjhkCMa89QJbFULs8WPZBgVg8kMq5qdX1nx7CnQpZgoyKAN" => {
            GrandpaId::from_ss58check("5D5DL9sru2ep3AWoHvmEUbFLirVr7tJ6BxBWH5M8j3r9kUpe").ok()
        }
        // Finney Validator 12
        "5F257gHitacwDGvYm2Xm7dBE882auTU8wraG6w4T3r63wh9V" => {
            GrandpaId::from_ss58check("5CovRCaioWENKejfaeccDQY4vCF8kTGtZ5fwagSCeDGmiSyh").ok()
        }
        // Finney Validator 13
        "5CtGLbiHWs6XVgNi9nW7oqSP4D4JMot7yHYuFokidZzAP6ny" => {
            GrandpaId::from_ss58check("5DSxsR9aAiq33uSYXWt4zEibx6KT6xxtFGkT9S4GLaCavgDE").ok()
        }
        // Finney Validator 14
        "5DeVtxyiniPzoHo4iQiLhGfhED6RP3V73B5nGSYWr5Mgt82c" => {
            GrandpaId::from_ss58check("5HaWL2AvLZHwyPXofWFTEZ6jHVmUG8U9cFATggKZonN1xZjm").ok()
        }
        // Finney Validator 15
        "5GF4a6pQ8TQuPhdkKqugzrZSW7YnpQtB4ihouKGZsVMwoTn6" => {
            GrandpaId::from_ss58check("5DaEhFN8bWjvhDxavSWFBr962qoTAMB4b51QebdRZ75VA4h2").ok()
        }
        // Finney Validator 16
        "5DAC8Did2NgeVfZeNmEfZuU6t7UseJNf9J68XTvhLf5yCsBZ" => {
            GrandpaId::from_ss58check("5G27pyXx9ieSRCTuDoqPgTvpCynH6yhum9HiQQ1iMj3rAeaP").ok()
        }
        // Finney Validator 17
        "5FmxaYznqMqiorPHQgKoRQgEHN7ud4yKsJWr6FvXuS6FS6be" => {
            GrandpaId::from_ss58check("5Ch5XFMKETDiiPiuhUj9TumUtgsnVG1VzQRvBykP9bRdt4km").ok()
        }
        // Finney Validator 18
        "5GNAkfKYmFbVRAYm1tPr1yG6bHCapaY7WKRmzkEdendDXj1j" => {
            GrandpaId::from_ss58check("5EC6JjwnE11qaRnjKM85eevQFV1EoaKPPtcBRmTp1XsR7Kx3").ok()
        }
        // Finney Validator 19
        "5GYk3B38R9F2TEcWoqCLojqPwx6AA1TsD3EovoTgggyRdzki" => {
            GrandpaId::from_ss58check("5FjdhdAxujZVev6HYqQcTB6UBAKfKFKPoftgMLenoxbNWoe2").ok()
        }
        // Finney Validator 20
        "5D7fthS7zBDhwi2u2JYd74t7FpQuseDkUkTuaLZoenXNpXPK" => {
            GrandpaId::from_ss58check("5DhAKQ4MFg39mQAYzndzbznLGqSV4VMUJUyRXe8QPDqD5G1D").ok()
        }
        // Greg Baedeker Alice
        "5G6okTjVk3urYHR1MyJLXmF6AtSvZ9qiwzWhCatNBXV9JMJd" => {
            GrandpaId::from_ss58check("5DRcVKY6Ccs6MotaMdaDDX7zzjeQ3V4LvyRJHDz4wgJhcC1K").ok()
        }
        // Greg Baedeker Bob
        "5Fy1xpe81NRpEBMc8h4wVwmcHBM3W7L6W16qLiaJpzcTVv7A" => {
            GrandpaId::from_ss58check("5F2BcLPjTQJWFgZZPCjR5YSb9CRUGqnKg7ZJFpgbaBrSXFrY").ok()
        }
        // Greg Baedeker Charlie
        "5Fe2G9aGa7izEN1XvFNc3eMGPbJBgswGScPbBVuMcwngcow8" => {
            GrandpaId::from_ss58check("5Gck7HYTpoK1qY6nYTDxBPPU1maSfkMnZBVY7Q2RqERoGrzX").ok()
        }
        _ => None,
    };

    match res {
        Some(grandpa_id) => {
            log::info!(
                "Successfully mapped BabeId {:?} to GrandpaId {:?}",
                babe.to_ss58check(),
                grandpa_id.to_ss58check()
            );
            Some(grandpa_id)
        }
        None => {
            log::error!(
                "Failed to map Babe authority {:?} to Grandpa authority!!!",
                babe.to_ss58check()
            );
            None
        }
    }
}

use crate::*;

pub struct Migration;

impl OnRuntimeUpgrade for Migration {
    fn on_runtime_upgrade() -> Weight {
        pos_upgrade()
    }
}
