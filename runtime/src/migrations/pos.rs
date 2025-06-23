use crate::opaque::SessionKeys;
use babe_primitives::AuthorityId as BabeAuthorityId;
use babe_primitives::AuthorityId as BabeId;
use babe_primitives::BabeAuthorityWeight;
// use core::str::FromStr;
use frame_support::WeakBoundedVec;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::OnRuntimeUpgrade;
use pallet_aura;
use pallet_babe;
use scale_info::prelude::string::String;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_consensus_slots::Slot;
use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;
use sp_runtime::SaturatedConversion;
use sp_runtime::traits::OpaqueKeys;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;

pub(crate) fn populate_babe() -> Weight {
    // Initialize weight counter
    // TODO: Compute weight correctly
    let weight = <Runtime as frame_system::Config>::DbWeight::get().reads(1);

    // Nothing to do if we have already migrated to Babe.
    //
    // This check is critical for the runtime upgrade to be idempotent!
    if !pallet_babe::Authorities::<Runtime>::get().len().is_zero() {
        return weight;
    }

    let authorities = pallet_aura::Authorities::<Runtime>::get();
    let authorities: Vec<(BabeAuthorityId, BabeAuthorityWeight)> = authorities
        .into_iter()
        .map(|a| {
            // BabeAuthorityId and AuraId are both sr25519::Public, so can convert between with
            // Encode/Decode.
            let encoded: Vec<u8> = a.encode();
            log::info!(
                "Converting Aura authority {:?} to Babe authority",
                array_bytes::bytes2hex("", &a)
            );
            let decoded: BabeAuthorityId =
                BabeAuthorityId::decode(&mut &encoded[..]).expect("Failed to decode authority");
            log::info!(
                "Decoded Babe authority: {:?}",
                array_bytes::bytes2hex("", &decoded)
            );
            (decoded, 1)
        })
        .collect::<Vec<_>>();
    let bounded_authorities =
        WeakBoundedVec::<_, <Runtime as pallet_babe::Config>::MaxAuthorities>::try_from(
            authorities.to_vec(),
        )
        .expect("Initial number of authorities should be lower than T::MaxAuthorities");

    log::info!("Set {} into bounded authorites", bounded_authorities.len());
    pallet_babe::SegmentIndex::<Runtime>::put(0);
    pallet_babe::Authorities::<Runtime>::put(&bounded_authorities);
    pallet_babe::NextAuthorities::<Runtime>::put(&bounded_authorities);
    pallet_babe::EpochConfig::<Runtime>::put(BABE_GENESIS_EPOCH_CONFIG);

    //     2025-06-17 13:11:31 panicked at /Users/liamaharon/grimoire/polkadot-sdk/substrate/frame/babe/src/lib.rs:938:3:
    // assertion `left == right` failed: Timestamp slot must match `CurrentSlot`
    //   left: Slot(7000490749) // current slot
    //  right: Slot(7000490765) // timestamp slot

    let now = pallet_timestamp::Now::<Runtime>::get();
    let slot_duration = pallet_babe::Pallet::<Runtime>::slot_duration();
    let timestamp_slot = now / slot_duration;
    let timestamp_slot = Slot::from(timestamp_slot.saturated_into::<u64>());

    log::info!(
        "now: {:?}, slot_duration: {:?}, timestamp_slot: {:?}",
        &now,
        &slot_duration,
        &timestamp_slot
    );

    // let current_slot = pallet_aura::CurrentSlot::<Runtime>::get();
    pallet_babe::CurrentSlot::<Runtime>::put(timestamp_slot.saturating_add(1u64));

    // TODO: Init session pallet
    let ss58_authorities = authorities
        .iter()
        .map(|a| a.0.to_ss58check())
        .collect::<Vec<_>>();
    initialize_pallet_session(ss58_authorities);

    // TODO: Init Staking pallet

    // Brick the Aura pallet so no new Aura blocks can be produced after this runtime upgrade.
    let _ = pallet_aura::Authorities::<Runtime>::take();

    weight
}

fn initialize_pallet_session(ss58_authorities: Vec<String>) {
    log::info!(
        "Initializing pallet_session with authorities: {:?}",
        ss58_authorities
    );

    let keys: Vec<(AccountId32, SessionKeys)> = ss58_authorities
        .into_iter()
        .map(|ss58| {
            let account = AccountId32::from_ss58check(&ss58).unwrap();
            let keys = SessionKeys {
                babe: BabeId::from_ss58check(&ss58).unwrap(),
                grandpa: GrandpaId::from_ss58check(&ss58).unwrap(),
            };
            // let babe: BabeId = Ss58Codec::from_ss58check(&ss58).unwrap();
            // let grandpa: GrandpaId = Ss58Codec::from_ss58check(&ss58).unwrap();
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
            // fn put_key_owner(id: KeyTypeId, key_data: &[u8], v: &T::ValidatorId) {
            // KeyOwner::<T>::insert((id, key_data), v)
            // }
            // for i in new_ids.iter() {
            // 	Self::put_key_owner(*i, new_keys.get_raw(*i), &val);
            // }
        }
    }
    pallet_session::QueuedKeys::<Runtime>::put(keys);

    // pallet_session::KeyOwner::<Runtime>::put(
    //     keys.iter()
    //         .map(|(account, _)| (account.clone(), account.clone()))
    //         .collect::<Vec<_>>(),
    // );

    // Set NextKeys and KeyOwner.
    //  for (account, val, keys) in keys.iter().cloned()
    // {
    // 	pallet_session::Pallet::<Runtime>::inner_set_keys(&val, keys)
    // 		.expect("genesis config must not contain duplicates; qed");
    // 	if frame_system::Pallet::<Runtime>::inc_consumers_without_limit(&account).is_err() {
    // 		// This will leak a provider reference, however it only happens once (at
    // 		// genesis) so it's really not a big deal and we assume that the user wants to
    // 		// do this since it's the only way a non-endowed account can contain a session
    // 		// key.
    // 		frame_system::Pallet::<Runtime>::inc_providers(&account);
    // 	}
    // }
}

use crate::*;

pub struct Migration;

impl OnRuntimeUpgrade for Migration {
    /// Performs the migration to initialize and update the total issuance.
    ///
    /// This function does the following:
    /// 1. Calculates the total locked tokens across all subnets
    /// 2. Retrieves the total account balances and total stake
    /// 3. Computes and updates the new total issuance
    ///
    /// Returns the weight of the migration operation.
    fn on_runtime_upgrade() -> Weight {
        populate_babe()
    }

    /// Performs post-upgrade checks to ensure the migration was successful.
    ///
    /// This function is only compiled when the "try-runtime" feature is enabled.
    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        // TODO: impl
        Ok(())
    }
}
