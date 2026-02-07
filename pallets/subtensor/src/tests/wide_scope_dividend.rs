#![allow(
    unused,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::arithmetic_side_effects
)]

use super::mock::*;
use crate::*;
use frame_support::assert_ok;
use sp_core::U256;
use substrate_fixed::types::{I64F64, I96F32, U96F32};
use subtensor_runtime_common::{AlphaCurrency, MechId, NetUid, TaoCurrency};

/// Asserts that `value` is within `eps` of `target` (absolute difference).
fn close(value: u64, target: u64, eps: u64) {
    assert!(
        (value as i128 - target as i128).unsigned_abs() < eps as u128,
        "close assertion failed: value = {value}, target = {target}, eps = {eps}, diff = {}",
        (value as i128 - target as i128).abs()
    );
}

// ===========================
// Neuron identity constants
// ===========================

// Subnet 1 owner
const OWNER1_HK: u64 = 10;
const OWNER1_CK: u64 = 110;

// Subnet 2 owner
const OWNER2_HK: u64 = 20;
const OWNER2_CK: u64 = 120;

// Root validators (registered in both subnets)
const MAJOR_ROOT_HK: u64 = 1;
const MAJOR_ROOT_CK: u64 = 101;
const MINOR_ROOT_HK: u64 = 2;
const MINOR_ROOT_CK: u64 = 102;

// Subnet 1 validators and miner
const MAJOR_SN1_HK: u64 = 11;
const MAJOR_SN1_CK: u64 = 111;
const MINOR_SN1_HK: u64 = 12;
const MINOR_SN1_CK: u64 = 112;
const MINER1_HK: u64 = 13;
const MINER1_CK: u64 = 113;

// Subnet 2 validators and miner
const MAJOR_SN2_HK: u64 = 21;
const MAJOR_SN2_CK: u64 = 121;
const MINOR_SN2_HK: u64 = 22;
const MINOR_SN2_CK: u64 = 122;
const MINER2_HK: u64 = 23;
const MINER2_CK: u64 = 123;

// Stake amounts
const OWNER_ALPHA: u64 = 1_000;
const MAJOR_SUBNET_ALPHA: u64 = 999_000;
const MINOR_SUBNET_ALPHA: u64 = 1_000;
const MAJOR_ROOT_TAO: u64 = 5_550_000;
const MINOR_ROOT_TAO: u64 = 5_556;

// Test setup result
struct TestSetup {
    netuid1: NetUid,
    netuid2: NetUid,
}

