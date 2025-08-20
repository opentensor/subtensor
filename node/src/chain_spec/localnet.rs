// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use node_subtensor_runtime::{BABE_GENESIS_EPOCH_CONFIG, UNITS};
use pallet_staking::Forcing;
use sp_runtime::Perbill;
use sp_staking::StakerStatus;
use std::collections::HashMap;

use super::*;

pub fn localnet_config(single_authority: bool) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // Give front-ends necessary data to present to users
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    Ok(ChainSpec::builder(
        wasm_binary,
        Extensions {
            bad_blocks: Some(HashSet::from_iter(vec![
                // Example bad block
                H256::from_str(
                    "0xc174d485de4bc3813ac249fe078af605c74ff91d07b0a396cf75fa04f81fa312",
                )
                .unwrap(),
            ])),
            ..Default::default()
        },
    )
    .with_name("Bittensor")
    .with_protocol_id("bittensor")
    .with_id("bittensor")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(localnet_genesis(
        // Initial NPoS authorities
        if single_authority {
            // single authority allows you to run the network using a single node
            vec![AuthorityKeys::from_seed("Alice")]
        } else {
            vec![
                AuthorityKeys::from_seed("Alice"),
                AuthorityKeys::from_seed("Bob"),
                AuthorityKeys::from_seed("Charlie"),
            ]
        },
        // Pre-funded accounts
        true,
    ))
    .with_properties(properties)
    .build())
}

fn localnet_genesis(
    initial_authorities: Vec<AuthorityKeys>,
    _enable_println: bool,
) -> serde_json::Value {
    let mut balances: HashMap<AccountId, u128> = vec![
        (
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            1000000000000000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            1000000000000000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            1000000000000000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            2000000000000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            2000000000000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            2000000000000u128,
        ),
        // ETH
        (
            // Alith - 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac
            AccountId::from_str("5Fghzk1AJt88PeFEzuRfXzbPchiBbsVGTTXcdx599VdZzkTA").unwrap(),
            2000000000000u128,
        ),
        (
            // Baltathar - 0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0
            AccountId::from_str("5GeqNhKWj1KG78VHzbmo3ZjZgUTrCuWeamdgiA114XHGdaEr").unwrap(),
            2000000000000u128,
        ),
    ]
    .into_iter()
    .collect();

    for a in initial_authorities.iter() {
        balances.insert(a.account().clone(), 2000000000000u128);
    }

    // Check if the environment variable is set
    if let Ok(bt_wallet) = env::var("BT_DEFAULT_TOKEN_WALLET") {
        if let Ok(decoded_wallet) = Ss58Codec::from_ss58check(&bt_wallet) {
            balances.insert(decoded_wallet, 1_000_000_000_000_000u128);
        } else {
            eprintln!("Invalid format for BT_DEFAULT_TOKEN_WALLET.");
        }
    }

    let trimvirate_members: Vec<AccountId> = bounded_vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
    ];

    let senate_members: Vec<AccountId> = bounded_vec![
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
    ];

    const STAKE: u64 = 1000 * UNITS;
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
            "key": Some(get_account_id_from_seed::<sr25519::Public>("Alice"))
        },
        "triumvirateMembers": {
            "members": trimvirate_members
        },
        "senateMembers": {
            "members": senate_members,
        },
        "evmChainId": {
            "chainId": 42,
        },
    })
}
