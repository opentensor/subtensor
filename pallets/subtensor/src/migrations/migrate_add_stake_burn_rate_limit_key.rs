use alloc::string::String;
use codec::{Decode, Encode, EncodeLike, Error as CodecError, Input, Output};
use frame_support::{
    pallet_prelude::{Identity, OptionQuery},
    storage_alias,
    traits::Get,
    weights::Weight,
};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use subtensor_runtime_common::NetUid;

use crate::{
    AccountIdOf, Config, HasMigrationRun, LastRateLimitedBlock, Pallet, RateLimitKey, SubnetOwner,
};

const MIGRATION_NAME: &[u8] = b"migrate_add_stake_burn_rate_limit_key";

#[allow(dead_code)]
#[derive(Decode, Encode, TypeInfo)]
enum RateLimitKeyV0<AccountId> {
    SetSNOwnerHotkey(NetUid),
    OwnerHyperparamUpdate(NetUid, crate::Hyperparameter),
    NetworkLastRegistered,
    LastTxBlock(AccountId),
    LastTxBlockChildKeyTake(AccountId),
    LastTxBlockDelegateTake(AccountId),
    AddStakeBurn(NetUid),
}

#[allow(dead_code)]
#[derive(TypeInfo)]
enum LegacyRateLimitKey<AccountId> {
    Legacy(RateLimitKeyV0<AccountId>),
    Other,
}

impl<AccountId: Encode> Encode for LegacyRateLimitKey<AccountId> {
    fn size_hint(&self) -> usize {
        match self {
            Self::Legacy(key) => key.size_hint(),
            Self::Other => 0,
        }
    }

    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        if let Self::Legacy(key) = self {
            key.encode_to(dest);
        }
    }
}

impl<AccountId: Encode> EncodeLike for LegacyRateLimitKey<AccountId> {}

impl<AccountId: Decode> Decode for LegacyRateLimitKey<AccountId> {
    fn decode<I: Input>(input: &mut I) -> Result<Self, CodecError> {
        let key = RateLimitKeyV0::<AccountId>::decode(input)?;
        if input.remaining_len()?.unwrap_or(0) == 0 {
            Ok(Self::Legacy(key))
        } else {
            Ok(Self::Other)
        }
    }
}

pub mod deprecated {
    use super::*;

    #[storage_alias]
    pub(super) type LastRateLimitedBlock<T: Config> =
        StorageMap<Pallet<T>, Identity, LegacyRateLimitKey<AccountIdOf<T>>, u64, OptionQuery>;
}

pub fn migrate_add_stake_burn_rate_limit_key<T: Config>() -> Weight {
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(MIGRATION_NAME) {
        log::info!(
            "Migration '{}' already executed - skipping",
            String::from_utf8_lossy(MIGRATION_NAME)
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(MIGRATION_NAME)
    );

    let mut migrated_entries = Vec::new();
    let mut translated = 0u64;

    deprecated::LastRateLimitedBlock::<T>::translate::<u64, _>(|key, block| {
        translated = translated.saturating_add(1);
        if let LegacyRateLimitKey::Legacy(RateLimitKeyV0::AddStakeBurn(netuid)) = key {
            migrated_entries.push((netuid, block));
            None
        } else {
            Some(block)
        }
    });
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

    for (netuid, legacy_value) in &migrated_entries {
        let owner = SubnetOwner::<T>::get(*netuid);
        weight = weight.saturating_add(T::DbWeight::get().reads(1));

        let new_key = RateLimitKey::AddStakeBurn(*netuid, owner);
        let merged_value = core::cmp::max(LastRateLimitedBlock::<T>::get(&new_key), *legacy_value);
        weight = weight.saturating_add(T::DbWeight::get().reads(1));

        LastRateLimitedBlock::<T>::insert(&new_key, merged_value);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    HasMigrationRun::<T>::insert(MIGRATION_NAME, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{}' completed. migrated={}",
        String::from_utf8_lossy(MIGRATION_NAME),
        migrated_entries.len()
    );

    weight
}
