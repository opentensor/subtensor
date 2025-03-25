#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use super::mock::*;
use crate::epoch::math::{fixed, u16_proportion_to_fixed};
use crate::*;

use approx::assert_abs_diff_eq;
use frame_support::{assert_err, assert_ok};

// use frame_system::Config;
use rand::{Rng, SeedableRng, distributions::Uniform, rngs::StdRng, seq::SliceRandom, thread_rng};
use sp_core::{Get, U256};
// use sp_runtime::DispatchError;
use std::time::Instant;
use substrate_fixed::types::I32F32;

// Normalizes (sum to 1 except 0) the input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize(x: &mut [I32F32]) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num(0.0_f32) {
        return;
    }
    for i in x.iter_mut() {
        *i /= x_sum;
    }
}

// Inplace normalize the passed positive integer weights so that they sum to u16 max value.
#[allow(dead_code)]
fn normalize_weights(mut weights: Vec<u16>) -> Vec<u16> {
    let sum: u64 = weights.iter().map(|x| *x as u64).sum();
    if sum == 0 {
        return weights;
    }
    weights.iter_mut().for_each(|x| {
        *x = (*x as u64 * u16::MAX as u64 / sum) as u16;
    });
    weights
}

// // Return as usize an I32F32 ratio of a usize input, avoiding the 0% and 100% extremes.
// fn non_extreme_fixed_ratio(ratio: I32F32, total: usize) -> usize {
//     if total == 0 {
//         return total;
//     }
//     let mut subset: usize = (ratio * I32F32::from_num(total)).to_num::<usize>();
//     if subset == 0 {
//         subset = 1;
//     } else if subset == total {
//         subset = total - 1;
//     }
//     return subset;
// }

// // Box-Muller Transform converting two uniform random samples to a normal random sample.
// fn normal(size: usize, rng: &mut StdRng, dist: &Uniform<u16>) -> Vec<I32F32> {
//     let max: I32F32 = I32F32::from_num(u16::MAX);
//     let two: I32F32 = I32F32::from_num(2);
//     let eps: I32F32 = I32F32::from_num(0.000001);
//     let pi: I32F32 = I32F32::from_num(PI);

//     let uniform_u16: Vec<u16> = (0..(2 * size)).map(|_| rng.sample(&dist)).collect();
//     let uniform: Vec<I32F32> = uniform_u16
//         .iter()
//         .map(|&x| I32F32::from_num(x) / max)
//         .collect();
//     let mut normal: Vec<I32F32> = vec![I32F32::from_num(0); size as usize];

//     for i in 0..size {
//         let u1: I32F32 = uniform[i] + eps;
//         let u2: I32F32 = uniform[i + size] + eps;
//         normal[i] = sqrt::<I32F32, I32F32>(-two * ln::<I32F32, I32F32>(u1).expect("")).expect("")
//             * cos(two * pi * u2);
//     }
//     normal
// }

// Returns validators and servers uids with either blockwise, regular, or random interleaving.
fn distribute_nodes(
    validators_n: usize,
    network_n: usize,
    interleave: usize,
) -> (Vec<u16>, Vec<u16>) {
    let mut validators: Vec<u16> = vec![];
    let mut servers: Vec<u16> = vec![];

    if interleave == 0 {
        // blockwise [validator_block, server_block]
        validators = (0..validators_n as u16).collect();
        servers = (validators_n as u16..network_n as u16).collect();
    } else if interleave == 1 {
        // regular interleaving [val, srv, srv, ..., srv, val, srv, srv, ..., srv, val, srv, ..., srv]
        (validators, servers) = (0..network_n as u16)
            .collect::<Vec<u16>>()
            .iter()
            .partition(|&i| *i as usize % (network_n / validators_n) == 0);
    } else if interleave == 2 {
        // random interleaving
        let mut permuted_uids: Vec<u16> = (0..network_n as u16).collect();
        permuted_uids.shuffle(&mut thread_rng());
        validators = permuted_uids[0..validators_n].into();
        servers = permuted_uids[validators_n..network_n].into();
    }

    (validators, servers)
}

#[allow(dead_code)]
fn uid_stats(netuid: u16, uid: u16) {
    log::info!(
        "stake: {:?}",
        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid)))
    );
    log::info!("rank: {:?}", SubtensorModule::get_rank_for_uid(netuid, uid));
    log::info!(
        "trust: {:?}",
        SubtensorModule::get_trust_for_uid(netuid, uid)
    );
    log::info!(
        "consensus: {:?}",
        SubtensorModule::get_consensus_for_uid(netuid, uid)
    );
    log::info!(
        "incentive: {:?}",
        SubtensorModule::get_incentive_for_uid(netuid, uid)
    );
    log::info!(
        "dividend: {:?}",
        SubtensorModule::get_dividends_for_uid(netuid, uid)
    );
    log::info!(
        "emission: {:?}",
        SubtensorModule::get_emission_for_uid(netuid, uid)
    );
}

#[allow(clippy::too_many_arguments)]
fn init_run_epochs(
    netuid: u16,
    n: u16,
    validators: &[u16],
    servers: &[u16],
    epochs: u16,
    stake_per_validator: u64,
    server_self: bool,
    input_stake: &[u64],
    use_input_stake: bool,
    input_weights: &[Vec<(u16, u16)>],
    use_input_weights: bool,
    random_weights: bool,
    random_seed: u64,
    sparse: bool,
    bonds_penalty: u16,
) {
    // === Create the network
    add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead

    // === Set bonds penalty
    SubtensorModule::set_bonds_penalty(netuid, bonds_penalty);

    // === Register uids
    SubtensorModule::set_max_allowed_uids(netuid, n);
    for key in 0..n {
        let stake = if use_input_stake {
            input_stake[key as usize]
        } else if validators.contains(&key) {
            stake_per_validator
        } else {
            // only validators receive stake
            0
        };

        // let stake: u64 = 1; // alternative test: all nodes receive stake, should be same outcome, except stake
        SubtensorModule::add_balance_to_coldkey_account(&(U256::from(key)), stake);
        SubtensorModule::append_neuron(netuid, &(U256::from(key)), 0);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &U256::from(key),
            &U256::from(key),
            netuid,
            stake,
        );
    }
    assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

    // === Issue validator permits
    SubtensorModule::set_max_allowed_validators(netuid, validators.len() as u16);
    assert_eq!(
        SubtensorModule::get_max_allowed_validators(netuid),
        validators.len() as u16
    );
    SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
    run_to_block(1); // run to next block to ensure weights are set on nodes after their registration block

    // === Set weights
    let mut rng = StdRng::seed_from_u64(random_seed); // constant seed so weights over multiple runs are equal
    let range = Uniform::new(0, u16::MAX);
    let mut weights: Vec<u16> = vec![u16::MAX / n; servers.len()];
    for uid in validators {
        if random_weights {
            weights = (0..servers.len()).map(|_| rng.sample(range)).collect();
            weights = normalize_weights(weights);
            // assert_eq!(weights.iter().map(|x| *x as u64).sum::<u64>(), u16::MAX as u64); // normalized weight sum not always u16::MAX
        }
        if use_input_weights {
            let sparse_weights = input_weights[*uid as usize].clone();
            weights = sparse_weights.iter().map(|(_, w)| *w).collect();
            let srvs: Vec<u16> = sparse_weights.iter().map(|(s, _)| *s).collect();
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(*uid as u64)),
                netuid,
                srvs,
                weights.clone(),
                0
            ));
        } else {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(*uid as u64)),
                netuid,
                servers.to_vec(),
                weights.clone(),
                0
            ));
        }
    }
    if server_self {
        for uid in servers {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(*uid as u64)),
                netuid,
                vec![*uid],
                vec![u16::MAX],
                0
            )); // server self-weight
        }
    }

    // === Run the epochs.
    log::info!("Start {epochs} epoch(s)");
    let start = Instant::now();
    for _ in 0..epochs {
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
    }
    let duration = start.elapsed();
    log::info!(
        "Time elapsed in (sparse={sparse}) epoch() is: {:?}",
        duration
    );

    // let bonds = SubtensorModule::get_bonds( netuid );
    // for (uid, node) in vec![ (validators[0], "validator"), (servers[0], "server") ] {
    // 	log::info!("\n{node}" );
    // 	uid_stats(netuid, uid);
    // 	log::info!("bonds: {:?} (on validator), {:?} (on server)", bonds[uid as usize][0], bonds[uid as usize][servers[0] as usize]);
    // }
}

// // Generate a random graph that is split into a major and minor set, each setting specific weight on itself and the complement on the other.
// fn split_graph(
//     major_stake: I32F32,
//     major_weight: I32F32,
//     minor_weight: I32F32,
//     weight_stddev: I32F32,
//     validators_n: usize,
//     network_n: usize,
//     interleave: usize,
// ) -> (
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u16>,
//     Vec<u64>,
//     Vec<Vec<(u16, u16)>>,
//     I32F32,
// ) {
//     let servers_n: usize = network_n - validators_n;
//     let major_servers_n: usize = non_extreme_fixed_ratio(major_stake, servers_n);
//     let major_validators_n: usize = non_extreme_fixed_ratio(major_stake, validators_n);

//     let (validators, servers) = distribute_nodes(validators_n, network_n, interleave as usize);
//     let major_validators: Vec<u16> = (0..major_validators_n).map(|i| validators[i]).collect();
//     let minor_validators: Vec<u16> = (major_validators_n..validators_n)
//         .map(|i| validators[i])
//         .collect();
//     let major_servers: Vec<u16> = (0..major_servers_n).map(|i| servers[i]).collect();
//     let minor_servers: Vec<u16> = (major_servers_n..servers_n).map(|i| servers[i]).collect();

//     let zero: I32F32 = I32F32::from_num(0);
//     let one: I32F32 = I32F32::from_num(1);
//     let stddev: I32F32 = I32F32::from_num(0.3);
//     let total_stake: I64F64 = I64F64::from_num(21_000_000_000_000_000 as u64);
//     let mut rng = StdRng::seed_from_u64(0); // constant seed so weights over multiple runs are equal
//     let dist = Uniform::new(0, u16::MAX);

//     let mut stake: Vec<u64> = vec![0; network_n];
//     let mut stake_fixed: Vec<I32F32> = vec![zero; network_n];
//     for (ratio, vals) in vec![
//         (major_stake, &major_validators),
//         (one - major_stake, &minor_validators),
//     ] {
//         let mut sample = normal(vals.len(), &mut rng, &dist)
//             .iter()
//             .map(|x: &I32F32| {
//                 let v: I32F32 = (stddev * x) + one;
//                 if v < zero {
//                     zero
//                 } else {
//                     v
//                 }
//             })
//             .collect();
//         inplace_normalize(&mut sample);
//         for (i, &val) in vals.iter().enumerate() {
//             stake[val as usize] =
//                 (I64F64::from_num(ratio) * I64F64::from_num(sample[i]) * total_stake)
//                     .to_num::<u64>();
//             stake_fixed[val as usize] =
//                 I32F32::from_num(I64F64::from_num(ratio) * I64F64::from_num(sample[i]));
//         }
//     }

//     let mut weights: Vec<Vec<(u16, u16)>> = vec![vec![]; network_n as usize];
//     let mut weights_fixed: Vec<Vec<I32F32>> = vec![vec![zero; network_n]; network_n];
//     for (first, second, vals) in vec![
//         (major_weight, one - major_weight, &major_validators),
//         (one - minor_weight, minor_weight, &minor_validators),
//     ] {
//         for &val in vals {
//             for (weight, srvs) in vec![(first, &major_servers), (second, &minor_servers)] {
//                 let mut sample: Vec<I32F32> = normal(srvs.len(), &mut rng, &dist)
//                     .iter()
//                     .map(|x: &I32F32| {
//                         let v: I32F32 = (weight_stddev * x) + one;
//                         if v < zero {
//                             zero
//                         } else {
//                             v
//                         }
//                     })
//                     .collect();
//                 inplace_normalize(&mut sample);

//                 for (i, &srv) in srvs.iter().enumerate() {
//                     weights[val as usize].push((srv, fixed_proportion_to_u16(weight * sample[i])));
//                     weights_fixed[val as usize][srv as usize] = weight * sample[i];
//                 }
//             }
//             inplace_normalize(&mut weights_fixed[val as usize]);
//         }
//     }

//     inplace_normalize(&mut stake_fixed);

//     // Calculate stake-weighted mean per server
//     let mut weight_mean: Vec<I32F32> = vec![zero; network_n];
//     for val in 0..network_n {
//         if stake_fixed[val] > zero {
//             for srv in 0..network_n {
//                 weight_mean[srv] += stake_fixed[val] * weights_fixed[val][srv];
//             }
//         }
//     }

//     // Calculate stake-weighted absolute standard deviation
//     let mut weight_dev: Vec<I32F32> = vec![zero; network_n];
//     for val in 0..network_n {
//         if stake_fixed[val] > zero {
//             for srv in 0..network_n {
//                 weight_dev[srv] +=
//                     stake_fixed[val] * (weight_mean[srv] - weights_fixed[val][srv]).abs();
//             }
//         }
//     }

//     // Calculate rank-weighted mean of weight_dev
//     let avg_weight_dev: I32F32 =
//         weight_dev.iter().sum::<I32F32>() / weight_mean.iter().sum::<I32F32>();

//     (
//         validators,
//         servers,
//         major_validators,
//         minor_validators,
//         major_servers,
//         minor_servers,
//         stake,
//         weights,
//         avg_weight_dev,
//     )
// }

// Test consensus guarantees with an epoch on a graph with 4096 nodes, of which the first 128 are validators, the graph is split into a major and minor set, each setting specific weight on itself and the complement on the other. Asserts that the major emission ratio >= major stake ratio.
// #[test]
// fn test_consensus_guarantees() {
//     let netuid: u16 = 0;
//     let network_n: u16 = 512;
//     let validators_n: u16 = 64;
//     let epochs: u16 = 1;
//     let interleave = 2;
//     log::info!("test_consensus_guarantees ({network_n:?}, {validators_n:?} validators)");
//     for (major_stake, major_weight, minor_weight, weight_stddev, bonds_penalty) in vec![
// 		   (0.51, 1., 1., 0.001, u16::MAX),
// 		   (0.51, 0.03, 0., 0.001, u16::MAX),
// 		   (0.51, 0.51, 0.49, 0.001, u16::MAX),
// 		   (0.51, 0.51, 1., 0.001, u16::MAX),
// 		   (0.51, 0.61, 0.8, 0.1, u16::MAX),
// 		   (0.6, 0.67, 0.65, 0.2, u16::MAX),
// 		   (0.6, 0.74, 0.77, 0.4, u16::MAX),
// 		   (0.6, 0.76, 0.8, 0.4, u16::MAX),
// 		   (0.6, 0.73, 1., 0.4, u16::MAX), // bonds_penalty = 100%
// 		   (0.6, 0.74, 1., 0.4, 55800), // bonds_penalty = 85%
// 		   (0.6, 0.76, 1., 0.4, 43690), // bonds_penalty = 66%
// 		   (0.6, 0.78, 1., 0.4, 21845), // bonds_penalty = 33%
// 		   (0.6, 0.79, 1., 0.4, 0), // bonds_penalty = 0%
// 		   (0.6, 0.92, 1., 0.4, u16::MAX),
// 		   (0.6, 0.94, 1., 0.4, u16::MAX),
// 		   (0.65, 0.78, 0.85, 0.6, u16::MAX),
// 		   (0.7, 0.81, 0.85, 0.8, u16::MAX),
// 		   (0.7, 0.83, 0.85, 1., u16::MAX),
//     ] {
//         let (
//             validators,
//             servers,
//             major_validators,
//             minor_validators,
//             major_servers,
//             minor_servers,
//             stake,
//             weights,
//             _avg_weight_dev,
//         ) = split_graph(
//             fixed(major_stake),
//             fixed(major_weight),
//             fixed(minor_weight),
//             fixed(weight_stddev),
//             validators_n as usize,
//             network_n as usize,
//             interleave as usize,
//         );