/// Creates 2 subnets and registers all neurons with the specified stakes.
/// SN1 has tempo=1 (epochs at blocks 3, 5, 7...), SN2 has tempo=0 (never fires).
/// BlockAtRegistration is set to 0 for all SN1 neurons so weights set at block 1
/// are not masked by the epoch's weight masking logic.
///
/// Per SN1 UIDs:
///   0 = subnet owner validator (1,000 alpha, does NOT set weights)
///   1 = major root validator (0 alpha here, 5,550,000 TAO on root)
///   2 = minor root validator (0 alpha here, 5,556 TAO on root)
///   3 = major subnet validator (999,000 alpha)
///   4 = minor subnet validator (1,000 alpha)
///   5 = miner (0 stake)
fn setup_test() -> TestSetup {
    // ----------- Create two subnets -----------
    let netuid1 =
        add_dynamic_network_disable_commit_reveal(&U256::from(OWNER1_HK), &U256::from(OWNER1_CK));
    let netuid2 =
        add_dynamic_network_disable_commit_reveal(&U256::from(OWNER2_HK), &U256::from(OWNER2_CK));
    log::info!(
        "Created subnets: netuid1={:?}, netuid2={:?}",
        netuid1,
        netuid2
    );

    // ----------- Subnet parameters -----------
    // SN1: tempo=1, epochs fire at blocks 3, 5, 7... for netuid=1
    SubtensorModule::set_tempo(netuid1, 1);
    // SN2: tempo=0 means it never fires epochs; it only exists for flow share
    SubtensorModule::set_tempo(netuid2, 0);

    for &netuid in &[netuid1, netuid2] {
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_min_allowed_weights(netuid, 1);
        SubtensorModule::set_max_allowed_validators(netuid, 5);
        SubtensorModule::set_activity_cutoff(netuid, 5000);
        SubtensorModule::set_max_registrations_per_block(netuid, 100);
        SubtensorModule::set_target_registrations_per_interval(netuid, 100);
    }

    // ----------- Subnet reserves for price 0.5 (default, tests can override) -----------
    let tao_reserve = TaoCurrency::from(500_000u64);
    let alpha_reserve = AlphaCurrency::from(1_000_000u64);
    setup_reserves(netuid1, tao_reserve, alpha_reserve);
    setup_reserves(netuid2, tao_reserve, alpha_reserve);

    SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.5));
    SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(0.5));

    // ----------- Subnet flow EMA = 0.001 -----------
    let now = SubtensorModule::get_current_block_as_u64();
    SubnetEmaTaoFlow::<Test>::insert(netuid1, (now, I64F64::from_num(0.001)));
    SubnetEmaTaoFlow::<Test>::insert(netuid2, (now, I64F64::from_num(0.001)));

    // ----------- TaoWeight ≈ 0.18 -----------
    TaoWeight::<Test>::set(u64::MAX / 100 * 18);

    // ----------- Subnet owner cut = 18% -----------
    SubtensorModule::set_subnet_owner_cut(u16::MAX / 100 * 18);

    // ----------- Enable EffectiveRootPropEmissionScaling -----------
    EffectiveRootPropEmissionScaling::<Test>::set(true);

    // ----------- Register neurons -----------
    // SN1
    register_ok_neuron(
        netuid1,
        U256::from(MAJOR_ROOT_HK),
        U256::from(MAJOR_ROOT_CK),
        0,
    );
    register_ok_neuron(
        netuid1,
        U256::from(MINOR_ROOT_HK),
        U256::from(MINOR_ROOT_CK),
        10,
    );
    register_ok_neuron(
        netuid1,
        U256::from(MAJOR_SN1_HK),
        U256::from(MAJOR_SN1_CK),
        20,
    );
    register_ok_neuron(
        netuid1,
        U256::from(MINOR_SN1_HK),
        U256::from(MINOR_SN1_CK),
        30,
    );
    register_ok_neuron(netuid1, U256::from(MINER1_HK), U256::from(MINER1_CK), 40);

    // SN2
    register_ok_neuron(
        netuid2,
        U256::from(MAJOR_ROOT_HK),
        U256::from(MAJOR_ROOT_CK),
        50,
    );
    register_ok_neuron(
        netuid2,
        U256::from(MINOR_ROOT_HK),
        U256::from(MINOR_ROOT_CK),
        60,
    );
    register_ok_neuron(
        netuid2,
        U256::from(MAJOR_SN2_HK),
        U256::from(MAJOR_SN2_CK),
        70,
    );
    register_ok_neuron(
        netuid2,
        U256::from(MINOR_SN2_HK),
        U256::from(MINOR_SN2_CK),
        80,
    );
    register_ok_neuron(netuid2, U256::from(MINER2_HK), U256::from(MINER2_CK), 90);

    // ----------- Fix BlockAtRegistration for SN1 -----------
    // Set to 0 so weights at block 1 have last_update=1 > 0=block_at_registration
    // and are NOT masked by epoch's vec_mask_sparse_matrix.
    for uid in 0..6u16 {
        BlockAtRegistration::<Test>::insert(netuid1, uid, 0u64);
    }

    // ----------- Add alpha stakes -----------
    // SN1
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &U256::from(OWNER1_HK),
        &U256::from(OWNER1_CK),
        netuid1,
        AlphaCurrency::from(OWNER_ALPHA),
    );
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &U256::from(MAJOR_SN1_HK),
        &U256::from(MAJOR_SN1_CK),
        netuid1,
        AlphaCurrency::from(MAJOR_SUBNET_ALPHA),
    );
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &U256::from(MINOR_SN1_HK),
        &U256::from(MINOR_SN1_CK),
        netuid1,
        AlphaCurrency::from(MINOR_SUBNET_ALPHA),
    );

    // SN2
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &U256::from(OWNER2_HK),
        &U256::from(OWNER2_CK),
        netuid2,
        AlphaCurrency::from(OWNER_ALPHA),
    );
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &U256::from(MAJOR_SN2_HK),
        &U256::from(MAJOR_SN2_CK),
        netuid2,
        AlphaCurrency::from(MAJOR_SUBNET_ALPHA),
    );
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &U256::from(MINOR_SN2_HK),
        &U256::from(MINOR_SN2_CK),
        netuid2,
        AlphaCurrency::from(MINOR_SUBNET_ALPHA),
    );

    // Track SubnetAlphaOut
    let total_subnet_alpha =
        AlphaCurrency::from(OWNER_ALPHA + MAJOR_SUBNET_ALPHA + MINOR_SUBNET_ALPHA);
    SubnetAlphaOut::<Test>::mutate(netuid1, |total| {
        *total = total.saturating_add(total_subnet_alpha);
    });
    SubnetAlphaOut::<Test>::mutate(netuid2, |total| {
        *total = total.saturating_add(total_subnet_alpha);
    });

    // ----------- Root stakes -----------
    SubtensorModule::add_balance_to_coldkey_account(&U256::from(MAJOR_ROOT_CK), MAJOR_ROOT_TAO);
    SubtensorModule::add_balance_to_coldkey_account(&U256::from(MINOR_ROOT_CK), MINOR_ROOT_TAO);
    TotalIssuance::<Test>::mutate(|total| {
        *total = total.saturating_add(TaoCurrency::from(MAJOR_ROOT_TAO + MINOR_ROOT_TAO));
    });
    increase_stake_on_coldkey_hotkey_account(
        &U256::from(MAJOR_ROOT_CK),
        &U256::from(MAJOR_ROOT_HK),
        TaoCurrency::from(MAJOR_ROOT_TAO),
        NetUid::ROOT,
    );
    increase_stake_on_coldkey_hotkey_account(
        &U256::from(MINOR_ROOT_CK),
        &U256::from(MINOR_ROOT_HK),
        TaoCurrency::from(MINOR_ROOT_TAO),
        NetUid::ROOT,
    );

    // ----------- Unstaked TAO (10% of MAJOR_ROOT_TAO) -----------
    // This TAO exists in TotalIssuance but is not staked anywhere.
    // It should have zero effect on utilization.
    TotalIssuance::<Test>::mutate(|total| {
        *total = total.saturating_add(TaoCurrency::from(MAJOR_ROOT_TAO / 10));
    });

    // ----------- Validator permits (manual) -----------
    ValidatorPermit::<Test>::insert(netuid1, vec![true, true, true, true, true, false]);
    ValidatorPermit::<Test>::insert(netuid2, vec![true, true, true, true, true, false]);

    // ----------- Log initial state -----------
    log::info!("=== Initial State ===");
    log::info!("  SN1 SubnetTAO: {:?}", SubnetTAO::<Test>::get(netuid1));
    log::info!(
        "  SN1 SubnetAlphaIn: {:?}",
        SubnetAlphaIn::<Test>::get(netuid1)
    );
    log::info!(
        "  SN1 SubnetAlphaOut: {:?}",
        SubnetAlphaOut::<Test>::get(netuid1)
    );
    log::info!(
        "  SN1 Moving price: {:?}",
        SubnetMovingPrice::<Test>::get(netuid1)
    );
    log::info!(
        "  SN1 EMA flow: {:?}",
        SubnetEmaTaoFlow::<Test>::get(netuid1)
    );
    log::info!(
        "  BlockEmission: {:?}",
        SubtensorModule::get_block_emission()
    );
    log::info!("  TaoWeight: {:?}", TaoWeight::<Test>::get());
    log::info!(
        "  SubnetOwnerCut: {:?}",
        SubtensorModule::get_subnet_owner_cut()
    );

    TestSetup { netuid1, netuid2 }
}

