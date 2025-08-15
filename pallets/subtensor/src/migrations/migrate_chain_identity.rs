use crate::alloc::borrow::ToOwned;
use codec::Decode;
use scale_info::prelude::{string::String, vec::Vec};
use serde::Deserialize;
use sp_core::{ConstU32, crypto::Ss58Codec};
use sp_runtime::{AccountId32, BoundedVec};

use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;

#[derive(Deserialize, Debug)]
struct RegistrationRecordJSON {
    address: String,
    name: String,
    url: String,
    description: String,
}

fn string_to_bounded_vec(input: &str) -> Result<BoundedVec<u8, ConstU32<1024>>, &'static str> {
    let vec_u8: Vec<u8> = input.to_owned().into_bytes();

    // Check if the length is within bounds
    if vec_u8.len() > 64 {
        return Err("Input string is too long");
    }

    // Convert to BoundedVec
    BoundedVec::<u8, ConstU32<1024>>::try_from(vec_u8)
        .map_err(|_| "Failed to convert to BoundedVec")
}

pub fn migrate_set_hotkey_identities<T: Config>() -> Weight {
    let migration_name = b"migrate_identities".to_vec();

    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    // Check if the migration has already run
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }
    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    // Include the JSON file with delegate info
    let data = include_str!("../../../../docs/delegate-info.json");

    // Iterate over all the delegate records
    if let Ok(delegates) = serde_json::from_str::<Vec<RegistrationRecordJSON>>(data) {
        // Iterate through the delegates
        for delegate in delegates.iter() {
            // Convert fields to bounded vecs
            let name_result = string_to_bounded_vec(&delegate.name);
            let desc_result = string_to_bounded_vec(&delegate.description);
            let url_result = string_to_bounded_vec(&delegate.url);
            let hotkey: AccountId32 = match AccountId32::from_ss58check(&delegate.address) {
                Ok(account) => account,
                Err(_) => {
                    log::warn!(
                        "Invalid SS58 address: {:?}. Skipping this delegate.",
                        delegate.address
                    );
                    continue;
                }
            };
            let decoded_hotkey: T::AccountId = match T::AccountId::decode(&mut hotkey.as_ref()) {
                Ok(decoded) => decoded,
                Err(e) => {
                    log::warn!("Failed to decode hotkey: {e:?}. Skipping this delegate.");
                    continue;
                }
            };
            log::info!("Hotkey unwrapped: {decoded_hotkey:?}");

            // If we should continue with real values.
            let mut name: BoundedVec<u8, ConstU32<1024>> = BoundedVec::default();
            let mut description: BoundedVec<u8, ConstU32<1024>> = BoundedVec::default();
            let mut url: BoundedVec<u8, ConstU32<1024>> = BoundedVec::default();
            if let Ok(n) = name_result {
                name = n;
            }
            if let Ok(d) = desc_result {
                description = d;
            }
            if let Ok(u) = url_result {
                url = u;
            }

            // Unwrap the real values.
            let image: BoundedVec<u8, ConstU32<1024>> = BoundedVec::default();
            let discord: BoundedVec<u8, ConstU32<1024>> = BoundedVec::default();
            let additional: BoundedVec<u8, ConstU32<1024>> = BoundedVec::default();

            // Create the chain identity.
            let identity = ChainIdentityOf {
                name: name.into(),
                url: url.into(),
                image: image.into(),
                discord: discord.into(),
                description: description.into(),
                additional: additional.into(),
            };

            // Log the identity details
            log::info!("Setting identity for hotkey: {hotkey:?}");
            log::info!("Name: {:?}", String::from_utf8_lossy(&identity.name));
            log::info!("URL: {:?}", String::from_utf8_lossy(&identity.url));
            log::info!("Image: {:?}", String::from_utf8_lossy(&identity.image));
            log::info!("Discord: {:?}", String::from_utf8_lossy(&identity.discord));
            log::info!(
                "Description: {:?}",
                String::from_utf8_lossy(&identity.description)
            );
            log::info!(
                "Additional: {:?}",
                String::from_utf8_lossy(&identity.additional)
            );

            // Check validation.
            let total_length = identity
                .name
                .len()
                .saturating_add(identity.url.len())
                .saturating_add(identity.image.len())
                .saturating_add(identity.discord.len())
                .saturating_add(identity.description.len())
                .saturating_add(identity.additional.len());
            let is_valid: bool = total_length <= 256 + 256 + 1024 + 256 + 1024 + 1024
                && identity.name.len() <= 256
                && identity.url.len() <= 256
                && identity.image.len() <= 1024
                && identity.discord.len() <= 256
                && identity.description.len() <= 1024
                && identity.additional.len() <= 1024;
            if !is_valid {
                log::info!("Bytes not correct");
                continue;
            }

            // Get the owning coldkey.
            let coldkey = Owner::<T>::get(decoded_hotkey.clone());
            log::info!("ColdKey: {decoded_hotkey:?}");

            weight = weight.saturating_add(T::DbWeight::get().reads(1));

            // Sink into the map.
            Identities::<T>::insert(coldkey.clone(), identity.clone());
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
        }
    } else {
        log::info!("Failed to decode JSON");
    }
    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed. Storage version set to 7.",
        String::from_utf8_lossy(&migration_name)
    );

    // Return the migration weight.
    weight
}