//         new_test_ext(1).execute_with(|| {
//             init_run_epochs(
//                 netuid,
//                 network_n,
//                 &validators,
//                 &servers,
//                 epochs,
//                 1,
//                 true,
//                 &stake,
//                 true,
//                 &weights,
//                 true,
//                 false,
//                 0,
//                 false,
//                 bonds_penalty
//             );

//             let mut major_emission: I64F64 = I64F64::from_num(0);
//             let mut minor_emission: I64F64 = I64F64::from_num(0);
//             for set in vec![major_validators, major_servers] {
//                 for uid in set {
//                     major_emission +=
//                         I64F64::from_num(SubtensorModule::get_emission_for_uid(netuid, uid));
//                 }
//             }
//             for set in vec![minor_validators, minor_servers] {
//                 for uid in set {
//                     minor_emission +=
//                         I64F64::from_num(SubtensorModule::get_emission_for_uid(netuid, uid));
//                 }
//             }
//             let major_ratio: I32F32 =
//                 I32F32::from_num(major_emission / (major_emission + minor_emission));
//             assert!(major_stake <= major_ratio);
//         });
//     }
// }

// Test an epoch on an empty graph.
// #[test]
// fn test_overflow() {
//     new_test_ext(1).execute_with(|| {
//         log::info!("test_overflow:");
//         let netuid: u16 = 1;
//         add_network(netuid, 1,  0);
//         SubtensorModule::set_max_allowed_uids(netuid, 3);
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(
//             &U256::from(0),
//             &U256::from(0),
//             10,
//         );
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(
//             &U256::from(1),
//             &U256::from(1),
//             10,
//         );
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(
//             &U256::from(2),
//             &U256::from(2),
//             10,
//         );
//         SubtensorModule::append_neuron(netuid, &U256::from(0), 0);
//         SubtensorModule::append_neuron(netuid, &U256::from(1), 0);
//         SubtensorModule::append_neuron(netuid, &U256::from(2), 0);
//         SubtensorModule::set_validator_permit_for_uid(0, 0, true);
//         SubtensorModule::set_validator_permit_for_uid(0, 1, true);
//         SubtensorModule::set_validator_permit_for_uid(0, 2, true);
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(U256::from(0)),
//             netuid,
//             vec![0, 1, 2],
//             vec![u16::MAX / 3, u16::MAX / 3, u16::MAX],
//             0
//         ));
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(U256::from(1)),
//             netuid,
//             vec![1, 2],
//             vec![u16::MAX / 2, u16::MAX / 2],
//             0
//         ));
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(U256::from(2)),
//             netuid,
//             vec![2],
//             vec![u16::MAX],
//             0
//         ));
//         SubtensorModule::epoch(0, u64::MAX);
//     });
// }

// Test an epoch on an empty graph.
// #[test]
// fn test_nill_epoch_subtensor() {
//     new_test_ext(1).execute_with(|| {
//         log::info!("test_nill_epoch:");
//         SubtensorModule::epoch(0, 0);
//     });
// }

// Test an epoch on a graph with a single item.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::epoch::test_1_graph --exact --show-output --nocapture
#[test]
fn test_1_graph() {
    new_test_ext(1).execute_with(|| {
        log::info!("test_1_graph:");
        let netuid: u16 = 1;
        let coldkey = U256::from(0);
        let hotkey = U256::from(0);
        let uid: u16 = 0;
        let stake_amount: u64 = 1;
        add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead
        SubtensorModule::set_max_allowed_uids(netuid, 1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake_amount);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_amount,
        );
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);
        run_to_block(1); // run to next block to ensure weights are set on nodes after their registration block
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        // SubtensorModule::set_weights_for_testing( netuid, i as u16, vec![ ( 0, u16::MAX )]); // doesn't set update status
        // SubtensorModule::set_bonds_for_testing( netuid, uid, vec![ ( 0, u16::MAX )]); // rather, bonds are calculated in epoch
        SubtensorModule::epoch(netuid, 1_000_000_000);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            stake_amount
        );
        assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 0);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 0);
    });
}

// Test an epoch on a graph with two items.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::epoch::test_10_graph --exact --show-output --nocapture
#[test]
fn test_10_graph() {
    new_test_ext(1).execute_with(|| {
        log::info!("test_10_graph");
        // Function for adding a nodes to the graph.
        pub fn add_node(netuid: u16, coldkey: U256, hotkey: U256, uid: u16, stake_amount: u64) {
            log::info!(
                "+Add net:{:?} coldkey:{:?} hotkey:{:?} uid:{:?} stake_amount: {:?} subn: {:?}",
                netuid,
                coldkey,
                hotkey,
                uid,
                stake_amount,
                SubtensorModule::get_subnetwork_n(netuid),
            );
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                netuid,
                stake_amount,
            );
            SubtensorModule::append_neuron(netuid, &hotkey, 0);
            assert_eq!(SubtensorModule::get_subnetwork_n(netuid) - 1, uid);
        }
        // Build the graph with 10 items
        // each with 1 stake and self weights.
        let n: usize = 10;
        let netuid: u16 = 1;
        add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead
        SubtensorModule::set_max_allowed_uids(netuid, n as u16);
        for i in 0..10 {
            add_node(netuid, U256::from(i), U256::from(i), i as u16, 1)
        }
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 10);
        run_to_block(1); // run to next block to ensure weights are set on nodes after their registration block
        for i in 0..10 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(i)),
                netuid,
                vec![i as u16],
                vec![u16::MAX],
                0
            ));
        }
        // Run the epoch.
        SubtensorModule::epoch(netuid, 1_000_000_000);
        // Check return values.
        for i in 0..n {
            assert_eq!(
                SubtensorModule::get_total_stake_for_hotkey(&(U256::from(i))),
                1
            );
            assert_eq!(SubtensorModule::get_rank_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_trust_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, i as u16), 0);
            assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, i as u16), 0);
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, i as u16),
                99999999
            );
        }
    });
}

// Test an epoch on a graph with 512 nodes, of which the first 64 are validators setting non-self weights, and the rest servers setting only self-weights.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::epoch::test_512_graph --exact --show-output --nocapture
#[test]
fn test_512_graph() {
    let netuid: u16 = 1;
    let network_n: u16 = 512;
    let validators_n: u16 = 64;
    let max_stake_per_validator: u64 = 328_125_000_000_000; // 21_000_000_000_000_000 / 64
    let epochs: u16 = 3;
    log::info!("test_{network_n:?}_graph ({validators_n:?} validators)");
    for interleave in 0..3 {
        for server_self in [false, true] {
            // server-self weight off/on
            let (validators, servers) = distribute_nodes(
                validators_n as usize,
                network_n as usize,
                interleave as usize,
            );
            let server: usize = servers[0] as usize;
            let validator: usize = validators[0] as usize;
            new_test_ext(1).execute_with(|| {
                init_run_epochs(
                    netuid,
                    network_n,
                    &validators,
                    &servers,
                    epochs,
                    max_stake_per_validator,
                    server_self,
                    &[],
                    false,
                    &[],
                    false,
                    false,
                    0,
                    false,
                    u16::MAX,
                );
                let bonds = SubtensorModule::get_bonds(netuid);
                for uid in validators {
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
                        max_stake_per_validator
                    );
                    assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 1023); // Note D = floor(1 / 64 * 65_535) = 1023
                    assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 7812500); // Note E = 0.5 / 200 * 1_000_000_000 = 7_812_500
                    assert_eq!(bonds[uid as usize][validator], 0.0);
                    assert_eq!(bonds[uid as usize][server], I32F32::from_num(276));
                }
                for uid in servers {
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
                        0
                    );
                    assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 146); // Note R = floor(1 / (512 - 64) * 65_535) = 146
                    assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 65535);
                    assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 146); // Note C = floor(1 / (512 - 64) * 65_535) = 146
                    assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 146); // Note I = floor(1 / (512 - 64) * 65_535) = 146
                    assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 0);
                    assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 1116071); // Note E = floor(0.5 / (512 - 64) * 1_000_000_000) = 1_116_071
                    assert_eq!(bonds[uid as usize][validator], 0.0);
                    assert_eq!(bonds[uid as usize][server], 0.0);
                }
            });
        }
    }
}

// Test an epoch on a graph with 4096 nodes, of which the first 256 are validators setting random non-self weights, and the rest servers setting only self-weights.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::epoch::test_512_graph_random_weights --exact --show-output --nocapture
#[test]
fn test_512_graph_random_weights() {
    let netuid: u16 = 1;
    let network_n: u16 = 512;
    let validators_n: u16 = 64;
    let epochs: u16 = 1;
    log::info!("test_{network_n:?}_graph_random_weights ({validators_n:?} validators)");
    for interleave in 0..3 {
        // server-self weight off/on
        for server_self in [false, true] {
            for bonds_penalty in [0, u16::MAX / 2, u16::MAX] {
                let (validators, servers) = distribute_nodes(
                    validators_n as usize,
                    network_n as usize,
                    interleave as usize,
                );
                let server: usize = servers[0] as usize;
                let validator: usize = validators[0] as usize;
                let (mut rank, mut incentive, mut dividend, mut emission, mut bondv, mut bonds): (
                    Vec<u16>,
                    Vec<u16>,
                    Vec<u16>,
                    Vec<u64>,
                    Vec<I32F32>,
                    Vec<I32F32>,
                ) = (vec![], vec![], vec![], vec![], vec![], vec![]);

                // Dense epoch
                new_test_ext(1).execute_with(|| {
                    init_run_epochs(
                        netuid,
                        network_n,
                        &validators,
                        &servers,
                        epochs,
                        1,
                        server_self,
                        &[],
                        false,
                        &[],
                        false,
                        true,
                        interleave as u64,
                        false,
                        bonds_penalty,
                    );

                    let bond = SubtensorModule::get_bonds(netuid);
                    for uid in 0..network_n {
                        rank.push(SubtensorModule::get_rank_for_uid(netuid, uid));
                        incentive.push(SubtensorModule::get_incentive_for_uid(netuid, uid));
                        dividend.push(SubtensorModule::get_dividends_for_uid(netuid, uid));
                        emission.push(SubtensorModule::get_emission_for_uid(netuid, uid));
                        bondv.push(bond[uid as usize][validator]);
                        bonds.push(bond[uid as usize][server]);
                    }
                });

                // Sparse epoch (same random seed as dense)
                new_test_ext(1).execute_with(|| {
                    init_run_epochs(
                        netuid,
                        network_n,
                        &validators,
                        &servers,
                        epochs,
                        1,
                        server_self,
                        &[],
                        false,
                        &[],
                        false,
                        true,
                        interleave as u64,
                        true,
                        bonds_penalty,
                    );
                    // Assert that dense and sparse epoch results are equal
                    let bond = SubtensorModule::get_bonds(netuid);
                    for uid in 0..network_n {
                        assert_eq!(
                            SubtensorModule::get_rank_for_uid(netuid, uid),
                            rank[uid as usize]
                        );
                        assert_eq!(
                            SubtensorModule::get_incentive_for_uid(netuid, uid),
                            incentive[uid as usize]
                        );
                        assert_eq!(
                            SubtensorModule::get_dividends_for_uid(netuid, uid),
                            dividend[uid as usize]
                        );
                        assert_eq!(
                            SubtensorModule::get_emission_for_uid(netuid, uid),
                            emission[uid as usize]
                        );
                        assert_eq!(bond[uid as usize][validator], bondv[uid as usize]);
                        assert_eq!(bond[uid as usize][server], bonds[uid as usize]);
                    }
                });
            }
        }
    }
}

// Test an epoch on a graph with 4096 nodes, of which the first 256 are validators setting non-self weights, and the rest servers setting only self-weights.
// #[test]
// fn test_4096_graph() {
//     let netuid: u16 = 1;
//     let network_n: u16 = 4096;
//     let validators_n: u16 = 256;
//     let epochs: u16 = 1;
//     let max_stake_per_validator: u64 = 82_031_250_000_000; // 21_000_000_000_000_000 / 256
//     log::info!("test_{network_n:?}_graph ({validators_n:?} validators)");
//     for interleave in 0..3 {
//         let (validators, servers) = distribute_nodes(
//             validators_n as usize,
//             network_n as usize,
//             interleave as usize,
//         );
//         let server: usize = servers[0] as usize;
//         let validator: usize = validators[0] as usize;
//         for server_self in [false, true] {
//             // server-self weight off/on
//             new_test_ext(1).execute_with(|| {
//                 init_run_epochs(
//                     netuid,
//                     network_n,
//                     &validators,
//                     &servers,
//                     epochs,
//                     max_stake_per_validator,
//                     server_self,
//                     &[],
//                     false,
//                     &[],
//                     false,
//                     false,
//                     0,
//                     true,
//                     u16::MAX,
//                 );
//                 let (total_stake, _, _) = SubtensorModule::get_stake_weights_for_network(netuid);
//                 assert_eq!(total_stake.iter().map(|s| s.to_num::<u64>()).sum::<u64>(), 21_000_000_000_000_000);
//                 let bonds = SubtensorModule::get_bonds(netuid);
//                 for uid in &validators {
//                     assert_eq!(
//                         SubtensorModule::get_total_stake_for_hotkey(&(U256::from(*uid as u64))),
//                         max_stake_per_validator
//                     );
//                     assert_eq!(SubtensorModule::get_rank_for_uid(netuid, *uid), 0);
//                     assert_eq!(SubtensorModule::get_trust_for_uid(netuid, *uid), 0);
//                     assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, *uid), 0);
//                     assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, *uid), 0);
//                     assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, *uid), 255); // Note D = floor(1 / 256 * 65_535)
//                     assert_eq!(SubtensorModule::get_emission_for_uid(netuid, *uid), 1953125); // Note E = 0.5 / 256 * 1_000_000_000 = 1953125
//                     assert_eq!(bonds[*uid as usize][validator], 0.0);
//                     assert_eq!(
//                         bonds[*uid as usize][server],
//                         I32F32::from_num(255) / I32F32::from_num(65_535)
//                     ); // Note B_ij = floor(1 / 256 * 65_535) / 65_535
//                 }
//                 for uid in &servers {
//                     assert_eq!(
//                         SubtensorModule::get_total_stake_for_hotkey(&(U256::from(*uid as u64))),
//                         0
//                     );
//                     assert_eq!(SubtensorModule::get_rank_for_uid(netuid, *uid), 17); // Note R = floor(1 / (4096 - 256) * 65_535) = 17
//                     assert_eq!(SubtensorModule::get_trust_for_uid(netuid, *uid), 65535);
//                     assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, *uid), 17); // Note C = floor(1 / (4096 - 256) * 65_535) = 17
//                     assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, *uid), 17); // Note I = floor(1 / (4096 - 256) * 65_535) = 17
//                     assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, *uid), 0);
//                     assert_eq!(SubtensorModule::get_emission_for_uid(netuid, *uid), 130208); // Note E = floor(0.5 / (4096 - 256) * 1_000_000_000) = 130208
//                     assert_eq!(bonds[*uid as usize][validator], 0.0);
//                     assert_eq!(bonds[*uid as usize][server], 0.0);
//                 }
//             });
//         }
//     }
// }

