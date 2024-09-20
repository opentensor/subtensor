// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;

pub fn localnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // Give front-ends necessary data to present to users
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());
    let genesis = localnet_genesis(
        // Initial PoA authorities (Validators)
        // aura | grandpa
        vec![
            // Keys for debug
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        // Pre-funded accounts
        false,
    );

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
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(genesis)
    .with_properties(properties)
    .build())
}

fn localnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _enable_println: bool,
) -> serde_json::Value {
    let mut balances = vec![
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
    ];

    let root_validator = (
        get_account_id_from_seed::<sr25519::Public>("RootValidator"),
        // 10000 TAO
        10_000_000_000_000_u128,
    );

    let subnet_validator = (
        get_account_id_from_seed::<sr25519::Public>("SubnetValidator"),
        // 2000 TAO
        2_000_000_000_000_u128,
    );

    let miners = [
        (
            get_account_id_from_seed::<sr25519::Public>("Miner1"),
            // 10 TAO
            10_000_000_000_u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Miner2"),
            // 10 TAO
            10_000_000_000_u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Miner3"),
            // 10 TAO
            10_000_000_000_u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Miner4"),
            // 10 TAO
            10_000_000_000_u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Miner5"),
            // 10 TAO
            10_000_000_000_u128,
        ),
    ];

    balances.push(root_validator.clone());
    balances.push(subnet_validator.clone());
    balances.append(&mut miners.to_vec());

    // Check if the environment variable is set
    if let Ok(bt_wallet) = env::var("BT_DEFAULT_TOKEN_WALLET") {
        if let Ok(decoded_wallet) = Ss58Codec::from_ss58check(&bt_wallet) {
            balances.push((decoded_wallet, 1_000_000_000_000_000u128));
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

    let alice_account = get_account_id_from_seed::<sr25519::Public>("Alice");
    serde_json::json!({
        "balances": { "balances": balances },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>()
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>()
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
        "subtensorModule": {
            "initializeNetwork1": true,
            "initializeNetwork3": false,
            "rootColdkeyValidator": Some(vec![alice_account.clone(), root_validator.0]),
            "subnetColdkeyValidator": Some(vec![alice_account.clone(), subnet_validator.0]),
            "miners": Some(vec![
                (alice_account.clone(), miners[0].0.clone()),
                (alice_account.clone(), miners[1].0.clone()),
                (alice_account.clone(), miners[2].0.clone()),
                (alice_account.clone(), miners[3].0.clone()),
                (alice_account.clone(), miners[4].0.clone()),
            ]),
        },
    })
}