/// Logs detailed per-neuron state for a subnet
fn log_neuron_state(label: &str, netuid: NetUid, neurons: &[(&str, u64, u64)]) {
    log::info!("=== {} (subnet {:?}) ===", label, netuid);
    for &(name, hk_id, _ck_id) in neurons {
        let hotkey = U256::from(hk_id);
        let stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid);
        let alpha_divs = AlphaDividendsPerSubnet::<Test>::get(netuid, hotkey);
        let root_divs = RootAlphaDividendsPerSubnet::<Test>::get(netuid, hotkey);
        let root_stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, NetUid::ROOT);
        log::info!(
            "  {} (hk={}): stake={:?}, alpha_divs={:?}, root_divs={:?}, root_stake={:?}",
            name,
            hk_id,
            stake,
            alpha_divs,
            root_divs,
            root_stake
        );
    }
}

/// Logs subnet-level state including per-UID epoch vectors
fn log_subnet_state(label: &str, netuid: NetUid) {
    log::info!("=== {} (subnet {:?}) ===", label, netuid);
    log::info!("  SubnetTAO: {:?}", SubnetTAO::<Test>::get(netuid));
    log::info!("  SubnetAlphaIn: {:?}", SubnetAlphaIn::<Test>::get(netuid));
    log::info!(
        "  SubnetAlphaOut: {:?}",
        SubnetAlphaOut::<Test>::get(netuid)
    );
    log::info!(
        "  PendingServerEmission: {:?}",
        PendingServerEmission::<Test>::get(netuid)
    );
    log::info!(
        "  PendingValidatorEmission: {:?}",
        PendingValidatorEmission::<Test>::get(netuid)
    );
    log::info!(
        "  PendingRootAlphaDivs: {:?}",
        PendingRootAlphaDivs::<Test>::get(netuid)
    );
    log::info!(
        "  EffectiveRootProp: {:?}",
        EffectiveRootProp::<Test>::get(netuid)
    );
    log::info!("  RootProp: {:?}", RootProp::<Test>::get(netuid));
    let mech_idx = SubtensorModule::get_mechanism_storage_index(netuid, MechId::from(0u8));
    let incentive_vec = Incentive::<Test>::get(mech_idx);
    let dividends_vec = Dividends::<Test>::get(netuid);
    let emission_vec = Emission::<Test>::get(netuid);
    log::info!("  Incentive (per UID): {:?}", incentive_vec);
    log::info!("  Dividends (per UID): {:?}", dividends_vec);
    log::info!("  Emission (per UID): {:?}", emission_vec);
}