// Test an epoch_sparse on a graph with 16384 nodes, of which the first 512 are validators setting non-self weights, and the rest servers setting only self-weights.
// #[test]
// fn test_16384_graph_sparse() {
//     new_test_ext(1).execute_with(|| {
//         let netuid: u16 = 1;
//         let n: u16 = 16384;
//         let validators_n: u16 = 512;
//         let validators: Vec<u16> = (0..validators_n).collect();
//         let servers: Vec<u16> = (validators_n..n).collect();
//         let server: u16 = servers[0];
//         let epochs: u16 = 1;
//         log::info!("test_{n:?}_graph ({validators_n:?} validators)");
//         init_run_epochs(
//             netuid,
//             n,
//             &validators,
//             &servers,
//             epochs,
//             1,
//             false,
//             &[],
//             false,
//             &[],
//             false,
//             false,
//             0,
//             true,
//             u16::MAX,
//         );
//         let bonds = SubtensorModule::get_bonds(netuid);
//         for uid in validators {
//             assert_eq!(
//                 SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
//                 1
//             );
//             assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 0);
//             assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 0);
//             assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 438); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
//             assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 0);
//             assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 127); // Note D = floor(1 / 512 * 65_535) = 127
//             assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 976085); // Note E = 0.5 / 512 * 1_000_000_000 = 976_562 (discrepancy)
//             assert_eq!(bonds[uid as usize][0], 0.0);
//             assert_eq!(
//                 bonds[uid as usize][server as usize],
//                 I32F32::from_num(127) / I32F32::from_num(65_535)
//             ); // Note B_ij = floor(1 / 512 * 65_535) / 65_535 = 127 / 65_535
//         }
//         for uid in servers {
//             assert_eq!(
//                 SubtensorModule::get_total_stake_for_hotkey(&(U256::from(uid))),
//                 0
//             );
//             assert_eq!(SubtensorModule::get_rank_for_uid(netuid, uid), 4); // Note R = floor(1 / (16384 - 512) * 65_535) = 4
//             assert_eq!(SubtensorModule::get_trust_for_uid(netuid, uid), 65535);
//             assert_eq!(SubtensorModule::get_consensus_for_uid(netuid, uid), 4); // Note C = floor(1 / (16384 - 512) * 65_535) = 4
//             assert_eq!(SubtensorModule::get_incentive_for_uid(netuid, uid), 4); // Note I = floor(1 / (16384 - 512) * 65_535) = 4
//             assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 0);
//             assert_eq!(SubtensorModule::get_emission_for_uid(netuid, uid), 31517); // Note E = floor(0.5 / (16384 - 512) * 1_000_000_000) = 31502 (discrepancy)
//             assert_eq!(bonds[uid as usize][0], 0.0);
//             assert_eq!(bonds[uid as usize][server as usize], 0.0);
//         }
//     });
// }

