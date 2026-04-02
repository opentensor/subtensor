use sp_core::sr25519;
// use sp_consensus_aura::sr25519::AuthorityId as AuraId;
// use sp_consensus_grandpa::AuthorityId as GrandpaId;

use node_subtensor::chain_spec::*;
use serde_json::Value;

#[test]
fn test_get_from_seed() {
    let seed = "WoOt";
    let pare = get_from_seed::<sr25519::Public>(seed);
    let expected = "5Gj3QEiZaFJPFK1yN4Lkj6FLM4V7GEBCewVBVniuvZ75S2Fd";
    assert_eq!(pare.to_string(), expected);
}

#[test]
#[should_panic(expected = "static values are valid; qed: InvalidFormat")]
fn test_get_from_seed_panics() {
    let bad_seed = "";
    get_from_seed::<sr25519::Public>(bad_seed);
}

#[test]
fn test_get_account_id_from_seed() {
    let seed = "WoOt";
    let account_id = get_account_id_from_seed::<sr25519::Public>(seed);
    let expected = "5Gj3QEiZaFJPFK1yN4Lkj6FLM4V7GEBCewVBVniuvZ75S2Fd";
    assert_eq!(account_id.to_string(), expected);
}

#[test]
#[should_panic(expected = "static values are valid; qed: InvalidFormat")]
fn test_get_account_id_from_seed_panics() {
    let bad_seed = "";
    get_account_id_from_seed::<sr25519::Public>(bad_seed);
}

#[test]
fn test_authority_keys_from_seed() {
    let seed = "WoOt";
    let (aura_id, grandpa_id) = authority_keys_from_seed(seed);

    let expected_aura_id = "5Gj3QEiZaFJPFK1yN4Lkj6FLM4V7GEBCewVBVniuvZ75S2Fd";
    let expected_grandpa_id = "5H7623Nvxq655p9xrLQPip1mwssFRMfL5fvT5LUSa4nWwLSm";

    assert_eq!(aura_id.to_string(), expected_aura_id);
    assert_eq!(grandpa_id.to_string(), expected_grandpa_id);
}

#[test]
#[should_panic(expected = "static values are valid; qed: InvalidFormat")]
fn test_authority_keys_from_seed_panics() {
    let bad_seed = "";
    authority_keys_from_seed(bad_seed);
}

#[test]
fn test_finney_testnet_chain_spec_protocol_id() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_node-subtensor"))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .args([
            "build-spec",
            "--chain",
            "test_finney",
            "--disable-default-bootnode",
        ])
        .output()
        .expect("node-subtensor build-spec should run");

    assert!(
        output.status.success(),
        "build-spec failed: {:?}",
        output.status
    );

    let spec_json: Value =
        serde_json::from_slice(&output.stdout).expect("build-spec should emit valid json");

    assert_eq!(
        spec_json.get("protocolId").and_then(Value::as_str),
        Some("bittensor-testnet")
    );
}

#[test]
fn test_checked_in_plain_testnet_spec_protocol_id() {
    let spec_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../chainspecs/plain_spec_testfinney.json"
    );
    let spec_json: Value = serde_json::from_str(
        &std::fs::read_to_string(spec_path).expect("plain testnet spec should exist"),
    )
    .expect("plain testnet spec should be valid json");

    assert_eq!(
        spec_json.get("protocolId").and_then(Value::as_str),
        Some("bittensor-testnet")
    );
}
