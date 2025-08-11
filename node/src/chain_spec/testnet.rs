// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use subtensor_runtime_common::keys::known_ss58;

use super::*;

pub fn finney_testnet_config() -> Result<ChainSpec, String> {
    let path: PathBuf = std::path::PathBuf::from("./snapshot.json");
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // We mmap the file into memory first, as this is *a lot* faster than using
    // `serde_json::from_reader`. See https://github.com/serde-rs/json/issues/160
    let file = File::open(&path)
        .map_err(|e| format!("Error opening genesis file `{}`: {}", path.display(), e))?;

    // SAFETY: `mmap` is fundamentally unsafe since technically the file can change
    //         underneath us while it is mapped; in practice it's unlikely to be a problem
    let bytes = unsafe {
        memmap2::Mmap::map(&file)
            .map_err(|e| format!("Error mmaping genesis file `{}`: {}", path.display(), e))?
    };

    let old_state: ColdkeyHotkeys =
        json::from_slice(&bytes).map_err(|e| format!("Error parsing genesis file: {e}"))?;

    let mut balances_issuance: u64 = 0;
    let mut processed_balances: Vec<(sp_runtime::AccountId32, u64)> = Vec::new();
    for (key_str, amount) in old_state.balances.iter() {
        let key =
            <sr25519::Public as Ss58Codec>::from_ss58check(key_str).map_err(|e| e.to_string())?;
        let key_account = sp_runtime::AccountId32::from(key);

        processed_balances.push((key_account, *amount));
        balances_issuance = balances_issuance
            .checked_add(*amount)
            .ok_or("Balances issuance overflowed".to_string())?;
    }
    processed_balances.sort();

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
    .with_boot_nodes(vec![
        "/dns/bootnode.test.chain.opentensor.ai/tcp/30333/p2p/12D3KooWPM4mLcKJGtyVtkggqdG84zWrd7Rij6PGQDoijh1X86Vr"
            .parse()
            .unwrap(),
    ])
    .with_protocol_id("bittensor")
    .with_id("bittensor")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(testnet_genesis(
        // Initial validators
        vec![
            // Testnet keys
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_1),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_2),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_3),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_4),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_5),
            AuthorityKeys::from_known_ss58(known_ss58::TESTNET_VALI_6),
        ],
        // Sudo account
        Ss58Codec::from_ss58check("5GpzQgpiAKHMWNSH3RN4GLf96GVTDct9QxYEFAY7LWcVzTbx").unwrap(),
        // Pre-funded accounts
        vec![],
        true,
        vec![],
        processed_balances,
        balances_issuance,
    ))
    .with_properties(properties)
    .build())
}

// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
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
            // Configure sudo balance
            "balances": vec![(root_key.clone(), 1_000_000_000_000u128)],
        },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| (x.account().clone())).collect::<Vec<_>>(),
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|x| (x.grandpa().clone(), 1))
                .collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(root_key),
        },
    })
}
