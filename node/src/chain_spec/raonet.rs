// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;

/// Generates the configuration for the Raonet chain.
///
/// # Returns
///
/// * `Result<ChainSpec, String>` - The chain specification or an error message.
///
pub fn raonet_config() -> Result<ChainSpec, String> {
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
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(raonet_genesis(
        // Initial PoA authorities (Validators)
        // aura | grandpa
        vec![
            // Keys for debug
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        // Pre-funded accounts
        true,
    ))
    .with_properties(properties)
    .build())
}

/// Generates the genesis configuration for the Raonet chain.
///
/// # Arguments
///
/// * `initial_authorities` - A vector of initial authorities (AuraId, GrandpaId).
/// * `_enable_println` - A boolean flag to enable println (currently unused).
///
/// # Returns
///
/// * `serde_json::Value` - The genesis configuration as a JSON value.
fn raonet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _enable_println: bool,
) -> serde_json::Value {
    let mut balances = vec![
        // Add Alice, Bob, and Charlie with 10 trillion tokens each
        (
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            10_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            10_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            10_000_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5FRo4vab84LM3aiK4DijnVawGDKagLGLzfn95j9tjDaHja8Z",
            )
            .unwrap(),
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5H3qhPGzKMNV9fTPuizxzp8azyFRMd4BnheSuwN9Qxb5Cz3u",
            )
            .unwrap(), // Greg
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5H8zkg8K9hkM5PSeumivNXGuK8J8cUjtwEL9PfyqmgWELPka",
            )
            .unwrap(), // William
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5Ckg2z5NdrfbXpsPtZuVYXxRWh283QWw1gbXfN8CJC3tmxnY",
            )
            .unwrap(), // Dick
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5CiUxGA5dTh1cPdgH67Kt62x4w5aKubnmWZMYSrZzoB4hpQi",
            )
            .unwrap(), // Michal
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5FLHD4eZkPStKUG7p9B1VPjD4w93Fxncf6JG5EK2uRcELmJy",
            )
            .unwrap(), // Carlos
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5G4FseBtaQd8sqeC98ZEL7xgtF2GSdueMXwUs8vsBENs4Ysn",
            )
            .unwrap(), // Sai
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5DiJqQoQdpgKaLz97Fk8ZkChAxMqf2mF2pVa6xDKYH8Cf9Sx",
            )
            .unwrap(), // Xavier
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5EUJyRh3x9QftnUw7taFk3Xen6fCQgdc9ko8ort51RnR6LCn",
            )
            .unwrap(), // Elo
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5FREUpwG4wJYr1Usyp1i5XmxM3ycGTfvtjJtrSZETPgvh4Hx",
            )
            .unwrap(), // Roman
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5GxREgL1Kvuv1kixbY2oJe36Q2HnWqTcfakBZqTPKWiPpVxf",
            )
            .unwrap(), // Abe
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5GsSxM8p1TYrpXfCx7Un5cTp1fr1RwHJczYDUUn8Xjnqj9Sk",
            )
            .unwrap(), // Nico
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5HjEUemUaXSkxPcxGYiLykHmi5VfXBh5NCeNXYMbj9akYHbn",
            )
            .unwrap(), // Jip
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5DUFdkP4rJrkXq9pfrWMHQS8zgiwXBZRgw2MMEAnBot59Taz",
            )
            .unwrap(), // Bob
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5CoSC9nRDT4CbnEGLCHcC8PxgpJsWpSdF3RLnTx2aBvX3qPu",
            )
            .unwrap(), // Faybian
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5CXgmrp6Ts5igz9uxSdQQy9ERUVaJFtswzaSBUXhb3Ci7drK",
            )
            .unwrap(), // Special K
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5HDpGZLNYSxHGWVhsPgKRDKVM6oob7MMnwxdpU8dBP7N51dX",
            )
            .unwrap(), // Michal
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5GH3haJWuJjcZWuC7iFGtaVajJNEpNg2Guaqyf71y9uDfFrt",
            )
            .unwrap(), // Isa
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5HTukLb2y59rrL5tM9RMw3baziCFZUeSZATWMWHgSrNknc9A",
            )
            .unwrap(), // Gus
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5CkV7PPFWh8EihTK5uLm7VNF4C9hiKJ9UeJJwQuByn3bx82L",
            )
            .unwrap(), // Carrot
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check(
                "5DV8tTjq8EXE6KmoCbJ3xaN54HTXsfev5ZyKJEQPyTcm4MmE",
            )
            .unwrap(), // Paul
            100_000_000_000u128,
        ),
    ];

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
    })
}
