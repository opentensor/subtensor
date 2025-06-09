// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

pub mod devnet;
pub mod finney;
pub mod localnet;
pub mod testnet;

use babe_primitives::AuthorityId as BabeId;
use node_subtensor_runtime::{Block, WASM_BINARY};
use sc_chain_spec_derive::ChainSpecExtension;
use sc_service::ChainType;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::crypto::Ss58Codec;
use sp_core::{H256, Pair, Public, bounded_vec, sr25519};
use sp_runtime::AccountId32;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use subtensor_runtime_common::{AccountId, Signature};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthorityKeys {
    account: AccountId,
    babe: BabeId,
    grandpa: GrandpaId,
    authority_discovery: AuthorityDiscoveryId,
}

impl AuthorityKeys {
    pub fn new(
        account: AccountId,
        babe: BabeId,
        grandpa: GrandpaId,
        authority_discovery: AuthorityDiscoveryId,
    ) -> Self {
        Self {
            account,
            babe,
            grandpa,
            authority_discovery,
        }
    }

    pub fn account(&self) -> &AccountId {
        &self.account
    }

    pub fn babe(&self) -> &BabeId {
        &self.babe
    }

    pub fn grandpa(&self) -> &GrandpaId {
        &self.grandpa
    }

    pub fn authority_discovery(&self) -> &AuthorityDiscoveryId {
        &self.authority_discovery
    }
}

fn get_authority_keys_from_seed(seed: &str) -> AuthorityKeys {
    AuthorityKeys::new(
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

// pub fn get_authority_keys_from_ss58(
//     account: &str,
//     babe: &str,
//     grandpa: &str,
//     authority_discovery: &str,
// ) -> AuthorityKeys {
//     AuthorityKeys::new(
//         AccountId32::from_str(account).unwrap(),
//         get_from_ss58_addr::<BabeId>(babe),
//         get_from_ss58_addr::<GrandpaId>(grandpa),
//         get_from_ss58_addr::<AuthorityDiscoveryId>(authority_discovery),
//     )
// }
//
// pub fn get_from_ss58_addr<TPublic: Public>(addr: &str) -> <TPublic::Pair as Pair>::Public {
//     Ss58Codec::from_ss58check(addr).unwrap()
// }

// Includes for nakamoto genesis
use serde::{Deserialize, Serialize};
use serde_json as json;
use std::{fs::File, path::PathBuf};

// Configure storage from nakamoto data
#[derive(Deserialize, Debug)]
struct ColdkeyHotkeys {
    stakes: std::collections::HashMap<String, std::collections::HashMap<String, (u64, u16)>>,
    balances: std::collections::HashMap<String, u64>,
}