/// Standard neuron list for SN1
fn sn1_neurons() -> Vec<(&'static str, u64, u64)> {
    vec![
        ("owner1", OWNER1_HK, OWNER1_CK),
        ("major_root", MAJOR_ROOT_HK, MAJOR_ROOT_CK),
        ("minor_root", MINOR_ROOT_HK, MINOR_ROOT_CK),
        ("major_sn1", MAJOR_SN1_HK, MAJOR_SN1_CK),
        ("minor_sn1", MINOR_SN1_HK, MINOR_SN1_CK),
        ("miner1", MINER1_HK, MINER1_CK),
    ]
}

/// Helper closures for reading stake/dividends
fn stake_of(hk: u64, netuid: NetUid) -> u64 {
    u64::from(SubtensorModule::get_stake_for_hotkey_on_subnet(
        &U256::from(hk),
        netuid,
    ))
}

fn alpha_divs_of(hk: u64, netuid: NetUid) -> u64 {
    u64::from(AlphaDividendsPerSubnet::<Test>::get(netuid, U256::from(hk)))
}

fn root_divs_of(hk: u64, netuid: NetUid) -> u64 {
    u64::from(RootAlphaDividendsPerSubnet::<Test>::get(
        netuid,
        U256::from(hk),
    ))
}

/// 1% tolerance
fn eps(val: u64) -> u64 {
    val / 100
}

// ===========================================================================
// Test 1: Basic case - all validators set weights to miner (price=0.6)
//
// With price=0.6, total_ema_price = 1.2 > 1.0, so root_sell_flag = true
// and root validators earn dividends.
//
// Block structure (5 total):
//   Block 1: setup + set weights
//   Blocks 2-4: coinbase accumulates pending
//   Block 3: 1st epoch + drain (bonds form, dividends still 0 for miners)
//   Block 5: 2nd epoch + drain (bonds active, miners earn incentive)
//
// Run:
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --package pallet-subtensor --lib -- tests::wide_scope_dividend::test_basic_all_validators_set_weights_to_miners --exact --show-output --nocapture
// ===========================================================================
#[test]
fn test_basic_all_validators_set_weights_to_miners() {
    new_test_ext(1).execute_with(|| {
        let setup = setup_test();
        let netuid1 = setup.netuid1;

        // Override prices to 0.6 (root_sell_flag = true: 2*0.6=1.2 > 1.0)
        let tao_reserve = TaoCurrency::from(600_000u64);
        let alpha_reserve = AlphaCurrency::from(1_000_000u64);
        setup_reserves(netuid1, tao_reserve, alpha_reserve);
        setup_reserves(setup.netuid2, tao_reserve, alpha_reserve);
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.6));
        SubnetMovingPrice::<Test>::insert(setup.netuid2, I96F32::from_num(0.6));

        // Get miner UID
        let miner1_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINER1_HK)).unwrap();

        // Set weights: all validators (except owner) -> miner (block 1)
        for hk_id in [MAJOR_ROOT_HK, MINOR_ROOT_HK, MAJOR_SN1_HK, MINOR_SN1_HK] {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(hk_id)),
                netuid1,
                vec![miner1_uid],
                vec![u16::MAX],
                0
            ));
        }
        log::info!(
            "Weights set at block {}",
            SubtensorModule::get_current_block_as_u64()
        );

        // Step 4 blocks: block 1→5. Epochs fire at blocks 3 and 5 for netuid=1, tempo=1.
        let neurons = sn1_neurons();
        for block in 2..=5 {
            step_block(1);
            log::info!(
                "--- Block {} ---",
                SubtensorModule::get_current_block_as_u64()
            );
            log_subnet_state("SN1", netuid1);
            log_neuron_state("SN1 neurons", netuid1, &neurons);
        }

        // ========================================================================
        // SUBNET 1 assertions
        // ========================================================================

        // 1. Miner earned incentive from server emission
        let miner1_stake = stake_of(MINER1_HK, netuid1);
        log::info!("miner1_stake = {}", miner1_stake);
        close(miner1_stake, 1_640_192_260, eps(1_640_192_260));

        // 2. Major subnet validator earned more dividends than minor
        let major_sn1_divs = alpha_divs_of(MAJOR_SN1_HK, netuid1);
        let minor_sn1_divs = alpha_divs_of(MINOR_SN1_HK, netuid1);
        log::info!(
            "major_sn1_divs = {}, minor_sn1_divs = {}",
            major_sn1_divs,
            minor_sn1_divs
        );
        close(major_sn1_divs, 622_577_642, eps(622_577_642));
        close(minor_sn1_divs, 618_529, eps(618_529));
        assert!(major_sn1_divs > minor_sn1_divs);

        // 3. Major subnet validator stake
        close(
            stake_of(MAJOR_SN1_HK, netuid1),
            1_578_898_899,
            eps(1_578_898_899),
        );

        // 4. Root validators earn nonzero (root_sell_flag=true, price=0.6*2=1.2>1.0)
        close(
            stake_of(MAJOR_ROOT_HK, netuid1),
            60_006_436,
            eps(60_006_436),
        );
        close(
            alpha_divs_of(MAJOR_ROOT_HK, netuid1),
            49_088_509,
            eps(49_088_509),
        );
        close(root_divs_of(MAJOR_ROOT_HK, netuid1), 147_661, eps(147_661));
        close(stake_of(MINOR_ROOT_HK, netuid1), 61_228, eps(61_228));
        close(alpha_divs_of(MINOR_ROOT_HK, netuid1), 50_091, eps(50_091));
        close(root_divs_of(MINOR_ROOT_HK, netuid1), 146, eps(146) + 2);
        assert!(stake_of(MAJOR_ROOT_HK, netuid1) > stake_of(MINOR_ROOT_HK, netuid1));

        // 5. Owner earned owner cut (18% of emissions), no dividends
        close(stake_of(OWNER1_HK, netuid1), 719_616_472, eps(719_616_472));
        assert_eq!(alpha_divs_of(OWNER1_HK, netuid1), 0);

        // 6. Miner has 0 dividends
        assert_eq!(alpha_divs_of(MINER1_HK, netuid1), 0);
        assert_eq!(root_divs_of(MINER1_HK, netuid1), 0);

        // 7. Incentive vector: miner (UID 5) has 100% of incentive
        let mech_idx = SubtensorModule::get_mechanism_storage_index(netuid1, MechId::from(0u8));
        let incentive_vec = Incentive::<Test>::get(mech_idx);
        assert_eq!(incentive_vec.get(5).copied().unwrap_or(0), u16::MAX);
        for uid in 0..5 {
            assert_eq!(incentive_vec.get(uid).copied().unwrap_or(0), 0);
        }

        // 8. Root stakes increase due to root dividends being converted to root claimable
        close(
            stake_of(MAJOR_ROOT_HK, NetUid::ROOT),
            5_750_691,
            eps(5_750_691),
        );
        close(
            stake_of(MINOR_ROOT_HK, NetUid::ROOT),
            MINOR_ROOT_TAO,
            eps(MINOR_ROOT_TAO) + 200,
        );

        // 9. EffectiveRootProp is close to RootProp (all root stake is active, utilization ≈ 1.0)
        let erp = EffectiveRootProp::<Test>::get(netuid1);
        let rp = RootProp::<Test>::get(netuid1);
        log::info!(
            "EffectiveRootProp = {:?}, RootProp = {:?}",
            erp,
            rp
        );
        // EffectiveRootProp should be within 2x of RootProp
        assert!(
            erp >= rp,
            "EffectiveRootProp ({erp:?}) should be >= RootProp ({rp:?}) when all root validators set weights"
        );
    });
}

