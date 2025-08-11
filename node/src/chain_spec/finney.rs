// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;
use hex::FromHex;
use subtensor_runtime_common::keys::known_ss58;

pub fn finney_mainnet_config() -> Result<ChainSpec, String> {
    let path: PathBuf = std::path::PathBuf::from("./snapshot.json");
    let wasm_binary = WASM_BINARY.ok_or("Development wasm not available".to_string())?;

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

    let mut processed_stakes: Vec<(
        sp_runtime::AccountId32,
        Vec<(sp_runtime::AccountId32, (u64, u16))>,
    )> = Vec::new();
    for (coldkey_str, hotkeys) in old_state.stakes.iter() {
        let coldkey = <sr25519::Public as Ss58Codec>::from_ss58check(coldkey_str)
            .map_err(|e| e.to_string())?;
        let coldkey_account = sp_runtime::AccountId32::from(coldkey);

        let mut processed_hotkeys: Vec<(sp_runtime::AccountId32, (u64, u16))> = Vec::new();

        for (hotkey_str, amount_uid) in hotkeys.iter() {
            let (amount, uid) = amount_uid;
            let hotkey = <sr25519::Public as Ss58Codec>::from_ss58check(hotkey_str)
                .map_err(|e| e.to_string())?;
            let hotkey_account = sp_runtime::AccountId32::from(hotkey);

            processed_hotkeys.push((hotkey_account, (*amount, *uid)));
        }
        processed_hotkeys.sort();

        processed_stakes.push((coldkey_account, processed_hotkeys));
    }

    processed_stakes.sort();

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
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    let chain_spec = ChainSpec::builder(
        wasm_binary,
        Extensions {
            bad_blocks: Some(HashSet::new()),
            ..Default::default()
        },
    )
    .with_name("Bittensor")
    .with_protocol_id("bittensor")
    .with_id("bittensor")
    .with_chain_type(ChainType::Live)
	.with_boot_nodes(vec![
        "/dns/bootnode.finney.chain.opentensor.ai/tcp/30333/ws/p2p/12D3KooWRwbMb85RWnT8DSXSYMWQtuDwh4LJzndoRrTDotTR5gDC"
            .parse()
            .unwrap(),
    ])
    .with_genesis_config_patch(finney_genesis(
        // Initial validators
        vec![
            AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_1),
            AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_2),
            AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_3),
            AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_4),
            AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_5),
            AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_6),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_7),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_8),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_9),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_10),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_11),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_12),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_13),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_14),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_15),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_16),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_17),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_18),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_19),
			AuthorityKeys::from_known_ss58(known_ss58::FINNEY_VALI_20),
        ],
        // Sudo account
        Ss58Codec::from_ss58check("5FCM3DBXWiGcwYYQtT8z4ZD93TqYpYxjaAfgv6aMStV1FTCT").unwrap(),
        // Pre-funded accounts
        vec![],
        true,
        processed_stakes.clone(),
        processed_balances.clone(),
        balances_issuance,
    ))
    .with_properties(properties)
    .build();

    // Load and set the code substitute to avoid archive node sync panic
    // See <https://github.com/opentensor/subtensor/pull/1051>
    //
    // Need to do it in this hacky way because the ChainSpec builder doesn't support setting it
    let code_substitute_2585476_hex = include_bytes!("code_substitute_2585476.txt");
    let chain_spec_json = chain_spec.as_json(false).unwrap();
    let mut chain_spec_json = serde_json::from_str(&chain_spec_json).unwrap();
    sc_chain_spec::set_code_substitute_in_json_chain_spec(
        &mut chain_spec_json,
        Vec::from_hex(code_substitute_2585476_hex)
            .unwrap()
            .as_slice(),
        2585476,
    );
    let chain_spec_bytes = chain_spec_json.to_string().into_bytes();
    let chain_spec = ChainSpec::from_json_bytes(chain_spec_bytes).unwrap();
    Ok(chain_spec)
}

// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn finney_genesis(
    initial_authorities: Vec<AuthorityKeys>,
    _root_key: AccountId,
    _endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
    stakes: Vec<(AccountId, Vec<(AccountId, (u64, u16))>)>,
    balances: Vec<(AccountId, u64)>,
    balances_issuance: u64,
) -> serde_json::Value {
    serde_json::json!({
        "balances": { "balances": balances.to_vec() },
        "aura": { "authorities": initial_authorities.iter().map(|x| (x.account().clone())).collect::<Vec<_>>() },
        "grandpa": { "authorities": initial_authorities
                .iter()
                .map(|x| (x.grandpa().clone(), 1))
                .collect::<Vec<_>>(),
        },
        "sudo": { "key": Some(<AccountId32 as Ss58Codec>::from_ss58check("5FCM3DBXWiGcwYYQtT8z4ZD93TqYpYxjaAfgv6aMStV1FTCT").unwrap()) },
        "subtensorModule": {
            "stakes": stakes,
            "balancesIssuance": balances_issuance,
        }
    })
}