// Test bonds exponential moving average over a sequence of epochs.
#[test]
fn test_bonds() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 8;
        let netuid: u16 = 1;
        let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
        let max_stake: u64 = 4;
        let stakes: Vec<u64> = vec![1, 2, 3, 4, 0, 0, 0, 0];
        let block_number = System::block_number();
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_min_allowed_weights(netuid, 1);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);
        SubtensorModule::set_bonds_penalty(netuid, u16::MAX);

        // === Register [validator1, validator2, validator3, validator4, server1, server2, server3, server4]
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), max_stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                <<Test as frame_system::Config>::RuntimeOrigin>::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(key),
                &U256::from(key),
                netuid,
                stakes[key as usize],
            );
        }
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

        // === Issue validator permits
        SubtensorModule::set_max_allowed_validators(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_validators(netuid), n);
        SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
        next_block(); // run to next block to ensure weights are set on nodes after their registration block

        // === Set weights [val->srv1: 0.1, val->srv2: 0.2, val->srv3: 0.3, val->srv4: 0.4]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / 4, u16::MAX / 2, (u16::MAX / 4) * 3, u16::MAX],
                0
            ));
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  n: 8
            current_block: 2
            activity_cutoff: 5000
            Last update: [2, 2, 2, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.099997558, 0.2000012202, 0.2999926745, 0.4000085443]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Clipped Weights: [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0.9999999995, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.099997558, 0.2000012202, 0.2999926745, 0.4000085443]
            Trust: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926752, 0.4000085455]
            Bonds: [[], [], [], [], [], [], [], []]
            Bonds: (mask+norm) [[], [], [], [], [], [], [], []]
            weights_for_bonds: [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            ΔB:
            , 0.1999999997), (7, 0.1999999997)], [(4, 0.299999999), (5, 0.2999999998), (6, 0.3), (7, 0.3)], [(4, 0.4000000013), (5, 0.4), (6, 0.4000000004), (7, 0.4000000001)], [], [], [], []]

            emaB: [[(4, 0.0099999998), (5, 0.0099999998), (6, 0.0099999998), (7, 0.0099999998)], [(4, 0.0199999998), (5, 0.0199999998), (6, 0.0199999998), (7, 0.0199999998)], [(4, 0.0299999998), (5, 0.0299999998), (6, 0.03), (7, 0.03)], [(4, 0.04), (5, 0.0399999998), (6, 0.04), (7, 0.04)], [], [], [], []]
            emaB norm: [[(4, 0.0999999982), (5, 0.0999999985), (6, 0.099999998), (7, 0.099999998)], [(4, 0.199999999), (5, 0.1999999995), (6, 0.1999999986), (7, 0.1999999986)], [(4, 0.2999999996), (5, 0.3000000003), (6, 0.3000000012), (7, 0.3000000012)], [(4, 0.4000000027), (5, 0.4000000013), (6, 0.4000000018), (7, 0.4000000018)], [], [], [], []]
            total_bonds_per_validator: [0.0999999975, 0.1999999979, 0.3000000003, 0.4000000008, 0, 0, 0, 0]
            Dividends: [0.0333333318, 0.1333333314, 0.3000000005, 0.5333333358, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0.016666666, 0.0666666657, 0.1500000001, 0.266666668, 0, 0, 0, 0]
            Validator Emission: [16666665, 66666665, 150000000, 266666668, 0, 0, 0, 0]
            Normalized Combined Emission: [0.016666666, 0.0666666657, 0.1500000001, 0.266666668, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
            Combined Emission: [16666665, 66666665, 150000000, 266666668, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0.016666666, 0.0666666657, 0.1500000001, 0.266666668, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
        */

        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][4], 655);
        assert_eq!(bonds[1][4], 1310);
        assert_eq!(bonds[2][4], 1966);
        assert_eq!(bonds[3][4], 2621);

        // === Set self-weight only on val1
        let uid = 0;
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  n: 8
            current_block: 3
            activity_cutoff: 5000
            Last update: [2, 2, 2, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0
            .400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Clipped Weights: [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
            Trust: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            Bonds: [[(4, 655), (5, 655), (6, 655), (7, 655)], [(4, 1310), (5, 1310), (6, 1310), (7, 1310)], [(4, 1966), (5, 1966), (6, 1966), (7, 1966)], [(4, 2621), (5, 2621), (6, 2621), (7, 2621)], [], [], [], []]
            Bonds: (mask+norm) [[(4, 0.0099946593), (5, 0.0099946593), (6, 0.0099946593), (7, 0.0099946593)], [(4, 0.0199893187), (5, 0.0199893187), (6, 0.0199893187), (7, 0.0199893187)], [(4, 0.029999237), (5, 0.029999237), (6, 0.029999237), (7, 0.029999237)], [(4, 0.0399938964), (5, 0.0399938964), (6, 0.0399938964), (7, 0.0399938964)], [], [], [], []]
            weights_for_bonds: [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            emaB: [[(4, 0.0089951933), (5, 0.0089951933), (6, 0.0089951933), (7, 0.0089951933)], [(4, 0.0402126086), (5, 0.0402126086), (6, 0.0402126086), (7, 0.0402126086)], [(4, 0.0603326464), (5, 0.0603326464), (6, 0.0603326464), (7, 0.0603326464)], [(4, 0.0804389513), (5, 0.080438951), (6, 0.080438951), (7, 0.080438951)], [], [], [], []]
            emaB norm: [[(4, 0.0473482562), (5, 0.0473482562), (6, 0.0473482562), (7, 0.0473482562)], [(4, 0.2116682583), (5, 0.2116682585), (6, 0.2116682585), (7, 0.2116682585)], [(4, 0.3175746764), (5, 0.3175746768), (6, 0.3175746768), (7, 0.3175746768)], [(4, 0.4234088087), (5, 0.423408808), (6, 0.423408808), (7, 0.423408808)], [], [], [], []]
            total_bonds_per_validator: [0.0473482558, 0.2116682578, 0.3175746764, 0.4234088075, 0, 0, 0, 0]
            Dividends: [0.0151901136, 0.1358134532, 0.3056498466, 0.5433465862, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0.0075950567, 0.0679067266, 0.1528249232, 0.271673293, 0, 0, 0, 0]
            Validator Emission: [7595056, 67906726, 152824923, 271673293, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0075950567, 0.0679067266, 0.1528249232, 0.271673293, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Combined Emission: [7595056, 67906726, 152824923, 271673293, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0.0075950567, 0.0679067266, 0.1528249232, 0.271673293, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */
        assert_eq!(bonds[0][4], 655);
        assert_eq!(bonds[1][4], 1310);
        assert_eq!(bonds[2][4], 1966);
        assert_eq!(bonds[3][4], 2621);

        // === Set self-weight only on val2
        let uid = 1;
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  n: 8
            current_block: 4
            activity_cutoff: 5000
            Last update: [2, 3, 2, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Clipped Weights: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            Trust: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            Bonds: [[(4, 589), (5, 589), (6, 589), (7, 589)], [(4, 2635), (5, 2635), (6, 2635), (7, 2635)], [(4, 3953), (5, 3953), (6, 3953), (7, 3953)], [(4, 5271), (5, 5271), (6, 5271), (7, 5271)], [], [], [], []]
            Bonds: (mask+norm) [[(4, 0.008987564), (5, 0.008987564), (6, 0.008987564), (7, 0.008987564)], [(4, 0.0402075227), (5, 0.0402075227), (6, 0.0402075227), (7, 0.0402075227)], [(4, 0.0603189135), (5, 0.0603189135), (6, 0.0603189135), (7, 0.0603189135)], [(4, 0.0804303044), (5, 0.0804303044), (6, 0.0804303044), (7, 0.0804303044)], [], [], [], []]
            weights_for_bonds: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            emaB: [[(4, 0.0080888073), (5, 0.0080888073), (6, 0.0080888073), (7, 0.0080888073)], [(4, 0.0361867703), (5, 0.0361867703), (6, 0.0361867703), (7, 0.0361867703)], [(4, 0.0971441646), (5, 0.0971441648), (6, 0.0971441648), (7, 0.0971441648)], [(4, 0.1295301311), (5, 0.129530131), (6, 0.129530131), (7, 0.129530131)], [], [], [], []]
            emaB norm: [[(4, 0.0298535195), (5, 0.0298535195), (6, 0.0298535195), (7, 0.0298535195)], [(4, 0.1335552211), (5, 0.1335552211), (6, 0.1335552211), (7, 0.1335552211)], [(4, 0.3585318692), (5, 0.3585318702), (6, 0.3585318702), (7, 0.3585318702)], [(4, 0.4780593896), (5, 0.4780593887), (6, 0.4780593887), (7, 0.4780593887)], [], [], [], []]
            total_bonds_per_validator: [0.0298535188, 0.1335552207, 0.3585318697, 0.4780593882, 0, 0, 0, 0]
            Dividends: [0.0090883898, 0.08131718, 0.3274465878, 0.5821478418, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0.0045441948, 0.04065859, 0.163723294, 0.291073921, 0, 0, 0, 0]
            Validator Emission: [4544194, 40658589, 163723293, 291073920, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0045441948, 0.04065859, 0.163723294, 0.291073921, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Combined Emission: [4544194, 40658589, 163723293, 291073920, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0.0045441948, 0.04065859, 0.163723294, 0.291073921, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][4], 530);
        assert_eq!(bonds[1][4], 2371);
        assert_eq!(bonds[2][4], 6366);
        assert_eq!(bonds[3][4], 8488);

        // === Set self-weight only on val2
        let uid = 1;
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  n: 8
            current_block: 5
            activity_cutoff: 5000
            Last update: [2, 4, 2, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Clipped Weights: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            Trust: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            Bonds: [[(4, 530), (5, 530), (6, 530), (7, 530)], [(4, 2371), (5, 2371), (6, 2371), (7, 2371)], [(4, 6366), (5, 6366), (6, 6366), (7, 6366)], [(4, 8488), (5, 8488), (6, 8488), (7, 8488)], [], [], [], []]
            Bonds: (mask+norm) [[(4, 0.0080872816), (5, 0.0080872816), (6, 0.0080872816), (7, 0.0080872816)], [(4, 0.036179141), (5, 0.036179141), (6, 0.036179141), (7, 0.036179141)], [(4, 0.0971389334), (5, 0.0971389334), (6, 0.0971389334), (7, 0.0971389334)], [(4, 0.1295185778), (5, 0.1295185778), (6, 0.1295185778), (7, 0.1295185778)], [], [], [], []]
            weights_for_bonds: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            emaB: [[(4, 0.0072785532), (5, 0.0072785532), (6, 0.0072785532), (7, 0.0072785532)], [(4, 0.0325612267), (5, 0.0325612267), (6, 0.0325612267), (7, 0.0325612267)], [(4, 0.1302821825), (5, 0.1302821827), (6, 0.1302821827), (7, 0.1302821827)], [(4, 0.1737095772), (5, 0.173709577), (6, 0.173709577), (7, 0.173709577)], [], [], [], []]
            emaB norm: [[(4, 0.0211689514), (5, 0.0211689514), (6, 0.0211689514), (7, 0.0211689514)], [(4, 0.094701105), (5, 0.094701105), (6, 0.094701105), (7, 0.094701105)], [(4, 0.3789128321), (5, 0.3789128328), (6, 0.3789128328), (7, 0.3789128328)], [(4, 0.505217111), (5, 0.5052171105), (6, 0.5052171105), (7, 0.5052171105)], [], [], [], []]
            total_bonds_per_validator: [0.021168951, 0.0947011046, 0.3789128324, 0.50521711, 0, 0, 0, 0]
            Dividends: [0.0062849855, 0.0562328366, 0.3374935838, 0.599988594, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0.0031424926, 0.0281164183, 0.1687467918, 0.2999942969, 0, 0, 0, 0]
            Validator Emission: [3142492, 28116418, 168746791, 299994296, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0031424926, 0.0281164183, 0.1687467918, 0.2999942969, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Combined Emission: [3142492, 28116418, 168746791, 299994296, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0.0031424926, 0.0281164183, 0.1687467918, 0.2999942969, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 476);
        assert_eq!(bonds[1][7], 2133);
        assert_eq!(bonds[2][7], 8538);
        assert_eq!(bonds[3][7], 11384);

        // === Set val3->srv4: 1
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(2)),
            netuid,
            vec![7],
            vec![u16::MAX],
            0
        ));
        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  n: 8
            current_block: 6
            activity_cutoff: 5000
            Last update: [2, 4, 5, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [], [(7, 1)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0399990233, 0.080000488, 0.11999707, 0.4600034177]
            Consensus: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
            Clipped Weights: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
            Trust: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
            Incentive (=Rank): [0, 0, 0, 0, 0, 0, 0, 1]
            Bonds: [[(4, 476), (5, 476), (6, 476), (7, 476)], [(4, 2133), (5, 2133), (6, 2133), (7, 2133)], [(4, 8538), (5, 8538), (6, 8538), (7, 8538)], [(4, 11384), (5, 11384), (6, 11384), (7, 11384)], [], [], [], []]
            Bonds: (mask+norm) [[(4, 0.0072632944), (5, 0.0072632944), (6, 0.0072632944), (7, 0.0072632944)], [(4, 0.0325474937), (5, 0.0325474937), (6, 0.0325474937), (7, 0.0325474937)], [(4, 0.130281529), (5, 0.130281529), (6, 0.130281529), (7, 0.130281529)], [(4, 0.1737087052), (5, 0.1737087052), (6, 0.1737087052), (7, 0.1737087052)], [], [], [], []]
            weights_for_bonds: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            emaB: [[(4, 0.0065369648), (5, 0.0065369648), (6, 0.0065369648), (7, 0.0065369648)], [(4, 0.0292927441), (5, 0.0292927441), (6, 0.0292927441), (7, 0.0292927441)], [(4, 0.117253376), (5, 0.117253376), (6, 0.117253376), (7, 0.1601105188)], [(4, 0.1563378347), (5, 0.1563378347), (6, 0.1563378347), (7, 0.2134806917)], [], [], [], []]
            emaB norm: [[(4, 0.0211264472), (5, 0.0211264472), (6, 0.0211264472), (7, 0.0159663672)], [(4, 0.0946695658), (5, 0.0946695658), (6, 0.0946695658), (7, 0.0715467692)], [(4, 0.3789445655), (5, 0.3789445655), (6, 0.3789445655), (7, 0.3910657985)], [(4, 0.505259421), (5, 0.505259421), (6, 0.505259421), (7, 0.5214210646)], [], [], [], []]
            total_bonds_per_validator: [0.0159663672, 0.0715467692, 0.3910657985, 0.5214210646, 0, 0, 0, 0]
            Dividends: [0.0046713396, 0.0418654135, 0.3432467687, 0.610216478, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0, 0, 0, 0.5]
            Server Emission: [0, 0, 0, 0, 0, 0, 0, 500000000]
            Normalized Validator Emission: [0.0023356697, 0.0209327068, 0.1716233843, 0.305108239, 0, 0, 0, 0]
            Validator Emission: [2335669, 20932706, 171623384, 305108238, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0023356697, 0.0209327068, 0.1716233843, 0.305108239, 0, 0, 0, 0.5]
            Combined Emission: [2335669, 20932706, 171623384, 305108238, 0, 0, 0, 500000000]
            Pruning Scores: [0.0023356697, 0.0209327068, 0.1716233843, 0.305108239, 0, 0, 0, 0.5]

        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 428);
        assert_eq!(bonds[1][7], 1919);
        assert_eq!(bonds[2][7], 10492);
        assert_eq!(bonds[3][7], 13990);

        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  n: 8
            current_block: 7
            activity_cutoff: 5000
            Last update: [2, 4, 5, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [], [(7, 1)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0399990233, 0.080000488, 0.11999707, 0.4600034177]
            Consensus: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
            Clipped Weights: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
            Trust: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
            Incentive (=Rank): [0, 0, 0, 0, 0, 0, 0, 1]
            Bonds: [[(4, 428), (5, 428), (6, 428), (7, 428)], [(4, 1919), (5, 1919), (6, 1919), (7, 1919)], [(4, 7684), (5, 7684), (6, 7684), (7, 10492)], [(4, 10245), (5, 10245), (6, 10245), (7, 13990)], [], [], [], []]
            Bonds: (mask+norm) [[(4, 0.0065308614), (5, 0.0065308614), (6, 0.0065308614), (7, 0.0065308614)], [(4, 0.029282063), (5, 0.029282063), (6, 0.029282063), (7, 0.029282063)], [(4, 0.1172503242), (5, 0.1172503242), (6, 0.1172503242), (7, 0.1600976577)], [(4, 0.1563286793), (5, 0.1563286793), (6, 0.1563286793), (7, 0.2134737163)], [], [], [], []]
            weights_for_bonds: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            emaB: [[(4, 0.0058777751), (5, 0.0058777751), (6, 0.0058777751), (7, 0.0058777751)], [(4, 0.0263538565), (5, 0.0263538565), (6, 0.0263538565), (7, 0.0263538565)], [(4, 0.1055252918), (5, 0.1055252918), (6, 0.1055252918), (7, 0.1869450347)], [(4, 0.1406958112), (5, 0.1406958112), (6, 0.1406958112), (7, 0.2492692014)], [], [], [], []]
            emaB norm: [[(4, 0.0211086995), (5, 0.0211086995), (6, 0.0211086995), (7, 0.0125473945)], [(4, 0.0946439134), (5, 0.0946439134), (6, 0.0946439134), (7, 0.0562580617)], [(4, 0.3789702114), (5, 0.3789702114), (6, 0.3789702114), (7, 0.3990749998)], [(4, 0.5052771752), (5, 0.5052771752), (6, 0.5052771752), (7, 0.5321195435)], [], [], [], []]
            total_bonds_per_validator: [0.0125473945, 0.0562580617, 0.3990749998, 0.5321195435, 0, 0, 0, 0]
            Dividends: [0.003636117, 0.0326061223, 0.3469446378, 0.6168131223, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0, 0, 0, 0.5]
            Server Emission: [0, 0, 0, 0, 0, 0, 0, 500000000]
            Normalized Validator Emission: [0.0018180585, 0.016303061, 0.1734723188, 0.3084065611, 0, 0, 0, 0]
            Validator Emission: [1818058, 16303061, 173472318, 308406561, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0018180585, 0.016303061, 0.1734723188, 0.3084065611, 0, 0, 0, 0.5]
            Combined Emission: [1818058, 16303061, 173472318, 308406561, 0, 0, 0, 500000000]
            Pruning Scores: [0.0018180585, 0.016303061, 0.1734723188, 0.3084065611, 0, 0, 0, 0.5]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 385);
        assert_eq!(bonds[1][7], 1727);
        assert_eq!(bonds[2][7], 12251);
        assert_eq!(bonds[3][7], 16335);

        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  n: 8
            current_block: 8
            activity_cutoff: 5000
            Last update: [2, 4, 5, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [], [(7, 1)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0399990233, 0.080000488, 0.11999707, 0.4600034177]
            Consensus: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
            Clipped Weights: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
            Trust: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
            Incentive (=Rank): [0, 0, 0, 0, 0, 0, 0, 1]
            Bonds: [[(4, 385), (5, 385), (6, 385), (7, 385)], [(4, 1727), (5, 1727), (6, 1727), (7, 1727)], [(4, 6915), (5, 6915), (6, 6915), (7, 12251)], [(4, 9220), (5, 9220), (6, 9220), (7, 16335)], [], [], [], []]
            Bonds: (mask+norm) [[(4, 0.0058747234), (5, 0.0058747234), (6, 0.0058747234), (7, 0.0058747234)], [(4, 0.0263523308), (5, 0.0263523308), (6, 0.0263523308), (7, 0.0263523308)], [(4, 0.1055161364), (5, 0.1055161364), (6, 0.1055161364), (7, 0.1869382772)], [(4, 0.1406881819), (5, 0.1406881819), (6, 0.1406881819), (7, 0.2492561226)], [], [], [], []]
            weights_for_bonds: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            emaB: [[(4, 0.005287251), (5, 0.005287251), (6, 0.005287251), (7, 0.005287251)], [(4, 0.0237170977), (5, 0.0237170977), (6, 0.0237170977), (7, 0.0237170977)], [(4, 0.0949645226), (5, 0.0949645226), (6, 0.0949645226), (7, 0.2111015923)], [(4, 0.1266193634), (5, 0.1266193634), (6, 0.1266193634), (7, 0.2814733672)], [], [], [], []]
            emaB norm: [[(4, 0.0210993583), (5, 0.0210993583), (6, 0.0210993583), (7, 0.010137003)], [(4, 0.094645695), (5, 0.094645695), (6, 0.094645695), (7, 0.0454716997)], [(4, 0.3789664055), (5, 0.3789664055), (6, 0.3789664055), (7, 0.404735366)], [(4, 0.5052885406), (5, 0.5052885406), (6, 0.5052885406), (7, 0.539655931)], [], [], [], []]
            total_bonds_per_validator: [0.010137003, 0.0454716997, 0.404735366, 0.539655931, 0, 0, 0, 0]
            Dividends: [0.0029180378, 0.0261789719, 0.3495214381, 0.621381552, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0, 0, 0, 0.5]
            Server Emission: [0, 0, 0, 0, 0, 0, 0, 500000000]
            Normalized Validator Emission: [0.0014590188, 0.013089486, 0.174760719, 0.310690776, 0, 0, 0, 0]
            Validator Emission: [1459018, 13089485, 174760719, 310690775, 0, 0, 0, 0]
            Normalized Combined Emission: [0.0014590188, 0.013089486, 0.174760719, 0.310690776, 0, 0, 0, 0.5]
            Combined Emission: [1459018, 13089485, 174760719, 310690775, 0, 0, 0, 500000000]
            Pruning Scores: [0.0014590188, 0.013089486, 0.174760719, 0.310690776, 0, 0, 0, 0.5]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 346);
        assert_eq!(bonds[1][7], 1554);
        assert_eq!(bonds[2][7], 13834);
        assert_eq!(bonds[3][7], 18446);

        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::epoch::test_512_graph_random_weights --exact --show-output --nocapture
#[test]
fn test_bonds_with_liquid_alpha() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 8;
        let netuid: u16 = 1;
        let tempo: u16 = 1;
        let max_stake: u64 = 4;
        let stakes: Vec<u64> = vec![1, 2, 3, 4, 0, 0, 0, 0];
        let block_number = System::block_number();
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_min_allowed_weights(netuid, 1);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        // Register validators and servers
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), max_stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                RuntimeOrigin::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(key),
                &U256::from(key),
                netuid,
                stakes[key as usize],
            );
        }

        // Initilize with first epoch
        SubtensorModule::epoch(netuid, 1_000_000_000);
        next_block_no_epoch(netuid);

        // Set weights
        for uid in 0..(n / 2) {
            SubtensorModule::set_validator_permit_for_uid(netuid, uid, true);
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / 4, u16::MAX / 2, (u16::MAX / 4) * 3, u16::MAX],
                0
            ));
        }

        // Enable Liquid Alpha
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        // Run epoch with Liquid Alpha
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }

        // Check bonds and emissions
        let bonds = SubtensorModule::get_bonds(netuid);

        /*  n: 8
            current_block: 3
            activity_cutoff: 5000
            Last update: [2, 2, 2, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 64
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Clipped Weights: [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
            Trust: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            Bonds: [[(4, 4596), (5, 9192), (6, 13788), (7, 18385)], [(4, 4596), (5, 9192), (6, 13788), (7, 18385)], [(4, 4596), (5, 9192), (6, 13788), (7, 18385)], [(4, 4596), (5, 9192), (6, 13788), (7, 18385)], [], [], [], []]
            Bonds: (mask+norm) [[(4, 0.0701304646), (5, 0.1402609292), (6, 0.2103913939), (7, 0.2805371175)], [(4, 0.0701304646), (5, 0.1402609292), (6, 0.2103913939), (7, 0.2805371175)], [(4, 0.0701304646), (5, 0.1402609292), (6, 0.2103913939), (7, 0.2805371175)], [(4, 0.0701304646), (5, 0.1402609292), (6, 0.2103913939), (7, 0.2805371175)], [], [], [], []]
            weights_for_bonds: [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            alphas: [[0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7026884612, 0.7053405554, 0.7104771058, 0.7200544013], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993]]
            emaB: [[], [(4, 0.0910776372), (5, 0.1821595554), (6, 0.273232912), (7, 0.364327949)], [(4, 0.0910776372), (5, 0.1821595554), (6, 0.273232912), (7, 0.364327949)], [(4, 0.0910776372), (5, 0.1821595554), (6, 0.273232912), (7, 0.364327949)], [], [], [], []]
            emaB norm: [[], [(4, 0.3333333333), (5, 0.3333333333), (6, 0.3333333333), (7, 0.3333333333)], [(4, 0.3333333333), (5, 0.3333333333), (6, 0.3333333333), (7, 0.3333333333)], [(4, 0.3333333333), (5, 0.3333333333), (6, 0.3333333333), (7, 0.3333333333)], [], [], [], []]
            total_bonds_per_validator: [0, 0.3333333328, 0.3333333328, 0.3333333328, 0, 0, 0, 0]
            Dividends: [0, 0.222222222, 0.333333333, 0.4444444447, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0, 0.111111111, 0.1666666665, 0.2222222222, 0, 0, 0, 0]
            Validator Emission: [0, 111111111, 166666666, 222222222, 0, 0, 0, 0]
            Normalized Combined Emission: [0, 0.111111111, 0.1666666665, 0.2222222222, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Combined Emission: [0, 111111111, 166666666, 222222222, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0, 0.111111111, 0.1666666665, 0.2222222222, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]

        */

        assert_eq!(bonds[0][4], 4596); // Note: Calculated as explained above
        assert_eq!(bonds[1][4], 4596); // Note: Calculated as explained above
        assert_eq!(bonds[2][4], 4596); // Note: Calculated as explained above
        assert_eq!(bonds[3][4], 4596); // Note: Calculated as explained above

        // === Set self-weight only on val1
        let uid = 0;
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        next_block_no_epoch(netuid);
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }

        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][4], 0);
        assert_eq!(bonds[1][4], 5968);
        assert_eq!(bonds[2][4], 5968);
        assert_eq!(bonds[3][4], 5968);

        // === Set self-weight only on val2
        let uid = 1;
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid)),
            netuid,
            vec![uid],
            vec![u16::MAX],
            0
        ));
        next_block_no_epoch(netuid);
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        let bonds = SubtensorModule::get_bonds(netuid);

        /*  n: 8
            current_block: 4
            activity_cutoff: 5000
            Last update: [2, 3, 2, 2, 1, 1, 1, 1]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 64
            new_validator_permits: [true, true, true, true, true, true, true, true]
            Active Stake: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            Weights: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            Weights (mask+norm): [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Ranks (before): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            Consensus: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            Clipped Weights: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Validator Trust: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            Ranks (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            Trust: [0, 0, 0, 0, 1, 1, 1, 1]
            Incentive (=Rank): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            Bonds: [[], [(4, 5968), (5, 11937), (6, 17906), (7, 23876)], [(4, 5968), (5, 11937), (6, 17906), (7, 23876)], [(4, 5968), (5, 11937), (6, 17906), (7, 23876)], [], [], [], []]
            Bonds: (mask+norm) [[], [(4, 0.0910658427), (5, 0.1821469443), (6, 0.273228046), (7, 0.3643244067)], [(4, 0.0910658427), (5, 0.1821469443), (6, 0.273228046), (7, 0.3643244067)], [(4, 0.0910658427), (5, 0.1821469443), (6, 0.273228046), (7, 0.3643244067)], [], [], [], []]
            weights_for_bonds: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            alphas: [[0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7033024912, 0.7080039687, 0.718774016, 0.74096124], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993], [0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993, 0.7013461993]]
            emaB: [[], [], [(4, 0.0973300673), (5, 0.1946689729), (6, 0.291999317), (7, 0.3893513412)], [(4, 0.0973300673), (5, 0.1946689729), (6, 0.291999317), (7, 0.3893513412)], [], [], [], []]
            emaB norm: [[], [], [(4, 0.5), (5, 0.5), (6, 0.5), (7, 0.5)], [(4, 0.5), (5, 0.5), (6, 0.5), (7, 0.5)], [], [], [], []]
            total_bonds_per_validator: [0, 0, 0.4999999998, 0.4999999998, 0, 0, 0, 0]
            Dividends: [0, 0, 0.4285714282, 0.5714285716, 0, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0, 0, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Server Emission: [0, 0, 0, 0, 49998779, 100000610, 149996337, 200004272]
            Normalized Validator Emission: [0, 0, 0.214285714, 0.2857142857, 0, 0, 0, 0]
            Validator Emission: [0, 0, 214285714, 285714285, 0, 0, 0, 0]
            Normalized Combined Emission: [0, 0, 0.214285714, 0.2857142857, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            Combined Emission: [0, 0, 214285714, 285714285, 49998779, 100000610, 149996337, 200004272]
            Pruning Scores: [0, 0, 0.214285714, 0.2857142857, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]/
        */

        assert_eq!(bonds[0][4], 0);
        assert_eq!(bonds[1][4], 0);
        assert_eq!(bonds[2][4], 6378);
        assert_eq!(bonds[3][4], 6378);
    });
}