// ===========================================================================
// Test 2: No root sell - all validators set weights (price=0.5)
//
// With price=0.5, total_ema_price = 1.0, root_sell_flag = false.
// Root validators earn 0 dividends.
// ===========================================================================
#[test]
fn test_no_root_sell_all_validators_set_weights_to_miners() {
    new_test_ext(1).execute_with(|| {
        let setup = setup_test();
        let netuid1 = setup.netuid1;

        // Prices stay at 0.5 from setup (root_sell_flag = false: 2*0.5=1.0)

        let miner1_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINER1_HK)).unwrap();

        // Set weights: all validators (except owner) -> miner (block 1)
        for hk_id in [MAJOR_ROOT_HK, MINOR_ROOT_HK, MAJOR_SN1_HK, MINOR_SN1_HK] {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(hk_id)),
                netuid1,
                vec![miner1_uid],
                vec![u16::MAX],
                0
            ));
        }

        let neurons = sn1_neurons();
        for _ in 2..=5 {
            step_block(1);
        }
        log::info!(
            "--- Final state (block {}) ---",
            SubtensorModule::get_current_block_as_u64()
        );
        log_subnet_state("SN1", netuid1);
        log_neuron_state("SN1 neurons", netuid1, &neurons);

        // 1. Miner earned incentive
        close(
            stake_of(MINER1_HK, netuid1),
            1_639_765_956,
            eps(1_639_765_956),
        );

        // 2. Major SN1 validator earned more than minor
        let major_sn1_divs = alpha_divs_of(MAJOR_SN1_HK, netuid1);
        let minor_sn1_divs = alpha_divs_of(MINOR_SN1_HK, netuid1);
        close(major_sn1_divs, 671_619_324, eps(671_619_324));
        close(minor_sn1_divs, 667_252, eps(667_252));
        assert!(major_sn1_divs > minor_sn1_divs);

        // 3. Root validators earn 0 (root_sell_flag=false, total_ema_price=1.0)
        assert_eq!(alpha_divs_of(MAJOR_ROOT_HK, netuid1), 0);
        assert_eq!(root_divs_of(MAJOR_ROOT_HK, netuid1), 0);
        assert_eq!(stake_of(MAJOR_ROOT_HK, netuid1), 0);
        assert_eq!(alpha_divs_of(MINOR_ROOT_HK, netuid1), 0);
        assert_eq!(root_divs_of(MINOR_ROOT_HK, netuid1), 0);
        assert_eq!(stake_of(MINOR_ROOT_HK, netuid1), 0);

        // 4. Owner earned owner cut
        close(stake_of(OWNER1_HK, netuid1), 719_616_472, eps(719_616_472));
        assert_eq!(alpha_divs_of(OWNER1_HK, netuid1), 0);

        // 5. Root stakes unchanged
        assert_eq!(stake_of(MAJOR_ROOT_HK, NetUid::ROOT), MAJOR_ROOT_TAO);
        assert_eq!(stake_of(MINOR_ROOT_HK, NetUid::ROOT), MINOR_ROOT_TAO);
    });
}

