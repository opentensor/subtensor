use node_subtensor_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, SubtensorModuleConfig
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_core::crypto::Ss58Codec;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn authority_keys_from_ss58(s_aura :&str, s_grandpa : &str) -> (AuraId, GrandpaId) {
	(
		get_aura_from_ss58_addr(s_aura),
		get_grandpa_from_ss58_addr(s_grandpa),
	)
}

pub fn get_aura_from_ss58_addr(s: &str) -> AuraId {
	Ss58Codec::from_ss58check(s).unwrap()
}

pub fn get_grandpa_from_ss58_addr(s: &str) -> GrandpaId {
	Ss58Codec::from_ss58check(s).unwrap()
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

// Includes for nakamoto genesis
use std::{fs::File, path::PathBuf};
use serde::{Deserialize};
use serde_json as json;

// Configure storage from nakamoto data
#[derive(Deserialize, Debug)]
struct ColdkeyHotkeys {
	stakes: std::collections::HashMap<String, std::collections::HashMap<String, u64>>,
	balances: std::collections::HashMap<String, u64>
}

pub fn finney_config() -> Result<ChainSpec, String> {
	let path: PathBuf = std::path::PathBuf::from("/Users/ec2-user/repos/subtensorv3/nakamoto_gen.json");
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
		json::from_slice(&bytes).map_err(|e| format!("Error parsing genesis file: {}", e))?;

	let mut processed_stakes: Vec<(sp_runtime::AccountId32, Vec<(sp_runtime::AccountId32, u64)>)> = Vec::new();
	for (coldkey_str, hotkeys) in old_state.stakes.iter() {
		let coldkey = <sr25519::Public as Ss58Codec>::from_ss58check(&coldkey_str).unwrap();
		let coldkey_account = sp_runtime::AccountId32::from(coldkey);

		let mut processed_hotkeys: Vec<(sp_runtime::AccountId32, u64)> = Vec::new();

		for (hotkey_str, amount) in hotkeys.iter() {
			let hotkey = <sr25519::Public as Ss58Codec>::from_ss58check(&hotkey_str).unwrap();
			let hotkey_account = sp_runtime::AccountId32::from(hotkey);

			processed_hotkeys.push((hotkey_account, *amount));
		}

		processed_stakes.push((coldkey_account, processed_hotkeys));
	}

	let mut processed_balances: Vec<(sp_runtime::AccountId32, u64)> = Vec::new();
	for (key_str, amount) in old_state.balances.iter() {
		let key = <sr25519::Public as Ss58Codec>::from_ss58check(&key_str).unwrap();
		let key_account = sp_runtime::AccountId32::from(key);

		processed_balances.push((key_account, *amount))
	}

	Ok(ChainSpec::from_genesis(
		// Name
		"Finney",
		// ID
		"Finney",
		ChainType::Development,
		move || {
			finney_genesis(
				wasm_binary,
				// Initial PoA authorities (Validators)
				// aura | grandpa
				vec![
					// Keys for debug
					//authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob"),
					authority_keys_from_ss58("5EJUcFbe74FDQwPsZDbRVpdDxVZQQxjoGZA9ayJqJTbcRrGf", "5GRcfchgXZjkCfqgNvfjicjJw3vVGF4Ahqon2w8RfjXwyzy4"),// key 1
					authority_keys_from_ss58("5H5oVSbQxDSw1TohAvLvp9CTAua6PN4yHme19UrG4c1ojS8J", "5FAEYaHLZmLRX4XFs2SBHbLhkysbSPrcTp51w6sQNaYLa7Tu"), // key 2
					authority_keys_from_ss58("5CfBazEwCAsmscGj1J9rhXess9ZXZ5qYcuZvFWii9sxT977v", "5F6LgDAenzchE5tPmFHKGueYy1rj85oB2yxvm1xyKLVvk4gy"), // key 3
					authority_keys_from_ss58("5HZDvVFWH3ifx1Sx8Uaaa7oiT6U4fAKrR3LKy9r1zFnptc1z", "5GJY6A1X8KNvqHcf42Cpr5HZzG95FZVJkTHJvnHSBGgshEWn"), // key 4
					/*authority_keys_from_ss58("5H3v2VfQmsAAgj63EDaB1ZWmruTHHkJ4kci5wkt6SwMi2VW1", "5FXVk1gEsNweTB6AvS5jAWCivXQHTcyCWXs21wHvRU5UTZtb"), // key 5
					authority_keys_from_ss58("5CPhKdvHmMqRmMUrpFnvLc6GUcduVwpNHsPPEhnYQ7QXjPdz", "5GAzG6PhVvpeoZVkKupa2uZDrhwsUmk5fCHgwq95cN9s3Dvi"), // key 6
					authority_keys_from_ss58("5DZTjVhqVjHyhXLhommE4jqY9w1hJEKNQWJ8p6QnUWghRYS1", "5HmGN73kkcHaKNJrSPAxwiwAiiCkztDZ1AYi4gkpv6jaWaxi"), // key 7
					authority_keys_from_ss58("5ETyBUhi3uVCzsk4gyTmtf41nheH7wALqQQxbUkmRPNqEMGS", "5Cq63ca5KM5qScJYmQi7PvFPhJ6Cxr6yw6Xg9dLYoRYg33rN"), // key 8
					authority_keys_from_ss58("5DUSt6KiZWxA3tsiFkv3xYSNuox6PCfhyvqqM9x7N5kuHV2S", "5FF1kun4rb5B7C3tqh23XPVDDUJ3UchnaXxJeXu1i5n8KNHp"), // key 9
					authority_keys_from_ss58("5GgsDz9yixsdHxFu52SN37f6TrUtU2RwmGJejbHVmN1ERXL4", "5EZiep2gMyV2cz9x54TQDb1cuyFYYcwGRGZ7J19Ua4YSAWCZ"), // key 10
					authority_keys_from_ss58("5HjhkCMa89QJbFULs8WPZBgVg8kMq5qdX1nx7CnQpZgoyKAN", "5D5DL9sru2ep3AWoHvmEUbFLirVr7tJ6BxBWH5M8j3r9kUpe"), // key 11
					authority_keys_from_ss58("5F257gHitacwDGvYm2Xm7dBE882auTU8wraG6w4T3r63wh9V", "5CovRCaioWENKejfaeccDQY4vCF8kTGtZ5fwagSCeDGmiSyh"), // key 12
					authority_keys_from_ss58("5CtGLbiHWs6XVgNi9nW7oqSP4D4JMot7yHYuFokidZzAP6ny", "5DSxsR9aAiq33uSYXWt4zEibx6KT6xxtFGkT9S4GLaCavgDE"), // key 13
					authority_keys_from_ss58("5DeVtxyiniPzoHo4iQiLhGfhED6RP3V73B5nGSYWr5Mgt82c", "5HaWL2AvLZHwyPXofWFTEZ6jHVmUG8U9cFATggKZonN1xZjm"), // key 14
					authority_keys_from_ss58("5GF4a6pQ8TQuPhdkKqugzrZSW7YnpQtB4ihouKGZsVMwoTn6", "5DaEhFN8bWjvhDxavSWFBr962qoTAMB4b51QebdRZ75VA4h2"), // key 15
					authority_keys_from_ss58("5DAC8Did2NgeVfZeNmEfZuU6t7UseJNf9J68XTvhLf5yCsBZ", "5G27pyXx9ieSRCTuDoqPgTvpCynH6yhum9HiQQ1iMj3rAeaP"), // key 16
					authority_keys_from_ss58("5FmxaYznqMqiorPHQgKoRQgEHN7ud4yKsJWr6FvXuS6FS6be", "5Ch5XFMKETDiiPiuhUj9TumUtgsnVG1VzQRvBykP9bRdt4km"), // key 17
					authority_keys_from_ss58("5GNAkfKYmFbVRAYm1tPr1yG6bHCapaY7WKRmzkEdendDXj1j", "5EC6JjwnE11qaRnjKM85eevQFV1EoaKPPtcBRmTp1XsR7Kx3"), // key 18
					authority_keys_from_ss58("5GYk3B38R9F2TEcWoqCLojqPwx6AA1TsD3EovoTgggyRdzki", "5FjdhdAxujZVev6HYqQcTB6UBAKfKFKPoftgMLenoxbNWoe2"), // key 19
					authority_keys_from_ss58("5D7fthS7zBDhwi2u2JYd74t7FpQuseDkUkTuaLZoenXNpXPK", "5DhAKQ4MFg39mQAYzndzbznLGqSV4VMUJUyRXe8QPDqD5G1D"), // key 20*/
					], 
				// Sudo account
				Ss58Codec::from_ss58check("5G3rrAtAsZiZsRKhi9y7yckwhB6HL8AwD8K3J28YNX4n2tMZ").unwrap(), 
				// Pre-funded accounts
				vec![
				],
				true,
				processed_stakes.clone(),
				processed_balances.clone()
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		subtensor_module: Default::default(),
	}
}

// Configure initial storage state for FRAME modules.
fn finney_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	_endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
	stakes: Vec<(AccountId, Vec<(AccountId, u64)>)>,
	_balances: Vec<(AccountId, u64)>

) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			//balances: balances.iter().cloned().map(|k| k).collect(),
			balances: vec![ 
				(Ss58Codec::from_ss58check("5G3rrAtAsZiZsRKhi9y7yckwhB6HL8AwD8K3J28YNX4n2tMZ").unwrap(),10000000000)
				],
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		subtensor_module: SubtensorModuleConfig {
			stakes: stakes
		},
	}
}
