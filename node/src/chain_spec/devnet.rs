// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;

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
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
            get_authority_keys_from_seed("Charlie"),
        ],
        // Sudo account
        Ss58Codec::from_ss58check("5GpzQgpiAKHMWNSH3RN4GLf96GVTDct9QxYEFAY7LWcVzTbx").unwrap(),
        // Pre-funded accounts
        vec![],
        true,
        vec![],
        vec![],
        0,
    ))
    .with_properties(properties)
    .build())
}

// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn devnet_genesis(
    initial_authorities: Vec<AuthorityKeys>,
    root_key: AccountId,
    _endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
    _stakes: Vec<(AccountId, Vec<(AccountId, (u64, u16))>)>,
    _balances: Vec<(AccountId, u64)>,
    _balances_issuance: u64,
) -> serde_json::Value {
    serde_json::json!({
        "balances": {
            "balances": vec![(root_key.clone(), 1_000_000_000_000u128)],
        },
        "session": {
            "keys": initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.account(),
                        x.account(),
                        node_subtensor_runtime::opaque::SessionKeys {
                            babe: x.babe().clone(),
                            grandpa: x.grandpa().clone(),
                            authority_discovery: x.authority_discovery().clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(root_key),
        },
    })
}