// ===========================================================================
// Test 3: Major root validator does NOT set weights on SN1
//
// Price=0.6 (root_sell_flag=true), but major root (5.55M TAO) doesn't
// set weights on SN1. Only minor root, major_sn1, minor_sn1 set weights.
// Expected: major root earns 0 dividends (no bonds), minor root still earns.
// ===========================================================================
#[test]
fn test_basic_major_root_no_weights() {
    new_test_ext(1).execute_with(|| {
        let setup = setup_test();
        let netuid1 = setup.netuid1;

        // Override prices to 0.6
        let tao_reserve = TaoCurrency::from(600_000u64);
        let alpha_reserve = AlphaCurrency::from(1_000_000u64);
        setup_reserves(netuid1, tao_reserve, alpha_reserve);
        setup_reserves(setup.netuid2, tao_reserve, alpha_reserve);
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.6));
        SubnetMovingPrice::<Test>::insert(setup.netuid2, I96F32::from_num(0.6));

        let miner1_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINER1_HK)).unwrap();

        // Set weights: only minor_root, major_sn1, minor_sn1 -> miner
        // MAJOR_ROOT does NOT set weights
        for hk_id in [MINOR_ROOT_HK, MAJOR_SN1_HK, MINOR_SN1_HK] {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(hk_id)),
                netuid1,
                vec![miner1_uid],
                vec![u16::MAX],
                0
            ));
        }

        let neurons = sn1_neurons();
        for _ in 2..=5 {
            step_block(1);
        }
        log::info!(
            "--- Final state (block {}) ---",
            SubtensorModule::get_current_block_as_u64()
        );
        log_subnet_state("SN1", netuid1);
        log_neuron_state("SN1 neurons", netuid1, &neurons);

        // 1. Miner earned incentive
        close(
            stake_of(MINER1_HK, netuid1),
            1_640_192_260,
            eps(1_640_192_260),
        );

        // 2. Major root earns 0 (didn't set weights, no bonds develop)
        assert_eq!(stake_of(MAJOR_ROOT_HK, netuid1), 0);
        assert_eq!(alpha_divs_of(MAJOR_ROOT_HK, netuid1), 0);
        assert_eq!(root_divs_of(MAJOR_ROOT_HK, netuid1), 0);

        // 3. Minor root: hard cap triggered (utilization ≈ 0.001 < 0.5), all root dividends recycled.
        //    Minor root loses its root_alpha_dividends and root-staked portion of alpha_dividends.
        assert_eq!(root_divs_of(MINOR_ROOT_HK, netuid1), 0);
        // Minor root may still have some alpha dividends from its alpha-stake portion
        // (since hard cap only zeroes the root-staked fraction)

        // 4. Subnet validators (alpha-only validators unaffected by hard cap)
        assert!(alpha_divs_of(MAJOR_SN1_HK, netuid1) > 0);
        assert!(alpha_divs_of(MINOR_SN1_HK, netuid1) > 0);

        // 5. Root stakes unchanged (no root dividends converted)
        assert_eq!(stake_of(MAJOR_ROOT_HK, NetUid::ROOT), MAJOR_ROOT_TAO);
        assert_eq!(stake_of(MINOR_ROOT_HK, NetUid::ROOT), MINOR_ROOT_TAO);

        // 6. EffectiveRootProp = 0 (hard cap triggered, utilization < 0.5)
        let erp = EffectiveRootProp::<Test>::get(netuid1);
        log::info!("EffectiveRootProp = {:?}", erp);
        assert_eq!(
            erp,
            U96F32::from_num(0),
            "EffectiveRootProp should be 0 when hard cap triggers (utilization < 0.5)"
        );
    });
}

