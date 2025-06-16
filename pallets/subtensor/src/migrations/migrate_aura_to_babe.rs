use super::*;
use babe_primitives::AuthorityId as BabeAuthorityId;
use babe_primitives::BabeAuthorityWeight;
use frame_support::WeakBoundedVec;
use frame_support::pallet_prelude::{Identity, OptionQuery, Weight};
use frame_support::storage_alias;
use pallet_aura;
use pallet_babe;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_std::vec::Vec;

/// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
/// The choice of is done in accordance to the slot duration and expected target
/// block time, for safely resisting network delays of maximum two seconds.
/// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: babe_primitives::BabeEpochConfiguration =
    babe_primitives::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: babe_primitives::AllowedSlots::PrimaryAndSecondaryVRFSlots,
    };

// TODO: Implement comprehensive tests

/// Module containing deprecated storage format for LoadedEmission
pub mod deprecated_loaded_emission_format {
    use super::*;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

pub(crate) fn populate_babe<
    T: Config + pallet_babe::Config + pallet_aura::Config<AuthorityId = AuraId>,
>() -> Weight {
    // Initialize weight counter
    // TODO: Compute weight correctly
    let weight = T::DbWeight::get().reads(1);

    let authorities = pallet_aura::Authorities::<T>::get();
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
        WeakBoundedVec::<_, <T as pallet_babe::Config>::MaxAuthorities>::try_from(
            authorities.to_vec(),
        )
        .expect("Initial number of authorities should be lower than T::MaxAuthorities");

    pallet_babe::SegmentIndex::<T>::put(0);
    pallet_babe::Authorities::<T>::put(&bounded_authorities);
    pallet_babe::NextAuthorities::<T>::put(&bounded_authorities);
    pallet_babe::EpochConfig::<T>::put(BABE_GENESIS_EPOCH_CONFIG);

    let current_slot = pallet_aura::CurrentSlot::<T>::get();
    pallet_babe::CurrentSlot::<T>::put(current_slot);

    weight
}

pub mod aura_to_babe {
    use frame_support::pallet_prelude::Weight;
    use frame_support::traits::OnRuntimeUpgrade;
    use sp_consensus_aura::sr25519::AuthorityId as AuraId;

    use crate::*;

    pub struct Migration<T: Config + pallet_babe::Config + pallet_aura::Config<AuthorityId = AuraId>>(
        PhantomData<T>,
    );

    impl<T: Config + pallet_babe::Config + pallet_aura::Config<AuthorityId = AuraId>>
        OnRuntimeUpgrade for Migration<T>
    {
        /// Performs the migration to initialize and update the total issuance.
        ///
        /// This function does the following:
        /// 1. Calculates the total locked tokens across all subnets
        /// 2. Retrieves the total account balances and total stake
        /// 3. Computes and updates the new total issuance
        ///
        /// Returns the weight of the migration operation.
        fn on_runtime_upgrade() -> Weight {
            super::populate_babe::<T>()
        }

        /// Performs post-upgrade checks to ensure the migration was successful.
        ///
        /// This function is only compiled when the "try-runtime" feature is enabled.
        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            // Verify that all accounting invariants are satisfied after the migration
            crate::Pallet::<T>::check_total_issuance()?;
            Ok(())
        }
    }
}
