use node_subtensor::chain_spec::*;
use sp_core::sr25519;

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
    let keys = AuthorityKeys::from_seed(seed);

    let expected_babe_id = "5Gj3QEiZaFJPFK1yN4Lkj6FLM4V7GEBCewVBVniuvZ75S2Fd";
    let expected_grandpa_id = "5H7623Nvxq655p9xrLQPip1mwssFRMfL5fvT5LUSa4nWwLSm";

    assert_eq!(keys.babe().to_string(), expected_babe_id);
    assert_eq!(keys.grandpa().to_string(), expected_grandpa_id);
}

#[test]
#[should_panic(expected = "static values are valid; qed: InvalidFormat")]
fn test_authority_keys_from_seed_panics() {
    let bad_seed = "";
    AuthorityKeys::from_seed(bad_seed);
}