// ===========================================================================
// Test 4: Unstaked TAO doesn't affect utilization
//
// Same setup as basic test (price=0.6, all validators set weights to miner),
// but with a massive amount of extra unstaked TAO added to TotalIssuance.
// Proves that utilization denominator = root stake on subnet, not TotalIssuance.
//
// Run:
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --package pallet-subtensor --lib -- tests::wide_scope_dividend::test_unstaked_tao_does_not_affect_utilization --exact --show-output --nocapture
// ===========================================================================
#[test]
fn test_unstaked_tao_does_not_affect_utilization() {
    new_test_ext(1).execute_with(|| {
        let setup = setup_test();
        let netuid1 = setup.netuid1;

        // Override prices to 0.6 (root_sell_flag = true)
        let tao_reserve = TaoCurrency::from(600_000u64);
        let alpha_reserve = AlphaCurrency::from(1_000_000u64);
        setup_reserves(netuid1, tao_reserve, alpha_reserve);
        setup_reserves(setup.netuid2, tao_reserve, alpha_reserve);
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.6));
        SubnetMovingPrice::<Test>::insert(setup.netuid2, I96F32::from_num(0.6));

        // Add a MASSIVE amount of unstaked TAO (100x MAJOR_ROOT_TAO)
        TotalIssuance::<Test>::mutate(|total| {
            *total = total.saturating_add(TaoCurrency::from(MAJOR_ROOT_TAO * 100));
        });

        let miner1_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINER1_HK)).unwrap();

        // Set weights: all validators -> miner (same as basic test)
        for hk_id in [MAJOR_ROOT_HK, MINOR_ROOT_HK, MAJOR_SN1_HK, MINOR_SN1_HK] {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(hk_id)),
                netuid1,
                vec![miner1_uid],
                vec![u16::MAX],
                0
            ));
        }

        let neurons = sn1_neurons();
        for _ in 2..=5 {
            step_block(1);
        }
        log::info!(
            "--- Final state (block {}) ---",
            SubtensorModule::get_current_block_as_u64()
        );
        log_subnet_state("SN1", netuid1);
        log_neuron_state("SN1 neurons", netuid1, &neurons);

        // 1. Root validators earn nonzero dividends (utilization = 1.0, no scaling)
        assert!(
            root_divs_of(MAJOR_ROOT_HK, netuid1) > 0,
            "Major root should earn root dividends"
        );
        assert!(
            alpha_divs_of(MAJOR_ROOT_HK, netuid1) > 0,
            "Major root should earn alpha dividends"
        );

        // 2. EffectiveRootProp should be >= RootProp (utilization = 1.0, no scaling)
        let erp = EffectiveRootProp::<Test>::get(netuid1);
        let rp = RootProp::<Test>::get(netuid1);
        log::info!(
            "EffectiveRootProp = {:?}, RootProp = {:?}",
            erp,
            rp
        );
        assert!(
            erp >= rp,
            "EffectiveRootProp ({erp:?}) should be >= RootProp ({rp:?}) with full utilization"
        );

        // 3. Root stakes increase (root dividends converted to root claimable)
        assert!(
            stake_of(MAJOR_ROOT_HK, NetUid::ROOT) > MAJOR_ROOT_TAO,
            "Major root stake should increase from root dividends"
        );

        // 4. Unstaked TAO only affects block emission rate, not utilization
        //    The key invariant: utilization denominator = root stake on subnet, not TotalIssuance
        log::info!(
            "TotalIssuance = {:?}",
            TotalIssuance::<Test>::get()
        );
    });
}

// ===========================================================================
// Test 5: Half-weights test - major root sets half weights to validator
//
// Big root sets half weights to miner, half to minor_root_validator.
// Small root (minor_root) DOES set full weights to miner.
// Utilization stays above 50% so dividends are scaled by utilization, not hard-capped.
//
// Run:
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --package pallet-subtensor --lib -- tests::wide_scope_dividend::test_basic_major_root_half_weights_to_validator --exact --show-output --nocapture
// ===========================================================================
#[test]
fn test_basic_major_root_half_weights_to_validator() {
    new_test_ext(1).execute_with(|| {
        let setup = setup_test();
        let netuid1 = setup.netuid1;

        // Override prices to 0.6
        let tao_reserve = TaoCurrency::from(600_000u64);
        let alpha_reserve = AlphaCurrency::from(1_000_000u64);
        setup_reserves(netuid1, tao_reserve, alpha_reserve);
        setup_reserves(setup.netuid2, tao_reserve, alpha_reserve);
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.6));
        SubnetMovingPrice::<Test>::insert(setup.netuid2, I96F32::from_num(0.6));

        let miner1_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINER1_HK)).unwrap();
        let minor_root_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINOR_ROOT_HK))
                .unwrap();

        // Major root sets HALF weights to miner, HALF to minor_root (validator)
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(MAJOR_ROOT_HK)),
            netuid1,
            vec![miner1_uid, minor_root_uid],
            vec![u16::MAX / 2, u16::MAX / 2],
            0
        ));

        // Minor root sets FULL weights to miner
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(MINOR_ROOT_HK)),
            netuid1,
            vec![miner1_uid],
            vec![u16::MAX],
            0
        ));

        // Subnet validators set weights to miner
        for hk_id in [MAJOR_SN1_HK, MINOR_SN1_HK] {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(hk_id)),
                netuid1,
                vec![miner1_uid],
                vec![u16::MAX],
                0
            ));
        }

        let neurons = sn1_neurons();
        for _ in 2..=5 {
            step_block(1);
        }
        log::info!(
            "--- Final state (block {}) ---",
            SubtensorModule::get_current_block_as_u64()
        );
        log_subnet_state("SN1", netuid1);
        log_neuron_state("SN1 neurons", netuid1, &neurons);

        // 1. EffectiveRootProp should be > 0 (utilization > 0.5, not hard-capped)
        let erp = EffectiveRootProp::<Test>::get(netuid1);
        log::info!("EffectiveRootProp = {:?}", erp);
        assert!(
            erp > U96F32::from_num(0),
            "EffectiveRootProp should be > 0 (utilization > 0.5)"
        );

        // 2. Root validators earn SOME dividends (scaled by utilization, not zero)
        let major_root_divs = root_divs_of(MAJOR_ROOT_HK, netuid1);
        let minor_root_divs = root_divs_of(MINOR_ROOT_HK, netuid1);
        log::info!(
            "major_root_divs = {}, minor_root_divs = {}",
            major_root_divs,
            minor_root_divs
        );
        // At least one root validator should earn some root dividends
        assert!(
            major_root_divs > 0 || minor_root_divs > 0,
            "At least one root validator should earn root dividends (utilization > 0.5)"
        );

        // 3. EffectiveRootProp should be less than the basic test (utilization < 1.0)
        let rp = RootProp::<Test>::get(netuid1);
        log::info!("RootProp = {:?}", rp);
    });
}