//
#[test]
fn test_set_alpha_disabled() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(1 + 456);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let signer = RuntimeOrigin::signed(coldkey);

        // Enable Liquid Alpha and setup
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey,));
        assert_ok!(SubtensorModule::add_stake(
            signer.clone(),
            hotkey,
            netuid,
            DefaultMinStake::<Test>::get() + DefaultStakingFee::<Test>::get()
        ));
        // Only owner can set alpha values
        assert_ok!(SubtensorModule::register_network(signer.clone(), hotkey));

        // Explicitly set to false
        SubtensorModule::set_liquid_alpha_enabled(netuid, false);
        assert_err!(
            SubtensorModule::do_set_alpha_values(signer.clone(), netuid, 12_u16, u16::MAX),
            Error::<Test>::LiquidAlphaDisabled
        );

        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            12_u16,
            u16::MAX
        ));
    });
}

// Test that epoch masks out inactive stake of validators with outdated weights beyond activity cutoff.
#[test]
fn test_active_stake() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        let sparse: bool = true;
        let n: u16 = 4;
        let netuid: u16 = 1;
        let tempo: u16 = 1;
        let block_number: u64 = System::block_number();
        let stake: u64 = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        // === Register [validator1, validator2, server1, server2]
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                RuntimeOrigin::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(key),
                &U256::from(key),
                netuid,
                stake,
            );
        }
        assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

        // === Issue validator permits
        SubtensorModule::set_max_allowed_validators(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_validators(netuid), n);
        SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
        next_block_no_epoch(netuid); // run to next block to ensure weights are set on nodes after their registration block

        // === Set weights [val1->srv1: 0.5, val1->srv2: 0.5, val2->srv1: 0.5, val2->srv2: 0.5]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / (n / 2); (n / 2) as usize],
                0
            ));
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        let bonds = SubtensorModule::get_bonds(netuid);
        for uid in 0..n {
            // log::info!("\n{uid}" );
            // uid_stats(netuid, uid);
            // log::info!("bonds: {:?}", bonds[uid as usize]);
            if uid < n / 2 {
                assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, uid), 32767);
                // Note D = floor(0.5 * 65_535)
            }
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, uid),
                250000000
            ); // Note E = 0.5 / (n/2) * 1_000_000_000 = 250_000_000
        }
        for bond in bonds.iter().take((n / 2) as usize) {
            // for on_validator in 0..(n / 2) as usize {
            for i in bond.iter().take((n / 2) as usize) {
                assert_eq!(*i, 0);
            }
            for i in bond.iter().take(n as usize).skip((n / 2) as usize) {
                assert_eq!(*i, I32F32::from_num(3276)); // floor(0.5*(2^16-1))/(2^16-1), then max-upscale to 65_535
            }
        }
        let activity_cutoff: u64 = SubtensorModule::get_activity_cutoff(netuid) as u64;
        run_to_block_no_epoch(netuid, activity_cutoff + 2); // run to block where validator (uid 0, 1) weights become outdated

        // === Update uid 0 weights
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(0)),
            netuid,
            ((n / 2)..n).collect(),
            vec![u16::MAX / (n / 2); (n / 2) as usize],
            0
        ));
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*
            current_block: 5002
            activity_cutoff: 5000
            Last update: [5002, 1, 0, 0]
            Block at registration: [0, 0, 0, 0]
            validator_permits: [true, true, true, true]
            max_allowed_validators: 4
            new_validator_permits: [true, true, true, true]
            Active Stake: [1, 0, 0, 0]
            Weights: [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (permit): [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (permit+diag): [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (permit+diag+outdate): [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
            Ranks (before): [0, 0, 0.5, 0.5]
            Consensus: [0, 0, 0.5, 0.5]
            Clipped Weights: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
            Validator Trust: [1, 1, 0, 0]
            Ranks (after): [0, 0, 0.5, 0.5]
            Trust: [0, 0, 1, 1]
            Incentive (=Rank): [0, 0, 0.5, 0.5]
            Bonds: [[(2, 3276), (3, 3276)], [(2, 3276), (3, 3276)], [], []]
            Bonds: (mask+norm) [[(2, 0.0499885557), (3, 0.0499885557)], [(2, 0.0499885557), (3, 0.0499885557)], [], []]
            weights_for_bonds: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
            emaB: [[(2, 0.1449897), (3, 0.1449897)], [(2, 0.0449897), (3, 0.0449897)], [], []]
            emaB norm: [[(2, 0.7631864299), (3, 0.7631864299)], [(2, 0.23681357), (3, 0.23681357)], [], []]
            total_bonds_per_validator: [0.7631864296, 0.23681357, 0, 0]
            Dividends: [1, 0, 0, 0]
            Normalized Server Emission: [0, 0, 0.25, 0.25]
            Server Emission: [0, 0, 250000000, 250000000]
            Normalized Validator Emission: [0.5, 0, 0, 0]
            Validator Emission: [500000000, 0, 0, 0]
            Normalized Combined Emission: [0.5, 0, 0.25, 0.25]
            Combined Emission: [500000000, 0, 250000000, 250000000]
            Pruning Scores: [0.5, 0, 0.25, 0.25]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 65535);
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 500000000);
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[0][server], I32F32::from_num(9501));
        }
        for validator in 1..(n / 2) {
            assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, validator), 0);
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, validator), 0);
            for server in ((n / 2) as usize)..n as usize {
                assert_eq!(bonds[validator as usize][server], I32F32::from_num(2948));
            }
        }

        // === Update uid 1 weights as well
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(1)),
            netuid,
            ((n / 2)..n).collect(),
            vec![u16::MAX / (n / 2); (n / 2) as usize],
            0
        ));
        run_to_block_no_epoch(netuid, activity_cutoff + 3); // run to block where validator (uid 0, 1) weights become outdated
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*
            current_block: 5003
            activity_cutoff: 5000
            Last update: [5002, 5002, 0, 0]
            Block at registration: [0, 0, 0, 0]
            validator_permits: [true, true, true, true]
            max_allowed_validators: 4
            new_validator_permits: [true, true, true, true]
            Active Stake: [0.5, 0.5, 0, 0]
            Weights: [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (permit): [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (permit+diag): [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (permit+diag+outdate): [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
            Weights (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
            Ranks (before): [0, 0, 0.5, 0.5]
            Consensus: [0, 0, 0.5, 0.5]
            Clipped Weights: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
            Validator Trust: [1, 1, 0, 0]
            Ranks (after): [0, 0, 0.5, 0.5]
            Trust: [0, 0, 1, 1]
            Incentive (=Rank): [0, 0, 0.5, 0.5]
            Bonds: [[(2, 9501), (3, 9501)], [(2, 2948), (3, 2948)], [], []]
            Bonds: (mask+norm) [[(2, 0.144975967), (3, 0.144975967)], [(2, 0.0449835965), (3, 0.0449835965)], [], []]
            weights_for_bonds: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
            emaB: [[(2, 0.1804783703), (3, 0.1804783703)], [(2, 0.0904852368), (3, 0.0904852368)], [], []]
            emaB norm: [[(2, 0.666061292), (3, 0.666061292)], [(2, 0.3339387078), (3, 0.3339387078)], [], []]
            total_bonds_per_validator: [0.666061292, 0.3339387076, 0, 0]
            Dividends: [0.6660612922, 0.3339387076, 0, 0]
            Normalized Server Emission: [0, 0, 0.25, 0.25]
            Server Emission: [0, 0, 250000000, 250000000]
            Normalized Validator Emission: [0.333030646, 0.1669693538, 0, 0]
            Validator Emission: [333030645, 166969353, 0, 0]
            Normalized Combined Emission: [0.333030646, 0.1669693538, 0.25, 0.25]
            Combined Emission: [333030645, 166969353, 250000000, 250000000]
            Pruning Scores: [0.333030646, 0.1669693538, 0.25, 0.25]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 43650);
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 333030645);
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[0][server], I32F32::from_num(11827));
        }
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 1), 21884);
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 1), 166969353);
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[1][server], I32F32::from_num(5929));
        }
    });
}

// Test that epoch masks out outdated weights and bonds of validators on deregistered servers.
#[test]
fn test_outdated_weights() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 4;
        let netuid: u16 = 1;
        let tempo: u16 = 0;
        let mut block_number: u64 = System::block_number();
        let stake: u64 = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);
        SubtensorModule::set_bonds_penalty(netuid, u16::MAX);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);

        // === Register [validator1, validator2, server1, server2]
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                RuntimeOrigin::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(key),
                &U256::from(key),
                netuid,
                stake,
            );
        }
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 4);

        // === Issue validator permits
        SubtensorModule::set_max_allowed_validators(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_validators(netuid), n);
        SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 4);
        block_number = next_block_no_epoch(netuid); // run to next block to ensure weights are set on nodes after their registration block
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);

        // === Set weights [val1->srv1: 2/3, val1->srv2: 1/3, val2->srv1: 2/3, val2->srv2: 1/3, srv1->srv1: 1, srv2->srv2: 1]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![2 * (u16::MAX / 3), u16::MAX / 3],
                0
            ));
        }
        for uid in ((n / 2) as u64)..n as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                vec![uid as u16],
                vec![u16::MAX],
                0
            )); // server self-weight
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*
            Number of Neurons in Network: 4
            current_block: 2
            activity_cutoff: 5000
            Last update: [2, 2, 2, 2]
            Block at registration: [1, 1, 1, 1]
            validator_permits: [true, true, true, true]
            max_allowed_validators: 4
            new_validator_permits: [true, true, true, true]
            Active Stake: [0.25, 0.25, 0.25, 0.25]
            Weights: [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
            Weights (permit): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
            Weights (permit+diag): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
            Weights (permit+diag+outdate): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
            Weights (mask+norm): [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 0.6666632756), (3, 0.3333367242)], [], []]
            Ranks (before): [0, 0, 0.3333316376, 0.166668362]
            Consensus: [0, 0, 0.6666632756, 0.3333367242]
            Clipped Weights: [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 0.6666632756), (3, 0.3333367242)], [], []]
            Validator Trust: [0.9999999998, 0.9999999998, 0, 0]
            Ranks (after): [0, 0, 0.3333316376, 0.166668362]
            Trust: [0, 0, 1, 1]
            Incentive (=Rank): [0, 0, 0.6666632756, 0.3333367242]
            Bonds: [[], [], [], []]
            Bonds: (mask+norm) [[], [], [], []]
            weights_for_bonds: [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 0.6666632756), (3, 0.3333367242)], [], []]
            emaB: [[(2, 0.05), (3, 0.05)], [(2, 0.05), (3, 0.05)], [], []]
            emaB norm: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
            total_bonds_per_validator: [0.4999999998, 0.4999999998, 0, 0]
            Dividends: [0.5, 0.5, 0, 0]
            Normalized Server Emission: [0, 0, 0.3333316378, 0.166668362]
            Server Emission: [0, 0, 333331637, 166668361]
            Normalized Validator Emission: [0.25, 0.25, 0, 0]
            Validator Emission: [250000000, 250000000, 0, 0]
            Normalized Combined Emission: [0.25, 0.25, 0.3333316378, 0.166668362]
            Combined Emission: [250000000, 250000000, 333331637, 166668361]
            Pruning Scores: [0.25, 0.25, 0.3333316378, 0.166668362]
        */

        // === Dereg server2 at uid3 (least emission) + register new key over uid3
        let new_key: u64 = n as u64; // register a new key while at max capacity, which means the least incentive uid will be deregistered
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &U256::from(new_key),
        );
        assert_eq!(System::block_number(), block_number);
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid), n);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);
        assert_ok!(SubtensorModule::register(
            RuntimeOrigin::signed(U256::from(new_key)),
            netuid,
            block_number,
            nonce,
            work,
            U256::from(new_key),
            U256::from(new_key)
        ));
        let deregistered_uid: u16 = n - 1; // since uid=n-1 only recieved 1/3 of weight, it will get pruned first
        assert_eq!(
            U256::from(new_key),
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, deregistered_uid)
                .expect("Not registered")
        );
        next_block_no_epoch(netuid); // run to next block to outdate weights and bonds set on deregistered uid

        // === Update weights from only uid=0
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(0)),
            netuid,
            ((n / 2)..n).collect(),
            vec![2 * (u16::MAX / 3), u16::MAX / 3],
            0
        ));
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*
            Number of Neurons in Network: 4
            current_block: 3
            activity_cutoff: 5000
            Last update: [3, 2, 2, 2]
            Block at registration: [1, 1, 1, 2]
            validator_permits: [true, true, true, true]
            max_allowed_validators: 4
            new_validator_permits: [true, true, true, true]
            Active Stake: [0.3333333333, 0.3333333333, 0.3333333333, 0]
            Weights: [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
            Weights (permit): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
            Weights (permit+diag): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
            Weights (permit+diag+outdate): [[(2, 65535), (3, 32768)], [(2, 65535)], [], []]
            Weights (mask+norm): [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 1)], [], []]
            Ranks (before): [0, 0, 0.5555544249, 0.1111122412]
            Consensus: [0, 0, 0.6666632756, 0]
            Clipped Weights: [[(2, 0.6666632756)], [(2, 0.6666632756)], [], []]
            Validator Trust: [0.6666632756, 0.6666632756, 0, 0]
            Ranks (after): [0, 0, 0.4444421832, 0]
            Trust: [0, 0, 0.799997558, 0]
            Incentive (=Rank): [0, 0, 1, 0]
            Bonds: [[(2, 3276), (3, 3276)], [(2, 3276), (3, 3276)], [], []]
            Bonds: (mask+norm) [[(2, 0.0499885557), (3, 0.0499885557)], [(2, 0.0499885557)], [], []]
            weights_for_bonds: [[(2, 0.6666632756)], [(2, 0.6666632756)], [], []]
            emaB: [[(2, 0.0949897), (3, 0.0449897)], [(2, 0.0949897)], [], []]
            emaB norm: [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
            total_bonds_per_validator: [0.5, 0.5, 0, 0]
            Dividends: [0.5, 0.5, 0, 0]
            Normalized Server Emission: [0, 0, 0.5, 0]
            Server Emission: [0, 0, 500000000, 0]
            Normalized Validator Emission: [0.25, 0.25, 0, 0]
            Validator Emission: [250000000, 250000000, 0, 0]
            Normalized Combined Emission: [0.25, 0.25, 0.5, 0]
            Combined Emission: [250000000, 250000000, 500000000, 0]
            Pruning Scores: [0.25, 0.25, 0.5, 0]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 32767); // Note D = floor(0.5 * 65_535)
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 250000000); // Note E = 0.5 * 0.5 * 1_000_000_000 = 249311245
        assert_eq!(bonds[0][2], I32F32::from_num(6225));
        assert_eq!(bonds[0][3], I32F32::from_num(2948));
    });
}

