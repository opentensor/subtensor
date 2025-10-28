// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use super::*;
use node_subtensor_runtime::{BABE_GENESIS_EPOCH_CONFIG, UNITS};
use pallet_staking::Forcing;
use sp_runtime::Perbill;
use sp_staking::StakerStatus;
use subtensor_runtime_common::keys::known_ss58;

pub fn devnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // Give front-ends necessary data to present to users
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "testTAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    Ok(ChainSpec::builder(
        wasm_binary,
        Extensions {
            bad_blocks: Some(HashSet::new()),
            ..Default::default()
        },
    )
    .with_name("Bittensor")
    .with_protocol_id("bittensor")
    .with_id("bittensor")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(devnet_genesis(
        vec![
            // Devnet keys
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_1),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_2),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_3),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_4),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_5),
        ],
        // Devnet sudo
        Ss58Codec::from_ss58check("5GpzQgpiAKHMWNSH3RN4GLf96GVTDct9QxYEFAY7LWcVzTbx").unwrap(),
        // Pre-funded accounts
        vec![],
    ))
    .with_properties(properties)
    .build())
}

// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn devnet_genesis(
    initial_authorities: Vec<AuthorityKeys>,
    root_key: AccountId,
    balances: Vec<(AccountId, u64)>,
) -> serde_json::Value {
    const STAKE: u64 = 1000 * UNITS;

    let mut balances: HashMap<AccountId, u64> = balances.into_iter().collect();
    for a in initial_authorities.iter() {
        let bal = balances.get(a.account()).unwrap_or(&0);
        balances.insert(a.account().clone(), bal.saturating_add(1500 * UNITS));
    }
    serde_json::json!({
        "balances": { "balances": balances.into_iter().collect::<Vec<_>>() },
        "session": {
            "keys": initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.account().clone(),
                        x.account().clone(),
                        node_subtensor_runtime::opaque::SessionKeys {
                            babe: x.babe().clone(),
                            grandpa: x.grandpa().clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        },
        "staking": {
            "minimumValidatorCount": 1,
            "validatorCount": initial_authorities.len() as u32,
            "stakers": initial_authorities
                .iter()
                .map(|x| (x.account().clone(), x.account().clone(), STAKE, StakerStatus::<AccountId>::Validator))
                .collect::<Vec<_>>(),
            "invulnerables": initial_authorities.iter().map(|x| x.account().clone()).collect::<Vec<_>>(),
            "forceEra": Forcing::NotForcing,
            "slashRewardFraction": Perbill::from_percent(10),
        },
        "babe": {
            "epochConfig": BABE_GENESIS_EPOCH_CONFIG,
        },
        "sudo": {
            "key": Some(root_key),
        },
    })
}
