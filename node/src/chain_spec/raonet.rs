// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;

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

fn raonet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _enable_println: bool,
) -> serde_json::Value {
    let mut balances = vec![
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FRo4vab84LM3aiK4DijnVawGDKagLGLzfn95j9tjDaHja8Z").unwrap(),
            100_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EeBuJRFUMS3CgisL1FT2w4AdqSQVGWRGNsTdR5YrFd189PT").unwrap(), // Greg
            250_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5H5wqFQp2Kq6C9mJJpymRmeywxdYXp5hfWTtPM4NKhFG77jr").unwrap(), // Formalized
            250_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DSTAKEJvuJTxZeg9Cew8JicA2Se8FPgHup6rDgXDoSbMwTc").unwrap(), // Sam
            250_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DD7xG5TJ48W7j7DUbHG5whrAkwUrbPe1NahiFbKqzytfhVt").unwrap(), // Spigot
            250_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HjEUemUaXSkxPcxGYiLykHmi5VfXBh5NCeNXYMbj9akYHbn").unwrap(), // Prop
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DUFdkP4rJrkXq9pfrWMHQS8zgiwXBZRgw2MMEAnBot59Taz").unwrap(), // Bob
            250_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CkV7PPFWh8EihTK5uLm7VNF4C9hiKJ9UeJJwQuByn3bx82L").unwrap(), // Carrot
            200_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5FREUpwG4wJYr1Usyp1i5XmxM3ycGTfvtjJtrSZETPgvh4Hx").unwrap(), // Roman
            200_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HTukLb2y59rrL5tM9RMw3baziCFZUeSZATWMWHgSrNknc9A").unwrap(), // Gus
            200_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5GxREgL1Kvuv1kixbY2oJe36Q2HnWqTcfakBZqTPKWiPpVxf").unwrap(), // Abe
            200_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5EssvMunCjyXhYFW8CzWKTdP4vjARAGsqU6wExSBfwJQLisy").unwrap(), // Dan
            200_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DLXQLrVZVRiBWCMduwhnFkEYLeoq7HDeR2Q4iLXQWmJYcj7").unwrap(), // Watchmaker
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5DiJqQoQdpgKaLz97Fk8ZkChAxMqf2mF2pVa6xDKYH8Cf9Sx").unwrap(), // Xavier
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5ECP53D9KTArYC3qJoURv86tT3d8G6JRUf5SHu1rejdD43uR").unwrap(), // Dick
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CXgmrp6Ts5igz9uxSdQQy9ERUVaJFtswzaSBUXhb3Ci7drK").unwrap(), // Special K
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5G4FseBtaQd8sqeC98ZEL7xgtF2GSdueMXwUs8vsBENs4Ysn").unwrap(), // Sai
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5CoSC9nRDT4CbnEGLCHcC8PxgpJsWpSdF3RLnTx2aBvX3qPu").unwrap(), // Faybian
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HDpGZLNYSxHGWVhsPgKRDKVM6oob7MMnwxdpU8dBP7N51dX").unwrap(), // Michal
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5HKxefbizpcKgozj6LmB7b2sh6AnPcAixmNwfPpHz897bFRP").unwrap(), // William
            333_000_000_000u128,
        ),
        (
            <AccountId32 as Ss58Codec>::from_ss58check("5Dc1mP2ebCMXeGTPKgN8deToR9sb3VB2EX1uB6eASC5SLfvo").unwrap(), // Fowwest
            333_000_000_000u128,
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