// Test the zero emission handling and fallback under zero effective weight conditions, to ensure non-zero effective emission.
#[test]
fn test_zero_weights() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 2;
        let netuid: u16 = 1;
        let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
        let mut block_number: u64 = 0;
        let stake: u64 = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);

        // === Register [validator, server]
        for key in 0..n as u64 {
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                RuntimeOrigin::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
        }
        for validator in 0..(n / 2) as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(validator), stake);
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(validator),
                &U256::from(validator),
                netuid,
                stake,
            );
        }
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

        // === No weights
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 0; activity_cutoff: 5000; Last update: [0, 0]; Inactive: [false, false]
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 0]
        W: [[], []]; W (diagmask): [[], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []]
        R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0]
        B: [[], []]; B (mask+norm): [[], []];
        ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0]
        E: [1000000000, 0]; P: [1, 0] */
        for validator in 0..(n / 2) {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000
            ); // Note E = 1 * 1_000_000_000
        }
        for server in (n / 2)..n {
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, server), 0);
            // no stake
        }
        run_to_block(1);
        block_number += 1; // run to next block to ensure weights are set on nodes after their registration block

        // === Self-weights only: set weights [srv->srv: 1]
        for uid in ((n / 2) as u64)..n as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                vec![uid as u16],
                vec![u16::MAX],
                0
            )); // server self-weight
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 1; activity_cutoff: 5000; Last update: [0, 1]; Inactive: [false, false]
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 0]
        W: [[], [(1, 1)]]
        W (diagmask): [[], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []]
        R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0]
        B: [[], []]: B (mask+norm): [[], []]
        ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0]
        E: [1000000000, 0]; P: [1, 0] */
        for validator in 0..(n / 2) {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000
            ); // Note E = 1 * 1_000_000_000
        }
        for server in (n / 2)..n {
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, server), 0);
            // no stake
        }
        run_to_block(2);
        block_number += 1;

        // === Set weights [val->srv: 1/(n/2)]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / (n / 2); (n / 2) as usize],
                0
            ));
        }

        // === Outdate weights by reregistering servers
        for new_key in n..n + (n / 2) {
            // register a new key while at max capacity, which means the least emission uid will be deregistered
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                new_key as u64 * 1_000_000,
                &(U256::from(new_key)),
            );
            assert_ok!(SubtensorModule::register(
                RuntimeOrigin::signed(U256::from(new_key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(new_key),
                U256::from(new_key)
            ));
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 2; activity_cutoff: 5000; Last update: [2, 1]; Inactive: [false, false];
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 2];
        W: [[(1, 1)], []]; W (diagmask): [[(1, 1)], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []];
        R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0];
        B: [[], []]; B (mask+norm): [[], []];
        ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0];
        E: [1000000000, 0]; P: [1, 0] */
        for validator in 0..(n / 2) {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000
            ); // Note E = 1 * 1_000_000_000
        }
        for server in (n / 2)..n {
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, server), 0);
            // no stake
        }
        run_to_block(3);

        // === Set new weights [val->srv: 1/(n/2)] to check that updated weights would produce non-zero incentive
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![u16::MAX / (n / 2); (n / 2) as usize],
                0
            ));
        }
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*	current_block: 3; activity_cutoff: 5000; Last update: [3, 1]; Inactive: [false, false];
        S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 2];
        W: [[(1, 1)], []]; W (diagmask): [[(1, 1)], []]; W (diag+outdatemask): [[(1, 1)], []]; W (mask+norm): [[(1, 1)], []];
        R: [0, 1]; W (threshold): [[(1, 1)], []]; T: [0, 1]; C: [0.006693358, 0.9933076561]; I: [0, 1];
        B: [[], []]; B (mask+norm): [[], []];
        ΔB: [[(1, 1)], []]; ΔB (norm): [[(1, 1)], []]; emaB: [[(1, 1)], []]; D: [1, 0]; emaB (max-upscale): [[(1, 1)], []]
        E: [500000000, 500000000]; P: [0.5, 0.5] */
        for validator in 0..n {
            assert_eq!(
                SubtensorModule::get_emission_for_uid(netuid, validator),
                1000000000 / (n as u64)
            ); // Note E = 1/2 * 1_000_000_000
        }
    });
}

// Test that recently/deregistered miner bonds are cleared before EMA.
#[test]
fn test_deregistered_miner_bonds() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 4;
        let netuid: u16 = 1;
        let high_tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead

        let stake: u64 = 1;
        add_network(netuid, high_tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, n);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, n);
        SubtensorModule::set_target_registrations_per_interval(netuid, n);
        SubtensorModule::set_min_allowed_weights(netuid, 0);
        SubtensorModule::set_max_weight_limit(netuid, u16::MAX);
        SubtensorModule::set_bonds_penalty(netuid, u16::MAX);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);

        // === Register [validator1, validator2, server1, server2]
        let block_number = System::block_number();
        for key in 0..n as u64 {
            SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), stake);
            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                key * 1_000_000,
                &U256::from(key),
            );
            assert_ok!(SubtensorModule::register(
                RuntimeOrigin::signed(U256::from(key)),
                netuid,
                block_number,
                nonce,
                work,
                U256::from(key),
                U256::from(key)
            ));
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(key),
                &U256::from(key),
                netuid,
                stake,
            );
        }
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 4);

        // === Issue validator permits
        SubtensorModule::set_max_allowed_validators(netuid, n);
        assert_eq!(SubtensorModule::get_max_allowed_validators(netuid), n);
        SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 4);
        next_block(); // run to next block to ensure weights are set on nodes after their registration block
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);

        // === Set weights [val1->srv1: 2/3, val1->srv2: 1/3, val2->srv1: 2/3, val2->srv2: 1/3]
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![2 * (u16::MAX / 3), u16::MAX / 3],
                0
            ));
        }

        // Set tempo high so we don't automatically run epochs
        SubtensorModule::set_tempo(netuid, high_tempo);

        // Run 2 blocks
        next_block();
        next_block();

        // set tempo to 2 blocks
        SubtensorModule::set_tempo(netuid, 2);
        // Run epoch
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }

        // Check the bond values for the servers
        let bonds = SubtensorModule::get_bonds(netuid);
        let bond_0_2 = bonds[0][2];
        let bond_0_3 = bonds[0][3];

        // Non-zero bonds
        assert!(bond_0_2 > 0);
        assert!(bond_0_3 > 0);

        // Set tempo high so we don't automatically run epochs
        SubtensorModule::set_tempo(netuid, high_tempo);

        // Run one more block
        next_block();

        // === Dereg server2 at uid3 (least emission) + register new key over uid3
        let new_key: u64 = n as u64; // register a new key while at max capacity, which means the least incentive uid will be deregistered
        let block_number = System::block_number();
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            0,
            &U256::from(new_key),
        );
        assert_eq!(SubtensorModule::get_max_registrations_per_block(netuid), n);
        assert_eq!(SubtensorModule::get_registrations_this_block(netuid), 0);
        assert_ok!(SubtensorModule::register(
            RuntimeOrigin::signed(U256::from(new_key)),
            netuid,
            block_number,
            nonce,
            work,
            U256::from(new_key),
            U256::from(new_key)
        ));
        let deregistered_uid: u16 = n - 1; // since uid=n-1 only recieved 1/3 of weight, it will get pruned first
        assert_eq!(
            U256::from(new_key),
            SubtensorModule::get_hotkey_for_net_and_uid(netuid, deregistered_uid)
                .expect("Not registered")
        );

        // Set weights again so they're active.
        for uid in 0..(n / 2) as u64 {
            assert_ok!(SubtensorModule::set_weights(
                RuntimeOrigin::signed(U256::from(uid)),
                netuid,
                ((n / 2)..n).collect(),
                vec![2 * (u16::MAX / 3), u16::MAX / 3],
                0
            ));
        }

        // Run 1 block
        next_block();
        // Assert block at registration happened after the last tempo
        let block_at_registration = SubtensorModule::get_neuron_block_at_registration(netuid, 3);
        let block_number = System::block_number();
        assert!(
            block_at_registration >= block_number - 2,
            "block at registration: {}, block number: {}",
            block_at_registration,
            block_number
        );

        // set tempo to 2 blocks
        SubtensorModule::set_tempo(netuid, 2);
        // Run epoch again.
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }

        // Check the bond values for the servers
        let bonds = SubtensorModule::get_bonds(netuid);
        let bond_0_2_new = bonds[0][2];
        let bond_0_3_new = bonds[0][3];

        // We expect the old bonds for server2, (uid3), to be reset.
        // For server1, (uid2), the bond should be higher than before.
        assert!(
            bond_0_2_new >= bond_0_2,
            "bond_0_2_new: {}, bond_0_2: {}",
            bond_0_2_new,
            bond_0_2
        );
        assert!(
            bond_0_3_new <= bond_0_3,
            "bond_0_3_new: {}, bond_0_3: {}",
            bond_0_3_new,
            bond_0_3
        );
    });
}

// Test that epoch assigns validator permits to highest stake uids, varies uid interleaving and stake values.
#[test]
fn test_validator_permits() {
    let netuid: u16 = 1;
    let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
    for interleave in 0..3 {
        for (network_n, validators_n) in [(2, 1), (4, 2), (8, 4)] {
            for assignment in 0..=1 {
                let (validators, servers) =
                    distribute_nodes(validators_n as usize, network_n, interleave as usize);
                let correct: bool = true;
                let mut stake: Vec<u64> = vec![0; network_n];
                for validator in &validators {
                    stake[*validator as usize] = match assignment {
                        1 => *validator as u64 + network_n as u64,
                        _ => 1,
                    };
                }
                for server in &servers {
                    stake[*server as usize] = match assignment {
                        1 => *server as u64,
                        _ => 0,
                    };
                }
                new_test_ext(1).execute_with(|| {
                    let block_number: u64 = 0;
                    add_network(netuid, tempo, 0);
                    SubtensorModule::set_max_allowed_uids(netuid, network_n as u16);
                    assert_eq!(
                        SubtensorModule::get_max_allowed_uids(netuid),
                        network_n as u16
                    );
                    SubtensorModule::set_max_registrations_per_block(netuid, network_n as u16);
                    SubtensorModule::set_target_registrations_per_interval(
                        netuid,
                        network_n as u16,
                    );

                    // === Register [validator1, validator2, server1, server2]
                    for key in 0..network_n as u64 {
                        SubtensorModule::add_balance_to_coldkey_account(
                            &U256::from(key),
                            stake[key as usize],
                        );
                        let (nonce, work): (u64, Vec<u8>) =
                            SubtensorModule::create_work_for_block_number(
                                netuid,
                                block_number,
                                key * 1_000_000,
                                &U256::from(key),
                            );
                        assert_ok!(SubtensorModule::register(
                            RuntimeOrigin::signed(U256::from(key)),
                            netuid,
                            block_number,
                            nonce,
                            work,
                            U256::from(key),
                            U256::from(key)
                        ));
                        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                            &U256::from(key),
                            &U256::from(key),
                            netuid,
                            stake[key as usize],
                        );
                    }
                    assert_eq!(SubtensorModule::get_subnetwork_n(netuid), network_n as u16);

                    // === Issue validator permits
                    SubtensorModule::set_max_allowed_validators(netuid, validators_n as u16);
                    assert_eq!(
                        SubtensorModule::get_max_allowed_validators(netuid),
                        validators_n as u16
                    );
                    SubtensorModule::epoch(netuid, 1_000_000_000); // run first epoch to set allowed validators
                    for validator in &validators {
                        assert_eq!(
                            correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *validator)
                        );
                    }
                    for server in &servers {
                        assert_eq!(
                            !correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *server)
                        );
                    }

                    // === Increase server stake above validators
                    for server in &servers {
                        SubtensorModule::add_balance_to_coldkey_account(
                            &(U256::from(*server as u64)),
                            2 * network_n as u64,
                        );
                        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                            &(U256::from(*server as u64)),
                            &(U256::from(*server as u64)),
                            netuid,
                            2 * network_n as u64,
                        );
                    }

                    // === Update validator permits
                    run_to_block(1);
                    SubtensorModule::epoch(netuid, 1_000_000_000);

                    // === Check that servers now own permits instead of the validator uids
                    for validator in &validators {
                        assert_eq!(
                            !correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *validator)
                        );
                    }
                    for server in &servers {
                        assert_eq!(
                            correct,
                            SubtensorModule::get_validator_permit_for_uid(netuid, *server)
                        );
                    }
                });
            }
        }
    }
}