// ===========================================================================
// Test 6: Almost-half-weights test - hard cap triggers
//
// Big root sets half weights to miner, half to minor_root_validator.
// Small root does NOT set weights at all.
// Utilization drops below 50%, hard cap triggers.
//
// Run:
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --package pallet-subtensor --lib -- tests::wide_scope_dividend::test_basic_major_root_half_weights_no_minor_root --exact --show-output --nocapture
// ===========================================================================
#[test]
fn test_basic_major_root_half_weights_no_minor_root() {
    new_test_ext(1).execute_with(|| {
        let setup = setup_test();
        let netuid1 = setup.netuid1;

        // Override prices to 0.6
        let tao_reserve = TaoCurrency::from(600_000u64);
        let alpha_reserve = AlphaCurrency::from(1_000_000u64);
        setup_reserves(netuid1, tao_reserve, alpha_reserve);
        setup_reserves(setup.netuid2, tao_reserve, alpha_reserve);
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(0.6));
        SubnetMovingPrice::<Test>::insert(setup.netuid2, I96F32::from_num(0.6));

        let miner1_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINER1_HK)).unwrap();
        let minor_root_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(netuid1, &U256::from(MINOR_ROOT_HK))
                .unwrap();

        // Major root sets HALF weights to miner, HALF to minor_root (validator)
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(MAJOR_ROOT_HK)),
            netuid1,
            vec![miner1_uid, minor_root_uid],
            vec![u16::MAX / 2, u16::MAX / 2],
            0
        ));

        // Minor root does NOT set weights (this is the key difference from test 5)

        // Subnet validators set weights to miner
        for hk_id in [MAJOR_SN1_HK, MINOR_SN1_HK] {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(hk_id)),
                netuid1,
                vec![miner1_uid],
                vec![u16::MAX],
                0
            ));
        }

        let neurons = sn1_neurons();
        for _ in 2..=5 {
            step_block(1);
        }
        log::info!(
            "--- Final state (block {}) ---",
            SubtensorModule::get_current_block_as_u64()
        );
        log_subnet_state("SN1", netuid1);
        log_neuron_state("SN1 neurons", netuid1, &neurons);

        // 1. EffectiveRootProp = 0 (hard cap triggered, utilization < 0.5)
        let erp = EffectiveRootProp::<Test>::get(netuid1);
        log::info!("EffectiveRootProp = {:?}", erp);
        assert_eq!(
            erp,
            U96F32::from_num(0),
            "EffectiveRootProp should be 0 when hard cap triggers (utilization < 0.5)"
        );

        // 2. All root alpha dividends should be 0 (recycled)
        assert_eq!(
            root_divs_of(MAJOR_ROOT_HK, netuid1),
            0,
            "Major root dividends should be 0 (hard cap)"
        );
        assert_eq!(
            root_divs_of(MINOR_ROOT_HK, netuid1),
            0,
            "Minor root dividends should be 0 (hard cap)"
        );

        // 3. Root stakes unchanged (no dividends converted)
        assert_eq!(stake_of(MAJOR_ROOT_HK, NetUid::ROOT), MAJOR_ROOT_TAO);
        assert_eq!(stake_of(MINOR_ROOT_HK, NetUid::ROOT), MINOR_ROOT_TAO);

        // 4. Miner should still earn incentive (not affected by root dividend recycling)
        assert!(
            stake_of(MINER1_HK, netuid1) > 0,
            "Miner should still earn incentive"
        );
    });
}
