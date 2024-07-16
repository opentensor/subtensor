use codec::Decode;
use scale_info::prelude::{
    string::{String, ToString},
    vec::Vec,
};
use serde::Deserialize;
use sp_core::{crypto::Ss58Codec, ConstU32};
use sp_runtime::{AccountId32, BoundedVec};
use sp_std::vec;

use super::*;
use frame_support::{
    traits::{Get, GetStorageVersion, StorageVersion},
    weights::Weight,
};
use log;

#[derive(Deserialize, Debug)]
struct RegistrationRecordJSON {
    address: String,
    name: String,
    url: String,
    description: String,
}

fn string_to_bounded_vec(input: &String) -> Result<BoundedVec<u8, ConstU32<64>>, &'static str> {
    let vec_u8: Vec<u8> = input.clone().into_bytes();

    // Check if the length is within bounds
    if vec_u8.len() > 64 {
        return Err("Input string is too long");
    }

    // Convert to BoundedVec
    BoundedVec::<u8, ConstU32<64>>::try_from(vec_u8).map_err(|_| "Failed to convert to BoundedVec")
}

pub fn migrate_set_hotkey_identities<T: Config>() -> Weight {
    let new_storage_version = 1;
    let migration_name = "set hotkey identities";
    let mut weight = T::DbWeight::get().reads_writes(1, 1);

    let title = "description".to_string();

    let onchain_version = Pallet::<T>::on_chain_storage_version();
    log::info!("Current on-chain storage version: {:?}", onchain_version);
    if onchain_version < new_storage_version {
        log::info!("Starting migration: {}.", migration_name);

        // Include the JSON file with delegate info
        let data = include_str!("../../../docs/delegate-info.json");

        // Deserialize the JSON data into a HashMap
        if let Ok(delegates) = serde_json::from_str::<Vec<RegistrationRecordJSON>>(data) {
            log::info!("{} delegate records loaded", delegates.len());

            // Iterate through the delegates
            for delegate in delegates.iter() {
                // Convert fields to bounded vecs
                let name_result = string_to_bounded_vec(&delegate.name);
                let desc_result = string_to_bounded_vec(&truncate_string(&delegate.description));
                let url_result = string_to_bounded_vec(&delegate.url);

                // Convert string address into AccountID
                let maybe_account_id_32 = AccountId32::from_ss58check(&delegate.address);
                let account_id = if maybe_account_id_32.is_ok() {
                    let account_id_32 = maybe_account_id_32.unwrap();
                    if let Ok(acc_id) = T::AccountId::decode(&mut account_id_32.as_ref()) {
                        Some(acc_id)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if name_result.is_ok()
                    && desc_result.is_ok()
                    && url_result.is_ok()
                    && account_id.is_some()
                {
                    let desc_title = Data::Raw(string_to_bounded_vec(&title).unwrap());
                    let desc_data = Data::Raw(desc_result.unwrap());
                    let desc_item = BoundedVec::try_from(vec![(desc_title, desc_data)]).unwrap();

                    let info: IdentityInfo<T::MaxAdditionalFields> = IdentityInfo {
                        display: Data::Raw(name_result.unwrap()),
                        additional: desc_item,
                        legal: Data::None,
                        web: Data::Raw(url_result.unwrap()),
                        riot: Data::None,
                        email: Data::None,
                        pgp_fingerprint: None,
                        image: Data::None,
                        twitter: Data::None,
                    };

                    // Insert delegate hotkeys info
                    let reg: Registration<BalanceOf<T>, T::MaxAdditionalFields> = Registration {
                        deposit: Zero::zero(),
                        info,
                    };

                    IdentityOf::<T>::insert(account_id.unwrap(), reg);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
                } else {
                    log::info!(
                        "Migration {} couldn't be completed, bad JSON item for: {}",
                        migration_name,
                        delegate.address
                    );
                    if !name_result.is_ok() {
                        log::info!("Name is bad");
                    }
                    if !desc_result.is_ok() {
                        log::info!("Description is bad");
                    }
                    if !url_result.is_ok() {
                        log::info!("URL is bad");
                    }
                    if !account_id.is_some() {
                        log::info!("Account ID is bad");
                    }
                }
            }
        } else {
            log::info!(
                "Migration {} couldn't be completed, bad JSON file: {}",
                migration_name,
                data
            );
            return weight;
        }

        StorageVersion::new(new_storage_version).put::<Pallet<T>>();
    } else {
        log::info!("Migration already done: {}", migration_name);
    }

    log::info!("Final weight: {:?}", weight);
    weight
}

fn truncate_string(s: &str) -> String {
    let max_len: usize = 64;
    if s.len() > max_len {
        s.chars().take(max_len).collect()
    } else {
        s.to_string()
    }
}