#[test]
fn test_get_set_alpha() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let alpha_low: u16 = 12_u16;
        let alpha_high: u16 = u16::MAX - 10;

        let hotkey: U256 = U256::from(1);
        let coldkey: U256 = U256::from(1 + 456);
        let signer = RuntimeOrigin::signed(coldkey);

        // Enable Liquid Alpha and setup
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey,));

        // Should fail as signer does not own the subnet
        assert_err!(
            SubtensorModule::do_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            DispatchError::BadOrigin
        );

        assert_ok!(SubtensorModule::register_network(signer.clone(), hotkey));
        assert_ok!(SubtensorModule::add_stake(
            signer.clone(),
            hotkey,
            netuid,
            DefaultMinStake::<Test>::get() + DefaultStakingFee::<Test>::get()
        ));

        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));
        let (grabbed_alpha_low, grabbed_alpha_high): (u16, u16) =
            SubtensorModule::get_alpha_values(netuid);

        log::info!(
            "alpha_low: {:?} alpha_high: {:?}",
            grabbed_alpha_low,
            grabbed_alpha_high
        );
        assert_eq!(grabbed_alpha_low, alpha_low);
        assert_eq!(grabbed_alpha_high, alpha_high);

        // Convert the u16 values to decimal values
        fn unnormalize_u16_to_float(normalized_value: u16) -> f32 {
            const MAX_U16: u16 = 65535;
            normalized_value as f32 / MAX_U16 as f32
        }

        let alpha_low_decimal = unnormalize_u16_to_float(alpha_low);
        let alpha_high_decimal = unnormalize_u16_to_float(alpha_high);

        let (alpha_low_32, alpha_high_32) = SubtensorModule::get_alpha_values_32(netuid);

        let tolerance: f32 = 1e-6; // 0.000001

        // Check if the values are equal to the sixth decimal
        assert!(
            (alpha_low_32.to_num::<f32>() - alpha_low_decimal).abs() < tolerance,
            "alpha_low mismatch: {} != {}",
            alpha_low_32.to_num::<f32>(),
            alpha_low_decimal
        );
        assert!(
            (alpha_high_32.to_num::<f32>() - alpha_high_decimal).abs() < tolerance,
            "alpha_high mismatch: {} != {}",
            alpha_high_32.to_num::<f32>(),
            alpha_high_decimal
        );

        // 1. Liquid alpha disabled
        SubtensorModule::set_liquid_alpha_enabled(netuid, false);
        assert_err!(
            SubtensorModule::do_set_alpha_values(signer.clone(), netuid, alpha_low, alpha_high),
            Error::<Test>::LiquidAlphaDisabled
        );
        // Correct scenario after error
        SubtensorModule::set_liquid_alpha_enabled(netuid, true); // Re-enable for further tests
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 2. Alpha high too low
        let alpha_high_too_low = (u16::MAX as u32 * 4 / 5) as u16 - 1; // One less than the minimum acceptable value
        assert_err!(
            SubtensorModule::do_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low,
                alpha_high_too_low
            ),
            Error::<Test>::AlphaHighTooLow
        );
        // Correct scenario after error
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        // 3. Alpha low too low or too high
        let alpha_low_too_low = 0_u16;
        assert_err!(
            SubtensorModule::do_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_low,
                alpha_high
            ),
            Error::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));

        let alpha_low_too_high = (u16::MAX as u32 * 4 / 5) as u16 + 1; // One more than the maximum acceptable value
        assert_err!(
            SubtensorModule::do_set_alpha_values(
                signer.clone(),
                netuid,
                alpha_low_too_high,
                alpha_high
            ),
            Error::<Test>::AlphaLowOutOfRange
        );
        // Correct scenario after error
        assert_ok!(SubtensorModule::do_set_alpha_values(
            signer.clone(),
            netuid,
            alpha_low,
            alpha_high
        ));
    });
}

#[test]
fn test_blocks_since_last_step() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);

        let netuid: u16 = 1;
        let tempo: u16 = 7200;
        add_network(netuid, tempo, 0);

        let original_blocks: u64 = SubtensorModule::get_blocks_since_last_step(netuid);

        step_block(5);

        let new_blocks: u64 = SubtensorModule::get_blocks_since_last_step(netuid);

        assert!(new_blocks > original_blocks);
        assert_eq!(new_blocks, 5);

        let blocks_to_step: u16 = SubtensorModule::blocks_until_next_epoch(
            netuid,
            tempo,
            SubtensorModule::get_current_block_as_u64(),
        ) as u16
            + 10;
        step_block(blocks_to_step);

        let post_blocks: u64 = SubtensorModule::get_blocks_since_last_step(netuid);

        assert_eq!(post_blocks, 10);

        let blocks_to_step: u16 = SubtensorModule::blocks_until_next_epoch(
            netuid,
            tempo,
            SubtensorModule::get_current_block_as_u64(),
        ) as u16
            + 20;
        step_block(blocks_to_step);

        let new_post_blocks: u64 = SubtensorModule::get_blocks_since_last_step(netuid);

        assert_eq!(new_post_blocks, 20);

        step_block(7);

        assert_eq!(SubtensorModule::get_blocks_since_last_step(netuid), 27);
    });
}

#[test]
fn test_can_set_self_weight_as_subnet_owner() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey: U256 = U256::from(1);
        let subnet_owner_hotkey: U256 = U256::from(1 + 456);

        let other_hotkey: U256 = U256::from(2);

        let stake = 5_000_000_000_000; // 5k TAO
        let to_emit: u64 = 1_000_000_000; // 1 TAO

        // Create subnet
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Register the other hotkey
        register_ok_neuron(netuid, other_hotkey, subnet_owner_coldkey, 0);

        // Add stake to owner hotkey.
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &subnet_owner_hotkey,
            &subnet_owner_coldkey,
            netuid,
            stake,
        );

        // Give vpermits to owner hotkey ONLY
        ValidatorPermit::<Test>::insert(netuid, vec![true, false]);

        // Set weight of 50% to each hotkey.
        // This includes a self-weight
        let fifty_percent: u16 = u16::MAX / 2;
        Weights::<Test>::insert(netuid, 0, vec![(0, fifty_percent), (1, fifty_percent)]);

        step_block(1);
        // Set updated so weights are valid
        LastUpdate::<Test>::insert(netuid, vec![2, 0]);

        // Run epoch
        let hotkey_emission: Vec<(U256, u64, u64)> = SubtensorModule::epoch(netuid, to_emit);

        // hotkey_emission is [(hotkey, incentive, dividend)]
        assert_eq!(hotkey_emission.len(), 2);
        assert_eq!(hotkey_emission[0].0, subnet_owner_hotkey);
        assert_eq!(hotkey_emission[1].0, other_hotkey);

        log::debug!("hotkey_emission: {:?}", hotkey_emission);
        // Both should have received incentive emission
        assert!(hotkey_emission[0].1 > 0);
        assert!(hotkey_emission[1].1 > 0);

        // Their incentive should be equal
        assert_eq!(hotkey_emission[0].1, hotkey_emission[1].1);
    });
}

#[test]
fn test_epoch_outputs_single_staker_registered_no_weights() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let high_tempo: u16 = u16::MAX - 1; // Don't run automatically.
        add_network(netuid, high_tempo, 0);

        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        // Give non-zero alpha
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey, &coldkey, netuid, 1,
        );

        let pending_alpha: u64 = 1_000_000_000;
        let hotkey_emission: Vec<(U256, u64, u64)> = SubtensorModule::epoch(netuid, pending_alpha);

        let sum_incentives: u64 = hotkey_emission
            .iter()
            .map(|(_, incentive, _)| incentive)
            .sum();
        let sum_dividends: u64 = hotkey_emission
            .iter()
            .map(|(_, _, dividend)| dividend)
            .sum();

        assert_abs_diff_eq!(
            sum_incentives.saturating_add(sum_dividends),
            pending_alpha,
            epsilon = 1_000
        );
    });
}

// Map the retention graph for consensus guarantees with an single epoch on a graph with 512 nodes,
// of which the first 64 are validators, the graph is split into a major and minor set, each setting
// specific weight on itself and the complement on the other.
//
// ```import torch
// import matplotlib.pyplot as plt
// from matplotlib.pyplot import cm
// %matplotlib inline
//
// with open('finney_consensus_0.4.txt') as f:  # test output saved to finney_consensus.txt
//     retention_map = eval(f.read())
//
// major_ratios = {}
// avg_weight_devs = {}
// for major_stake, major_weight, minor_weight, avg_weight_dev, major_ratio in retention_map:
//     major_stake = f'{major_stake:.2f}'
//     maj, min = int(round(50 * major_weight)), int(round(50 * minor_weight))
//     avg_weight_devs.setdefault(major_stake, torch.zeros((51, 51)))
//     avg_weight_devs[major_stake][maj][min] = avg_weight_dev
//     major_ratios.setdefault(major_stake, torch.zeros((51, 51)))
//     major_ratios[major_stake][maj][min] = major_ratio
//
// _x = torch.linspace(0, 1, 51); _y = torch.linspace(0, 1, 51)
// x, y = torch.meshgrid(_x, _y, indexing='ij')
//
// fig = plt.figure(figsize=(6, 6), dpi=70); ax = fig.gca()
// ax.set_xticks(torch.arange(0, 1, 0.05)); ax.set_yticks(torch.arange(0, 1., 0.05))
// ax.set_xticklabels([f'{_:.2f}'[1:] for _ in torch.arange(0, 1., 0.05)])
// plt.grid(); plt.rc('grid', linestyle="dotted", color=[0.85, 0.85, 0.85])
//
// isolate = ['0.60']; stakes = [0.51, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 0.99]
// colors = cm.viridis(torch.linspace(0, 1, len(stakes) + 1))
// for i, stake in enumerate(stakes):
//     contours = plt.contour(x, y, major_ratios[f'{stake:.2f}'], levels=[0., stake], colors=[colors[i + 1]])
//     if f'{stake:.2f}' in isolate:
//         contours.collections[1].set_linewidth(3)
//     plt.clabel(contours, inline=True, fontsize=10)
//
// plt.title(f'Major emission [$stake_{{maj}}=emission_{{maj}}$ retention lines]')
// plt.ylabel('Minor self-weight'); plt.xlabel('Major self-weight'); plt.show()
// ```
// #[test]
// fn _map_consensus_guarantees() {
//     let netuid: u16 = 1;
//     let network_n: u16 = 512;
//     let validators_n: u16 = 64;
//     let epochs: u16 = 1;
//     let interleave = 0;
//     let weight_stddev: I32F32 = fixed(0.4);
//     let bonds_penalty: u16 = u16::MAX;
//     println!("[");
//     for _major_stake in vec![0.51, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 0.99] {
//         let major_stake: I32F32 = I32F32::from_num(_major_stake);
//         for _major_weight in 0..51 {
//             let major_weight: I32F32 = I32F32::from_num(50 - _major_weight) / I32F32::from_num(50);
//             for _minor_weight in 0..51 {
//                 let minor_weight: I32F32 =
//                     I32F32::from_num(50 - _minor_weight) / I32F32::from_num(50);
//                 let (
//                     validators,
//                     servers,
//                     major_validators,
//                     minor_validators,
//                     major_servers,
//                     minor_servers,
//                     stake,
//                     weights,
//                     avg_weight_dev,
//                 ) = split_graph(
//                     major_stake,
//                     major_weight,
//                     minor_weight,
//                     weight_stddev,
//                     validators_n as usize,
//                     network_n as usize,
//                     interleave as usize,
//                 );
//
//                 new_test_ext(1).execute_with(|| {
// 					init_run_epochs(netuid, network_n, &validators, &servers, epochs, 1, true, &stake, true, &weights, true, false, 0, true, bonds_penalty);
//
// 					let mut major_emission: I64F64 = I64F64::from_num(0);
// 					let mut minor_emission: I64F64 = I64F64::from_num(0);
// 					for set in vec![major_validators, major_servers] {
// 						for uid in set {
// 							major_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
// 						}
// 					}
// 					for set in vec![minor_validators, minor_servers] {
// 						for uid in set {
// 							minor_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
// 						}
// 					}
// 					let major_ratio: I32F32 = I32F32::from_num(major_emission / (major_emission + minor_emission));
// 					println!("[{major_stake}, {major_weight:.2}, {minor_weight:.2}, {avg_weight_dev:.3}, {major_ratio:.3}], ");
// 				});
//             }
//         }
//     }
//     println!("]");
// }

// Helpers

/// Asserts that two I32F32 values are approximately equal within a given epsilon.
///
/// # Arguments
/// * `left` - The first value to compare.
/// * `right` - The second value to compare.
/// * `epsilon` - The maximum allowed difference between the two values.
pub fn assert_approx_eq(left: I32F32, right: I32F32, epsilon: I32F32) {
    if (left - right).abs() > epsilon {
        panic!(
            "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`,\n epsilon: `{:?}`",
            left, right, epsilon
        );
    }
}

// test Yuma 4 scenarios over a sequence of epochs.
fn setup_yuma_3_scenario(netuid: u16, n: u16, sparse: bool, max_stake: u64, stakes: Vec<u64>) {
    let block_number = System::block_number();
    let tempo: u16 = u16::MAX - 1; // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
    add_network(netuid, tempo, 0);

    SubtensorModule::set_max_allowed_uids(netuid, n);
    assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
    SubtensorModule::set_max_registrations_per_block(netuid, n);
    SubtensorModule::set_target_registrations_per_interval(netuid, n);
    SubtensorModule::set_weights_set_rate_limit(netuid, 0);
    SubtensorModule::set_min_allowed_weights(netuid, 1);
    SubtensorModule::set_max_weight_limit(netuid, u16::MAX);
    SubtensorModule::set_bonds_penalty(netuid, 0);
    SubtensorModule::set_alpha_sigmoid_steepness(netuid, 10);

    // === Register
    for key in 0..n as u64 {
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(key), max_stake);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            key * 1_000_000,
            &U256::from(key),
        );
        assert_ok!(SubtensorModule::register(
            <<Test as frame_system::Config>::RuntimeOrigin>::signed(U256::from(key)),
            netuid,
            block_number,
            nonce,
            work,
            U256::from(key),
            U256::from(key)
        ));
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &U256::from(key),
            &U256::from(key),
            netuid,
            stakes[key as usize],
        );
    }
    assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
    assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

    // Enable Liquid Alpha
    SubtensorModule::set_kappa(netuid, u16::MAX / 2);
    SubtensorModule::set_liquid_alpha_enabled(netuid, true);
    SubtensorModule::set_alpha_values_32(netuid, I32F32::from_num(0.1), I32F32::from_num(0.3));

    // === Issue validator permits
    SubtensorModule::set_max_allowed_validators(netuid, 3);

    // run first epoch to set allowed validators
    // run to next block to ensure weights are set on nodes after their registration block
    run_epoch(netuid, sparse);
}

fn run_epoch(netuid: u16, sparse: bool) {
    next_block();
    if sparse {
        SubtensorModule::epoch(netuid, 1_000_000_000);
    } else {
        SubtensorModule::epoch_dense(netuid, 1_000_000_000);
    }
}

fn run_epoch_and_check_bonds_dividends(
    netuid: u16,
    sparse: bool,
    target_bonds: &[Vec<f32>],
    target_dividends: &[f32],
) {
    run_epoch(netuid, sparse);
    let bonds = SubtensorModule::get_bonds_fixed_proportion(netuid);
    let dividends = SubtensorModule::get_dividends(netuid);

    let epsilon = I32F32::from_num(1e-3);
    // Check the bonds
    for (bond, target_bond) in bonds.iter().zip(target_bonds.iter()) {
        // skip the 3 validators
        for (b, t) in bond.iter().zip(target_bond.iter().skip(3)) {
            assert_approx_eq(*b, fixed(*t), epsilon);
        }
    }
    // Check the dividends
    for (dividend, target_dividend) in dividends.iter().zip(target_dividends.iter()) {
        assert_approx_eq(
            u16_proportion_to_fixed(*dividend),
            fixed(*target_dividend),
            epsilon,
        );
    }
}

fn set_yuma_3_weights(netuid: u16, weights: Vec<Vec<u16>>, indices: Vec<u16>) {
    for (uid, weight) in weights.iter().enumerate() {
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid as u64)),
            netuid,
            indices.clone(),
            weight.to_vec(),
            0
        ));
    }
}

