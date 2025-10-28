// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::expect_used, clippy::unwrap_used)]

pub mod devnet;
pub mod finney;
pub mod localnet;
pub mod testnet;

use node_subtensor_runtime::{Block, WASM_BINARY};
use sc_chain_spec_derive::ChainSpecExtension;
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::crypto::Ss58Codec;
use sp_core::{H256, Pair, Public, ed25519, sr25519};
use sp_runtime::AccountId32;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use subtensor_runtime_common::keys::KnownSs58;
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
    TPublic::Pair::from_string(&format!("//{seed}"), None)
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
}

impl AuthorityKeys {
    pub fn new(account: AccountId, babe: BabeId, grandpa: GrandpaId) -> Self {
        Self {
            account,
            babe,
            grandpa,
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

    pub fn from_seed(seed: &str) -> AuthorityKeys {
        AuthorityKeys::new(
            get_account_id_from_seed::<sr25519::Public>(seed),
            get_from_seed::<BabeId>(seed),
            get_from_seed::<GrandpaId>(seed),
        )
    }

    pub fn from_known_ss58(known: KnownSs58) -> AuthorityKeys {
        let sr25519_pub = ss58_to_public::<sr25519::Public>(known.sr25519);
        let ed25519_pub = ss58_to_public::<ed25519::Public>(known.ed25519);
        // Account and Babe are SR25519, Grandpa is ED25519
        AuthorityKeys::new(
            AccountId32::from(sr25519_pub),
            BabeId::from(sr25519_pub),
            GrandpaId::from(ed25519_pub),
        )
    }
}

fn ss58_to_public<TPublic: Public>(addr: &str) -> <TPublic::Pair as Pair>::Public {
    Ss58Codec::from_ss58check(addr).expect("Invalid SS58 address")
}

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