#[test]
fn test_yuma_3_kappa_moves_first() {
    for sparse in [true, false].iter() {
        new_test_ext(1).execute_with(|| {
            let n: u16 = 5; // 3 validators, 2 servers
            let netuid: u16 = 1;
            let max_stake: u64 = 8;

            // Validator A: kappa / Big validator (0.8) - moves first
            // Validator B: Small eager validator (0.1) - moves second
            // Validator C: Small lazy validator (0.1) - moves last
            let stakes: Vec<u64> = vec![8, 1, 1, 0, 0];

            setup_yuma_3_scenario(netuid, n, *sparse, max_stake, stakes);
            let targets_bonds = [
                vec![
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                ],
                vec![
                    vec![0.0908, 0.1013],
                    vec![0.3697, 0.0000],
                    vec![0.3697, 0.0000],
                ],
                vec![
                    vec![0.0815, 0.1924],
                    vec![0.3170, 0.1013],
                    vec![0.5580, 0.0000],
                ],
                vec![
                    vec![0.0731, 0.2742],
                    vec![0.2765, 0.1924],
                    vec![0.4306, 0.1013],
                ],
                vec![
                    vec![0.0656, 0.3478],
                    vec![0.2435, 0.2742],
                    vec![0.3589, 0.1924],
                ],
                vec![
                    vec![0.0588, 0.4139],
                    vec![0.2157, 0.3478],
                    vec![0.3089, 0.2742],
                ],
            ];

            let targets_dividends = [
                vec![0.8000, 0.1000, 0.1000, 0.0000, 0.0000],
                vec![1.0000, 0.0000, 0.0000, 0.0000, 0.0000],
                vec![0.9382, 0.0618, 0.0000, 0.0000, 0.0000],
                vec![0.8819, 0.0773, 0.0407, 0.0000, 0.0000],
                vec![0.8564, 0.0844, 0.0592, 0.0000, 0.0000],
                vec![0.8418, 0.0884, 0.0697, 0.0000, 0.0000],
            ];

            for (epoch, (target_bonds, target_dividends)) in targets_bonds
                .iter()
                .zip(targets_dividends.iter())
                .enumerate()
            {
                match epoch {
                    0 => {
                        // Initially, consensus is achieved by all Validators
                        set_yuma_3_weights(netuid, vec![vec![u16::MAX, 0]; 3], vec![3, 4]);
                    }
                    1 => {
                        // Validator A -> Server 2
                        // Validator B -> Server 1
                        // Validator C -> Server 1
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![0, u16::MAX], vec![u16::MAX, 0], vec![u16::MAX, 0]],
                            vec![3, 4],
                        );
                    }
                    2 => {
                        // Validator A -> Server 2
                        // Validator B -> Server 2
                        // Validator C -> Server 1
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![0, u16::MAX], vec![0, u16::MAX], vec![u16::MAX, 0]],
                            vec![3, 4],
                        );
                    }
                    3 => {
                        // Subsequent epochs All validators -> Server 2
                        set_yuma_3_weights(netuid, vec![vec![0, u16::MAX]; 3], vec![3, 4]);
                    }
                    _ => {}
                };
                run_epoch_and_check_bonds_dividends(netuid, *sparse, target_bonds, target_dividends);
            }
        })
    }
}

#[test]
fn test_yuma_3_kappa_moves_second() {
    for sparse in [true, false].iter() {
        new_test_ext(1).execute_with(|| {
            let n: u16 = 5; // 3 validators, 2 servers
            let netuid: u16 = 1;
            let max_stake: u64 = 8;

            // Validator A: kappa / Big validator (0.8) - moves second
            // Validator B: Small eager validator (0.1) - moves first
            // Validator C: Small lazy validator (0.1) - moves last
            let stakes: Vec<u64> = vec![8, 1, 1, 0, 0];

            setup_yuma_3_scenario(netuid, n, *sparse, max_stake, stakes);
            let targets_bonds = [
                vec![
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                ],
                vec![
                    vec![0.1924, 0.0000],
                    vec![0.0908, 0.2987],
                    vec![0.1924, 0.0000],
                ],
                vec![
                    vec![0.1715, 0.1013],
                    vec![0.0815, 0.3697],
                    vec![0.4336, 0.0000],
                ],
                vec![
                    vec![0.1531, 0.1924],
                    vec![0.0731, 0.4336],
                    vec![0.3608, 0.1013],
                ],
                vec![
                    vec![0.1369, 0.2742],
                    vec![0.0656, 0.4910],
                    vec![0.3103, 0.1924],
                ],
                vec![
                    vec![0.1225, 0.3478],
                    vec![0.0588, 0.5426],
                    vec![0.2712, 0.2742],
                ],
            ];
            let targets_dividends = [
                vec![0.8000, 0.1000, 0.1000, 0.0000, 0.0000],
                vec![0.8446, 0.0498, 0.1056, 0.0000, 0.0000],
                vec![0.6868, 0.3132, 0.0000, 0.0000, 0.0000],
                vec![0.7421, 0.2090, 0.0489, 0.0000, 0.0000],
                vec![0.7625, 0.1706, 0.0669, 0.0000, 0.0000],
                vec![0.7730, 0.1508, 0.0762, 0.0000, 0.0000],
            ];

            for (epoch, (target_bonds, target_dividends)) in targets_bonds
                .iter()
                .zip(targets_dividends.iter())
                .enumerate()
            {
                match epoch {
                    0 => {
                        // Initially, consensus is achieved by all Validators
                        set_yuma_3_weights(netuid, vec![vec![u16::MAX, 0]; 3], vec![3, 4]);
                    }
                    1 => {
                        // Validator A -> Server 1
                        // Validator B -> Server 2
                        // Validator C -> Server 1
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![u16::MAX, 0], vec![0, u16::MAX], vec![u16::MAX, 0]],
                            vec![3, 4],
                        );
                    }
                    2 => {
                        // Validator A -> Server 2
                        // Validator B -> Server 2
                        // Validator C -> Server 1
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![0, u16::MAX], vec![0, u16::MAX], vec![u16::MAX, 0]],
                            vec![3, 4],
                        );
                    }
                    3 => {
                        // Subsequent epochs All validators -> Server 2
                        set_yuma_3_weights(netuid, vec![vec![0, u16::MAX]; 3], vec![3, 4]);
                    }
                    _ => {}
                };
                run_epoch_and_check_bonds_dividends(netuid, *sparse, target_bonds, target_dividends);
            }
        })
    }
}

#[test]
fn test_yuma_3_kappa_moves_last() {
    for sparse in [true, false].iter() {
        new_test_ext(1).execute_with(|| {
            let n: u16 = 5; // 3 validators, 2 servers
            let netuid: u16 = 1;
            let max_stake: u64 = 8;

            // Validator A: kappa / Big validator (0.8) - moves last
            // Validator B: Small eager validator (0.1) - moves first
            // Validator C: Small lazy validator (0.1) - moves second
            let stakes: Vec<u64> = vec![8, 1, 1, 0, 0];

            setup_yuma_3_scenario(netuid, n, *sparse, max_stake, stakes);
            let targets_bonds = [
                vec![
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                ],
                vec![
                    vec![0.1924, 0.0000],
                    vec![0.0908, 0.2987],
                    vec![0.1924, 0.0000],
                ],
                vec![
                    vec![0.2742, 0.0000],
                    vec![0.0815, 0.5081],
                    vec![0.1715, 0.2987],
                ],
                vec![
                    vec![0.2416, 0.1013],
                    vec![0.0731, 0.5580],
                    vec![0.1531, 0.3697],
                ],
                vec![
                    vec![0.2141, 0.1924],
                    vec![0.0656, 0.6028],
                    vec![0.1369, 0.4336],
                ],
                vec![
                    vec![0.1903, 0.2742],
                    vec![0.0588, 0.6430],
                    vec![0.1225, 0.4910],
                ],
            ];
            let targets_dividends = [
                vec![0.8000, 0.1000, 0.1000, 0.0000, 0.0000],
                vec![0.8446, 0.0498, 0.1056, 0.0000, 0.0000],
                vec![0.8966, 0.0333, 0.0701, 0.0000, 0.0000],
                vec![0.4663, 0.3210, 0.2127, 0.0000, 0.0000],
                vec![0.5976, 0.2340, 0.1683, 0.0000, 0.0000],
                vec![0.6592, 0.1932, 0.1475, 0.0000, 0.0000],
            ];

            for (epoch, (target_bonds, target_dividends)) in targets_bonds
                .iter()
                .zip(targets_dividends.iter())
                .enumerate()
            {
                match epoch {
                    0 => {
                        // Initially, consensus is achieved by all Validators
                        set_yuma_3_weights(netuid, vec![vec![u16::MAX, 0]; 3], vec![3, 4]);
                    }
                    1 => {
                        // Validator A -> Server 1
                        // Validator B -> Server 2
                        // Validator C -> Server 1
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![u16::MAX, 0], vec![0, u16::MAX], vec![u16::MAX, 0]],
                            vec![3, 4],
                        );
                    }
                    2 => {
                        // Validator A -> Server 1
                        // Validator B -> Server 2
                        // Validator C -> Server 2
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![u16::MAX, 0], vec![0, u16::MAX], vec![0, u16::MAX]],
                            vec![3, 4],
                        );
                    }
                    3 => {
                        // Subsequent epochs All validators -> Server 2
                        set_yuma_3_weights(netuid, vec![vec![0, u16::MAX]; 3], vec![3, 4]);
                    }
                    _ => {}
                };
                run_epoch_and_check_bonds_dividends(netuid, *sparse, target_bonds, target_dividends);
            }
        })
    }
}

#[test]
fn test_yuma_3_one_epoch_switch() {
    for sparse in [true, false].iter() {
        new_test_ext(1).execute_with(|| {
            let n: u16 = 5; // 3 validators, 2 servers
            let netuid: u16 = 1;
            let max_stake: u64 = 8;

            // Equal stake validators
            let stakes: Vec<u64> = vec![33, 33, 34, 0, 0];

            setup_yuma_3_scenario(netuid, n, *sparse, max_stake, stakes);

            let targets_bonds = [
                vec![
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                    vec![0.1013, 0.0000],
                ],
                vec![
                    vec![0.1924, 0.0000],
                    vec![0.1924, 0.0000],
                    vec![0.1924, 0.0000],
                ],
                vec![
                    vec![0.2742, 0.0000],
                    vec![0.2742, 0.0000],
                    vec![0.1715, 0.2987],
                ],
                vec![
                    vec![0.3478, 0.0000],
                    vec![0.3478, 0.0000],
                    vec![0.2554, 0.2618],
                ],
                vec![
                    vec![0.4139, 0.0000],
                    vec![0.4139, 0.0000],
                    vec![0.3309, 0.2312],
                ],
                vec![
                    vec![0.4733, 0.0000],
                    vec![0.4733, 0.0000],
                    vec![0.3987, 0.2051],
                ],
            ];
            let targets_dividends = [
                vec![0.3300, 0.3300, 0.3400, 0.0000, 0.0000],
                vec![0.3300, 0.3300, 0.3400, 0.0000, 0.0000],
                vec![0.3782, 0.3782, 0.2436, 0.0000, 0.0000],
                vec![0.3628, 0.3628, 0.2745, 0.0000, 0.0000],
                vec![0.3541, 0.3541, 0.2917, 0.0000, 0.0000],
                vec![0.3487, 0.3487, 0.3026, 0.0000, 0.0000],
            ];

            for (epoch, (target_bonds, target_dividends)) in targets_bonds
                .iter()
                .zip(targets_dividends.iter())
                .enumerate()
            {
                match epoch {
                    2 => {
                        // Validator A -> Server 1
                        // Validator B -> Server 1
                        // Validator C -> Server 2
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![u16::MAX, 0], vec![u16::MAX, 0], vec![0, u16::MAX]],
                            vec![3, 4],
                        );
                    }
                    _ => {
                        // All validators -> Server 1
                        set_yuma_3_weights(netuid, vec![vec![u16::MAX, 0]; 3], vec![3, 4]);
                    }
                };
                run_epoch_and_check_bonds_dividends(netuid, *sparse, target_bonds, target_dividends);
            }
        })
    }
}

#[test]
fn test_yuma_3_stable_miner() {
    for sparse in [true, false].iter() {
        new_test_ext(1).execute_with(|| {
            let netuid: u16 = 1;
            let n: u16 = 6; // 3 validators, 3 servers
            let max_stake: u64 = 8;

            // Validator A: kappa / Big validator (0.8)
            // Validator B: Small eager validator (0.1)
            // Validator C: Small lazy validator (0.1)
            let stakes: Vec<u64> = vec![8, 1, 1, 0, 0, 0];

            setup_yuma_3_scenario(netuid, n, *sparse, max_stake, stakes);
            let targets_bonds = [
                vec![
                    vec![0.0507, 0.0000, 0.0507],
                    vec![0.0507, 0.0000, 0.0507],
                    vec![0.0507, 0.0000, 0.0507],
                ],
                vec![
                    vec![0.0962, 0.0000, 0.0962],
                    vec![0.0455, 0.1000, 0.0962],
                    vec![0.0962, 0.0000, 0.0962],
                ],
                vec![
                    vec![0.0863, 0.0507, 0.1371],
                    vec![0.0408, 0.1405, 0.1371],
                    vec![0.1770, 0.0000, 0.1371],
                ],
                vec![
                    vec![0.0774, 0.0962, 0.1739],
                    vec![0.0367, 0.1770, 0.1739],
                    vec![0.1579, 0.0507, 0.1739],
                ],
                vec![
                    vec![0.0694, 0.1371, 0.2069],
                    vec![0.0329, 0.2097, 0.2069],
                    vec![0.1411, 0.0962, 0.2069],
                ],
                vec![
                    vec![0.0623, 0.1739, 0.2366],
                    vec![0.0296, 0.2391, 0.2366],
                    vec![0.1263, 0.1371, 0.2366],
                ],
            ];
            let targets_dividends = [
                vec![0.8000, 0.1000, 0.1000, 0.0000, 0.0000, 0.0000],
                vec![0.8226, 0.0745, 0.1028, 0.0000, 0.0000, 0.0000],
                vec![0.7750, 0.1685, 0.0565, 0.0000, 0.0000, 0.0000],
                vec![0.7864, 0.1372, 0.0764, 0.0000, 0.0000, 0.0000],
                vec![0.7912, 0.1241, 0.0847, 0.0000, 0.0000, 0.0000],
                vec![0.7937, 0.1173, 0.0890, 0.0000, 0.0000, 0.0000],
            ];

            for (epoch, (target_bonds, target_dividends)) in targets_bonds
                .iter()
                .zip(targets_dividends.iter())
                .enumerate()
            {
                match epoch {
                    0 => {
                        // all validators 0.5 for first and third server
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![u16::MAX / 2, 0, u16::MAX / 2]; 3],
                            vec![3, 4, 5],
                        );
                    }
                    1 => {
                        // one of small validators moves 0.5 to seconds server
                        set_yuma_3_weights(
                            netuid,
                            vec![
                                vec![u16::MAX / 2, 0, u16::MAX / 2],
                                vec![0, u16::MAX / 2, u16::MAX / 2],
                                vec![u16::MAX / 2, 0, u16::MAX / 2],
                            ],
                            vec![3, 4, 5],
                        );
                    }
                    2 => {
                        // big validator follows
                        set_yuma_3_weights(
                            netuid,
                            vec![
                                vec![0, u16::MAX / 2, u16::MAX / 2],
                                vec![0, u16::MAX / 2, u16::MAX / 2],
                                vec![u16::MAX / 2, 0, u16::MAX / 2],
                            ],
                            vec![3, 4, 5],
                        );
                    }
                    3 => {
                        // Subsequent epochs all validators have moves
                        set_yuma_3_weights(
                            netuid,
                            vec![vec![0, u16::MAX / 2, u16::MAX / 2]; 3],
                            vec![3, 4, 5],
                        );
                    }
                    _ => {}
                };
                run_epoch_and_check_bonds_dividends(
                    netuid,
                    *sparse,
                    target_bonds,
                    target_dividends,
                );
            }
        })
    }
}
