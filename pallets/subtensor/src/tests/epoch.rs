#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use super::mock::*;
use crate::epoch::math::{fixed, mat_fixed_proportions_to_fixed, u16_proportion_to_fixed};
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
                    assert_eq!(bonds[uid as usize][server], I32F32::from_num(38));
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
            current_block: 2, activity_cutoff: 5000, Last update: [2, 2, 2, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            W: [[(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Tv: [0.9999999995, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0.099997558, 0.2000012202, 0.2999926745, 0.4000085443]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926752, 0.4000085455]
            B: [[], [], [], [], [], [], [], []]
            B (outdatedmask): [[], [], [], [], [], [], [], []]
            B: [[], [], [], [], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0099997558), (5, 0.020000122), (6, 0.0299992675), (7, 0.0400008545)], [(4, 0.0099997558), (5, 0.020000122), (6, 0.0299992675), (7, 0.0400008545)], [(4, 0.0099997558), (5, 0.020000122), (6, 0.0299992675), (7, 0.0400008545)], [(4, 0.0099997558), (5, 0.020000122), (6, 0.0299992675), (7, 0.0400008545)], [], [], [], []]
            emaB norm: [[(4, 0.25), (5, 0.25), (6, 0.25), (7, 0.25)], [(4, 0.25), (5, 0.25), (6, 0.25), (7, 0.25)], [(4, 0.25), (5, 0.25), (6, 0.25), (7, 0.25)], [(4, 0.25), (5, 0.25), (6, 0.25), (7, 0.25)], [], [], [], []]
            total_bonds_per_validator: [0.2499999995, 0.2499999995, 0.2499999995, 0.2499999995, 0, 0, 0, 0]
            D: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            nE: [0.0499999998, 0.0999999999, 0.15, 0.2, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
            E: [49999999, 99999999, 149999999, 199999999, 49998779, 100000610, 149996337, 200004272]
            P: [0.0499999998, 0.0999999999, 0.15, 0.2, 0.049998779, 0.1000006103, 0.1499963375, 0.2000042726]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][4], 655);
        assert_eq!(bonds[1][4], 655);
        assert_eq!(bonds[2][4], 655);
        assert_eq!(bonds[3][4], 655);

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
            current_block: 3, activity_cutoff: 5000, Last update: [2, 2, 2, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            W: [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Tv: [0, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            B: [[(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [], [], [], []]
            B (outdatedmask): [[(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [], [], [], []]
            B: [[(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0089951933), (5, 0.0179903866), (6, 0.0269993132), (7, 0.0359945067)], [(4, 0.018994949), (5, 0.0379905086), (6, 0.0569985807), (7, 0.0759953612)], [(4, 0.018994949), (5, 0.0379905086), (6, 0.0569985807), (7, 0.0759953612)], [(4, 0.018994949), (5, 0.0379905086), (6, 0.0569985807), (7, 0.0759953612)], [], [], [], []]
            emaB norm: [[(4, 0.1363320365), (5, 0.1363301442), (6, 0.136363573), (7, 0.1363528532)], [(4, 0.287889321), (5, 0.2878899518), (6, 0.2878788088), (7, 0.287882382)], [(4, 0.287889321), (5, 0.2878899518), (6, 0.2878788088), (7, 0.287882382)], [(4, 0.287889321), (5, 0.2878899518), (6, 0.2878788088), (7, 0.287882382)], [], [], [], []]
            total_bonds_per_validator: [0.136349445, 0.2878835173, 0.2878835173, 0.2878835173, 0, 0, 0, 0]
            D: [0.0499942757, 0.211112383, 0.3166685747, 0.422224766, 0, 0, 0, 0]
            nE: [0.0249971377, 0.1055561914, 0.1583342873, 0.211112383, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            E: [24997137, 105556191, 158334287, 211112383, 49998779, 100000610, 149996337, 200004272]
            P: [0.0249971377, 0.1055561914, 0.1583342873, 0.211112383, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */
        assert_eq!(bonds[0][4], 655);
        assert_eq!(bonds[1][4], 655);
        assert_eq!(bonds[2][4], 655);
        assert_eq!(bonds[3][4], 655);

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
        /*  current_block: 3
            current_block: 4, activity_cutoff: 5000, Last update: [2, 3, 2, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            W: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Tv: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            B: [[(4, 589), (5, 1178), (6, 1769), (7, 2358)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [], [], [], []]
            B (outdatedmask): [[(4, 589), (5, 1178), (6, 1769), (7, 2358)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [], [], [], []]
            B: [[(4, 0.008987564), (5, 0.0179751278), (6, 0.0269932097), (7, 0.0359807736)], [(4, 0.0189822232), (5, 0.0379797055), (6, 0.0569924468), (7, 0.075989929)], [(4, 0.0189822232), (5, 0.0379797055), (6, 0.0569924468), (7, 0.075989929)], [(4, 0.0189822232), (5, 0.0379797055), (6, 0.0569924468), (7, 0.075989929)], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0080888073), (5, 0.016177615), (6, 0.0242938886), (7, 0.0323826962)], [(4, 0.0170840009), (5, 0.0341817348), (6, 0.051293202), (7, 0.068390936)], [(4, 0.0270837566), (5, 0.0541818568), (6, 0.0812924695), (7, 0.1083917904)], [(4, 0.0270837566), (5, 0.0541818568), (6, 0.0812924695), (7, 0.1083917904)], [], [], [], []]
            emaB norm: [[(4, 0.1019507758), (5, 0.10192353), (6, 0.102001434), (7, 0.101974368)], [(4, 0.2153255814), (5, 0.2153545555), (6, 0.2153619886), (7, 0.215365714)], [(4, 0.3413618212), (5, 0.341360957), (6, 0.3413182884), (7, 0.3413299588)], [(4, 0.3413618212), (5, 0.341360957), (6, 0.3413182884), (7, 0.3413299588)], [], [], [], []]
            total_bonds_per_validator: [0.1019699604, 0.215358351, 0.3413358429, 0.3413358429, 0, 0, 0, 0]
            D: [0.034896868, 0.1474028623, 0.3504429725, 0.4672572967, 0, 0, 0, 0]
            nE: [0.017448434, 0.073701431, 0.1752214862, 0.2336286483, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            E: [17448433, 73701431, 175221486, 233628648, 49998779, 100000610, 149996337, 200004272]
            P: [0.017448434, 0.073701431, 0.1752214862, 0.2336286483, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][4], 530);
        assert_eq!(bonds[1][4], 1119);
        assert_eq!(bonds[2][4], 1774);
        assert_eq!(bonds[3][4], 1774);

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
        /*  current_block: 4
            current_block: 5, activity_cutoff: 5000, Last update: [2, 4, 2, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            W: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Tv: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            B: [[(4, 530), (5, 1060), (6, 1592), (7, 2122)], [(4, 1119), (5, 2240), (6, 3361), (7, 4481)], [(4, 1774), (5, 3550), (6, 5327), (7, 7103)], [(4, 1774), (5, 3550), (6, 5327), (7, 7103)], [], [], [], []]
            B (outdatedmask): [[(4, 530), (5, 1060), (6, 1592), (7, 2122)], [(4, 1119), (5, 2240), (6, 3361), (7, 4481)], [(4, 1774), (5, 3550), (6, 5327), (7, 7103)], [(4, 1774), (5, 3550), (6, 5327), (7, 7103)], [], [], [], []]
            B: [[(4, 0.0080872816), (5, 0.0161745632), (6, 0.0242923629), (7, 0.0323796445)], [(4, 0.0170748455), (5, 0.034180209), (6, 0.0512855726), (7, 0.068375677)], [(4, 0.0270695048), (5, 0.0541695277), (6, 0.0812848096), (7, 0.1083848325)], [(4, 0.0270695048), (5, 0.0541695277), (6, 0.0812848096), (7, 0.1083848325)], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0072785532), (5, 0.0145571067), (6, 0.0218631264), (7, 0.0291416799)], [(4, 0.0153673608), (5, 0.030762188), (6, 0.0461570153), (7, 0.0615381093)], [(4, 0.03436231), (5, 0.0687526967), (6, 0.1031555962), (7, 0.1375472036)], [(4, 0.03436231), (5, 0.0687526967), (6, 0.1031555962), (7, 0.1375472036)], [], [], [], []]
            emaB norm: [[(4, 0.0796597423), (5, 0.079623309), (6, 0.0796960597), (7, 0.0796712292)], [(4, 0.1681872709), (5, 0.168260579), (6, 0.1682528006), (7, 0.1682407067)], [(4, 0.3760764932), (5, 0.3760580558), (6, 0.3760255696), (7, 0.376044032)], [(4, 0.3760764932), (5, 0.3760580558), (6, 0.3760255696), (7, 0.376044032)], [], [], [], []]
            total_bonds_per_validator: [0.079667945, 0.1682429651, 0.3760445435, 0.3760445435, 0, 0, 0, 0]
            D: [0.0261337839, 0.1103787823, 0.3700660428, 0.493421391, 0, 0, 0, 0]
            nE: [0.0130668918, 0.0551893911, 0.1850330213, 0.2467106953, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            E: [13066891, 55189391, 185033021, 246710695, 49998779, 100000610, 149996337, 200004272]
            P: [0.0130668918, 0.0551893911, 0.1850330213, 0.2467106953, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 1909);
        assert_eq!(bonds[1][7], 4032);
        assert_eq!(bonds[2][7], 9014);
        assert_eq!(bonds[3][7], 9014);

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
        /*  current_block: 5
            current_block: 6, activity_cutoff: 5000, Last update: [2, 4, 5, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
            W: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            Tv: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
            T: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
            I (=R): [0, 0, 0, 0, 0, 0, 0, 1]
            B: [[(4, 476), (5, 953), (6, 1432), (7, 1909)], [(4, 1007), (5, 2015), (6, 3024), (7, 4032)], [(4, 2251), (5, 4505), (6, 6760), (7, 9014)], [(4, 2251), (5, 4505), (6, 6760), (7, 9014)], [], [], [], []]
            B (outdatedmask): [[(4, 476), (5, 953), (6, 1432), (7, 1909)], [(4, 1007), (5, 2015), (6, 3024), (7, 4032)], [(4, 2251), (5, 4505), (6, 6760), (7, 9014)], [(4, 2251), (5, 4505), (6, 6760), (7, 9014)], [], [], [], []]
            B: [[(4, 0.0072632944), (5, 0.0145418479), (6, 0.0218509194), (7, 0.0291294728)], [(4, 0.015365835), (5, 0.030746929), (6, 0.0461432822), (7, 0.0615243763)], [(4, 0.0343480583), (5, 0.0687418936), (6, 0.103150988), (7, 0.1375448233)], [(4, 0.0343480583), (5, 0.0687418936), (6, 0.103150988), (7, 0.1375448233)], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0065369648), (5, 0.0130876629), (6, 0.0196658273), (7, 0.0262165254)], [(4, 0.0138292515), (5, 0.027672236), (6, 0.041528954), (7, 0.0553719385)], [(4, 0.0309132524), (5, 0.0618677041), (6, 0.092835889), (7, 0.1637911955)], [(4, 0.0309132524), (5, 0.0618677041), (6, 0.092835889), (7, 0.1637911955)], [], [], [], []]
            emaB norm: [[(4, 0.0795321616), (5, 0.0795625302), (6, 0.0796617707), (7, 0.0640723184)], [(4, 0.1682539685), (5, 0.168225079), (6, 0.168224299), (7, 0.1353271813)], [(4, 0.3761069346), (5, 0.3761061952), (6, 0.3760569647), (7, 0.40030025)], [(4, 0.3761069346), (5, 0.3761061952), (6, 0.3760569647), (7, 0.40030025)], [], [], [], []]
            total_bonds_per_validator: [0.0640723184, 0.1353271813, 0.40030025, 0.40030025, 0, 0, 0, 0]
            D: [0.020425828, 0.0862828067, 0.3828391563, 0.5104522086, 0, 0, 0, 0]
            nE: [0.0102129139, 0.0431414032, 0.1914195782, 0.2552261043, 0, 0, 0, 0.5]
            E: [10212913, 43141403, 191419578, 255226104, 0, 0, 0, 500000000]
            P: [0.0102129139, 0.0431414032, 0.1914195782, 0.2552261043, 0, 0, 0, 0.5]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 1718);
        assert_eq!(bonds[1][7], 3628);
        assert_eq!(bonds[2][7], 10734);
        assert_eq!(bonds[3][7], 10734);

        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  current_block: 6
           current_block: 7, activity_cutoff: 5000, Last update: [2, 4, 5, 2, 1, 1, 1, 1]
           Inactive: [false, false, false, false, false, false, false, false]
           Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
           hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
           S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
           validator_permits: [true, true, true, true, true, true, true, true]
           max_allowed_validators: 8
           new_validator_permits: [true, true, true, true, true, true, true, true]
           S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
           W: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           W (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           W (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           W (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           C: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
           W: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
           Tv: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
           R (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
           T: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
           I (=R): [0, 0, 0, 0, 0, 0, 0, 1]
           B: [[(4, 428), (5, 857), (6, 1288), (7, 1718)], [(4, 906), (5, 1813), (6, 2721), (7, 3628)], [(4, 2025), (5, 4054), (6, 6083), (7, 10734)], [(4, 2025), (5, 4054), (6, 6083), (7, 10734)], [], [], [], []]
           B (outdatedmask): [[(4, 428), (5, 857), (6, 1288), (7, 1718)], [(4, 906), (5, 1813), (6, 2721), (7, 3628)], [(4, 2025), (5, 4054), (6, 6083), (7, 10734)], [(4, 2025), (5, 4054), (6, 6083), (7, 10734)], [], [], [], []]
           B: [[(4, 0.0065308614), (5, 0.0130769818), (6, 0.0196536202), (7, 0.0262149996)], [(4, 0.0138246738), (5, 0.0276646067), (6, 0.0415197986), (7, 0.0553597314)], [(4, 0.0308995193), (5, 0.0618600748), (6, 0.0928206302), (7, 0.163790341)], [(4, 0.0308995193), (5, 0.0618600748), (6, 0.0928206302), (7, 0.163790341)], [], [], [], []]
           alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
           emaB: [[(4, 0.0058777751), (5, 0.0117692836), (6, 0.017688258), (7, 0.0235934996)], [(4, 0.0124422063), (5, 0.0248981458), (6, 0.0373678186), (7, 0.0498237582)], [(4, 0.0278095673), (5, 0.0556740672), (6, 0.083538567), (7, 0.1874121614)], [(4, 0.0278095673), (5, 0.0556740672), (6, 0.083538567), (7, 0.1874121614)], [], [], [], []]
           emaB norm: [[(4, 0.0794947986), (5, 0.0795138243), (6, 0.0796290569), (7, 0.052635678)], [(4, 0.168276373), (5, 0.1682130254), (6, 0.1682225657), (7, 0.111153807)], [(4, 0.376114414), (5, 0.376136575), (6, 0.3760741884), (7, 0.4181052572)], [(4, 0.376114414), (5, 0.376136575), (6, 0.3760741884), (7, 0.4181052572)], [], [], [], []]
           total_bonds_per_validator: [0.052635678, 0.111153807, 0.4181052572, 0.4181052572, 0, 0, 0, 0]
           D: [0.0164400174, 0.069434674, 0.391767989, 0.5223573192, 0, 0, 0, 0]
           nE: [0.0082200086, 0.034717337, 0.1958839945, 0.2611786595, 0, 0, 0, 0.5]
           E: [8220008, 34717336, 195883994, 261178659, 0, 0, 0, 500000000]
           P: [0.0082200086, 0.034717337, 0.1958839945, 0.2611786595, 0, 0, 0, 0.5]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 1546);
        assert_eq!(bonds[1][7], 3265);
        assert_eq!(bonds[2][7], 12282);
        assert_eq!(bonds[3][7], 12282);

        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  current_block: 7
           current_block: 8, activity_cutoff: 5000, Last update: [2, 4, 5, 2, 1, 1, 1, 1]
           Inactive: [false, false, false, false, false, false, false, false]
           Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
           hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
           S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
           validator_permits: [true, true, true, true, true, true, true, true]
           max_allowed_validators: 8
           new_validator_permits: [true, true, true, true, true, true, true, true]
           S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
           W: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           W (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           W (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           W (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
           C: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
           W: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
           Tv: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
           R (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
           T: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
           I (=R): [0, 0, 0, 0, 0, 0, 0, 1]
           B: [[(4, 385), (5, 771), (6, 1159), (7, 1546)], [(4, 815), (5, 1631), (6, 2448), (7, 3265)], [(4, 1822), (5, 3648), (6, 5474), (7, 12282)], [(4, 1822), (5, 3648), (6, 5474), (7, 12282)], [], [], [], []]
           B (outdatedmask): [[(4, 385), (5, 771), (6, 1159), (7, 1546)], [(4, 815), (5, 1631), (6, 2448), (7, 3265)], [(4, 1822), (5, 3648), (6, 5474), (7, 12282)], [(4, 1822), (5, 3648), (6, 5474), (7, 12282)], [], [], [], []]
           B: [[(4, 0.0058747234), (5, 0.0117647059), (6, 0.0176852064), (7, 0.0235904478)], [(4, 0.0124361028), (5, 0.0248874647), (6, 0.0373540856), (7, 0.0498207065)], [(4, 0.027801938), (5, 0.0556649119), (6, 0.0835278858), (7, 0.187411307)], [(4, 0.027801938), (5, 0.0556649119), (6, 0.0835278858), (7, 0.187411307)], [], [], [], []]
           alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
           emaB: [[(4, 0.005287251), (5, 0.0105882352), (6, 0.0159166856), (7, 0.0212314029)], [(4, 0.0111924924), (5, 0.0223987182), (6, 0.033618677), (7, 0.0448386357)], [(4, 0.025021744), (5, 0.0500984206), (6, 0.0751750972), (7, 0.2086710306)], [(4, 0.025021744), (5, 0.0500984206), (6, 0.0751750972), (7, 0.2086710306)], [], [], [], []]
           emaB norm: [[(4, 0.0794797675), (5, 0.0795009276), (6, 0.0796289926), (7, 0.043919883)], [(4, 0.16824938), (5, 0.1681790059), (6, 0.1681896253), (7, 0.0927544753)], [(4, 0.3761354259), (5, 0.376160033), (6, 0.3760906907), (7, 0.4316628207)], [(4, 0.3761354259), (5, 0.376160033), (6, 0.3760906907), (7, 0.4316628207)], [], [], [], []]
           total_bonds_per_validator: [0.043919883, 0.0927544753, 0.4316628207, 0.4316628207, 0, 0, 0, 0]
           D: [0.0135093683, 0.0570609153, 0.398327021, 0.531102695, 0, 0, 0, 0]
           nE: [0.006754684, 0.0285304575, 0.1991635105, 0.2655513475, 0, 0, 0, 0.5]
           E: [6754684, 28530457, 199163510, 265551347, 0, 0, 0, 500000000]
           P: [0.006754684, 0.0285304575, 0.1991635105, 0.2655513475, 0, 0, 0, 0.5]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(bonds[0][7], 1391);
        assert_eq!(bonds[1][7], 2938);
        assert_eq!(bonds[2][7], 13675);
        assert_eq!(bonds[3][7], 13675);

        next_block();
        if sparse {
            SubtensorModule::epoch(netuid, 1_000_000_000);
        } else {
            SubtensorModule::epoch_dense(netuid, 1_000_000_000);
        }
        /*  current_block: 8
            current_block: 9, activity_cutoff: 5000, Last update: [2, 4, 5, 2, 1, 1, 1, 1]
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(0, 65535)], [(1, 65535)], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[], [], [(7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0, 0, 0, 0.400008545]
            W: [[], [], [(7, 0.400008545)], [(7, 0.400008545)], [], [], [], []]
            Tv: [0, 0, 0.400008545, 0.400008545, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0, 0, 0, 0.2800059812]
            T: [0, 0, 0, 0, 0, 0, 0, 0.6087041323]
            I (=R): [0, 0, 0, 0, 0, 0, 0, 1]
            B: [[(4, 346), (5, 693), (6, 1043), (7, 1391)], [(4, 733), (5, 1467), (6, 2203), (7, 2938)], [(4, 1639), (5, 3283), (6, 4926), (7, 13675)], [(4, 1639), (5, 3283), (6, 4926), (7, 13675)], [], [], [], []]
            B (outdatedmask): [[(4, 346), (5, 693), (6, 1043), (7, 1391)], [(4, 733), (5, 1467), (6, 2203), (7, 2938)], [(4, 1639), (5, 3283), (6, 4926), (7, 13675)], [(4, 1639), (5, 3283), (6, 4926), (7, 13675)], [], [], [], []]
            B: [[(4, 0.0052796216), (5, 0.0105745022), (6, 0.0159151598), (7, 0.0212252995)], [(4, 0.011184863), (5, 0.0223849851), (6, 0.0336156252), (7, 0.0448310063)], [(4, 0.0250095369), (5, 0.0500953689), (6, 0.0751659418), (7, 0.2086671244)], [(4, 0.0250095369), (5, 0.0500953689), (6, 0.0751659418), (7, 0.2086671244)], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0047516592), (5, 0.0095170517), (6, 0.0143236436), (7, 0.0191027694)], [(4, 0.0100663765), (5, 0.0201464866), (6, 0.0302540625), (7, 0.0403479056)], [(4, 0.022508583), (5, 0.0450858318), (6, 0.0676493475), (7, 0.2278012664)], [(4, 0.022508583), (5, 0.0450858318), (6, 0.0676493475), (7, 0.2278012664)], [], [], [], []]
            emaB norm: [[(4, 0.0794124375), (5, 0.0794178303), (6, 0.079630477), (7, 0.037088924)], [(4, 0.1682350226), (5, 0.1681182678), (6, 0.168193617), (7, 0.0783373541)], [(4, 0.3761762697), (5, 0.3762319507), (6, 0.3760879529), (7, 0.4422868607)], [(4, 0.3761762697), (5, 0.3762319507), (6, 0.3760879529), (7, 0.4422868607)], [], [], [], []]
            total_bonds_per_validator: [0.037088924, 0.0783373541, 0.4422868607, 0.4422868607, 0, 0, 0, 0]
            D: [0.011274011, 0.0476247966, 0.403329082, 0.5377721095, 0, 0, 0, 0]
            nE: [0.0056370054, 0.0238123983, 0.201664541, 0.2688860546, 0, 0, 0, 0.5]
            E: [5637005, 23812398, 201664540, 268886054, 0, 0, 0, 500000000]
            P: [0.0056370054, 0.0238123983, 0.201664541, 0.2688860546, 0, 0, 0, 0.5]
        */
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
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(0, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            W: [[], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Tv: [0, 0.9999999995, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0.0899978022, 0.1800010982, 0.2699934072, 0.36000769]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            B: [[(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [], [], [], []]
            B (outdatedmask): [[(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [(4, 655), (5, 1310), (6, 1966), (7, 2621)], [], [], [], []]
            B: [[(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [(4, 0.0099946593), (5, 0.0199893187), (6, 0.029999237), (7, 0.0399938964)], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0089951933), (5, 0.0179903866), (6, 0.0269993132), (7, 0.0359945067)], [(4, 0.018994949), (5, 0.0379905086), (6, 0.0569985807), (7, 0.0759953612)], [(4, 0.018994949), (5, 0.0379905086), (6, 0.0569985807), (7, 0.0759953612)], [(4, 0.018994949), (5, 0.0379905086), (6, 0.0569985807), (7, 0.0759953612)], [], [], [], []]
            emaB norm: [[(4, 0.1363320365), (5, 0.1363301442), (6, 0.136363573), (7, 0.1363528532)], [(4, 0.287889321), (5, 0.2878899518), (6, 0.2878788088), (7, 0.287882382)], [(4, 0.287889321), (5, 0.2878899518), (6, 0.2878788088), (7, 0.287882382)], [(4, 0.287889321), (5, 0.2878899518), (6, 0.2878788088), (7, 0.287882382)], [], [], [], []]
            total_bonds_per_validator: [0.136349445, 0.2878835173, 0.2878835173, 0.2878835173, 0, 0, 0, 0]
            D: [0.0499942757, 0.211112383, 0.3166685747, 0.422224766, 0, 0, 0, 0]
            nE: [0.0249971377, 0.1055561914, 0.1583342873, 0.211112383, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            E: [24997137, 105556191, 158334287, 211112383, 49998779, 100000610, 149996337, 200004272]
            P: [0.0249971377, 0.1055561914, 0.1583342873, 0.211112383, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */

        // Expected bonds calculations
        // For uid 0:
        // Initial weights: [0.25, 0.5, 0.75, 1.0]
        // Active stake: [1, 2, 3, 4]
        // ΔB = W◦S = [0.25*1, 0.5*2, 0.75*3, 1.0*4] = [0.25, 1.0, 2.25, 4.0]
        // Normalize ΔB: [0.25/7.5, 1.0/7.5, 2.25/7.5, 4.0/7.5] = [0.0333, 0.1333, 0.3, 0.5333]
        // Final bonds for netuid: [16383, 32767, 49151, 65535]

        assert_eq!(bonds[0][4], 1247); // Note: Calculated as explained above
        assert_eq!(bonds[1][4], 1247); // Note: Calculated as explained above
        assert_eq!(bonds[2][4], 1247); // Note: Calculated as explained above
        assert_eq!(bonds[3][4], 1247); // Note: Calculated as explained above

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
        assert_eq!(bonds[0][4], 1009);
        assert_eq!(bonds[1][4], 2257);
        assert_eq!(bonds[2][4], 2257);
        assert_eq!(bonds[3][4], 2257);

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
            Inactive: [false, false, false, false, false, false, false, false]
            Block at registration: [1, 1, 1, 1, 1, 1, 1, 1]
            hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            validator_permits: [true, true, true, true, true, true, true, true]
            max_allowed_validators: 8
            new_validator_permits: [true, true, true, true, true, true, true, true]
            S: [0.0999999999, 0.2, 0.2999999998, 0.4, 0, 0, 0, 0]
            W: [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit): [[(0, 65535)], [(1, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            W (permit+diag+outdate): [[], [], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [(4, 16383), (5, 32767), (6, 49149), (7, 65535)], [], [], [], []]
            C: [0, 0, 0, 0, 0.0999975584, 0.2000012207, 0.2999926754, 0.400008545]
            W: [[], [], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [(4, 0.0999975584), (5, 0.2000012207), (6, 0.2999926754), (7, 0.400008545)], [], [], [], []]
            Tv: [0, 0, 0.9999999995, 0.9999999995, 0, 0, 0, 0]
            R (after): [0, 0, 0, 0, 0.0699982906, 0.1400008542, 0.2099948723, 0.2800059812]
            T: [0, 0, 0, 0, 1, 1, 1, 1]
            I (=R): [0, 0, 0, 0, 0.0999975582, 0.2000012207, 0.2999926754, 0.4000085455]
            B: [[(4, 589), (5, 1178), (6, 1769), (7, 2358)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [], [], [], []]
            B (outdatedmask): [[(4, 589), (5, 1178), (6, 1769), (7, 2358)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [(4, 1244), (5, 2489), (6, 3735), (7, 4980)], [], [], [], []]
            B: [[(4, 0.008987564), (5, 0.0179751278), (6, 0.0269932097), (7, 0.0359807736)], [(4, 0.0189822232), (5, 0.0379797055), (6, 0.0569924468), (7, 0.075989929)], [(4, 0.0189822232), (5, 0.0379797055), (6, 0.0569924468), (7, 0.075989929)], [(4, 0.0189822232), (5, 0.0379797055), (6, 0.0569924468), (7, 0.075989929)], [], [], [], []]
            alphas: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
            emaB: [[(4, 0.0080888073), (5, 0.016177615), (6, 0.0242938886), (7, 0.0323826962)], [(4, 0.0170840009), (5, 0.0341817348), (6, 0.051293202), (7, 0.068390936)], [(4, 0.0270837566), (5, 0.0541818568), (6, 0.0812924695), (7, 0.1083917904)], [(4, 0.0270837566), (5, 0.0541818568), (6, 0.0812924695), (7, 0.1083917904)], [], [], [], []]
            emaB norm: [[(4, 0.1019507758), (5, 0.10192353), (6, 0.102001434), (7, 0.101974368)], [(4, 0.2153255814), (5, 0.2153545555), (6, 0.2153619886), (7, 0.215365714)], [(4, 0.3413618212), (5, 0.341360957), (6, 0.3413182884), (7, 0.3413299588)], [(4, 0.3413618212), (5, 0.341360957), (6, 0.3413182884), (7, 0.3413299588)], [], [], [], []]
            total_bonds_per_validator: [0.1019699604, 0.215358351, 0.3413358429, 0.3413358429, 0, 0, 0, 0]
            D: [0.034896868, 0.1474028623, 0.3504429725, 0.4672572967, 0, 0, 0, 0]
            nE: [0.017448434, 0.073701431, 0.1752214862, 0.2336286483, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
            E: [17448433, 73701431, 175221486, 233628648, 49998779, 100000610, 149996337, 200004272]
            P: [0.017448434, 0.073701431, 0.1752214862, 0.2336286483, 0.049998779, 0.1000006103, 0.1499963377, 0.2000042726]
        */

        assert_eq!(bonds[0][4], 816);
        assert_eq!(bonds[1][4], 1827);
        assert_eq!(bonds[2][4], 3075);
        assert_eq!(bonds[3][4], 3075);
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
        /*  current_block: 5002; activity_cutoff: 5000
                Last update: [5002, 1, 0, 0]; Inactive: [false, true, true, true]; Block at registration: [0, 0, 0, 0]
                Normalised Stake: [0.25, 0.25, 0.25, 0.25]
                validator_permits: [true, true, true, true]
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
                Bonds (outdatedmask): [[(2, 3276), (3, 3276)], [(2, 3276), (3, 3276)], [], []]
                Bonds: (mask+norm) [[(2, 0.0499885557), (3, 0.0499885557)], [(2, 0.0499885557), (3, 0.0499885557)], [], []]
                Alphas: [0.1, 0.1, 0.1, 0.1]
                weights_for_bonds: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
                emaB: [[(2, 0.0949897), (3, 0.0949897)], [(2, 0.0949897), (3, 0.0949897)], [], []]
                total_bonds_per_validator: [0.5, 0.5, 0, 0]
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
            assert_eq!(bonds[0][server], I32F32::from_num(6225));
        }
        for validator in 1..(n / 2) {
            assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, validator), 0);
            assert_eq!(SubtensorModule::get_emission_for_uid(netuid, validator), 0);
            for server in ((n / 2) as usize)..n as usize {
                assert_eq!(bonds[validator as usize][server], I32F32::from_num(6225));
                // floor(0.45*(2^16-1))/(2^16-1), then max-upscale
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
        /*  current_block: 5003; activity_cutoff: 5000
        Last update: [5002, 5002, 0, 0]; Inactive: [false, false, true, true]; Block at registration: [0, 0, 0, 0]
        Inactive: [false, false, true, true]
        Block at registration: [0, 0, 0, 0]
        hotkeys: [(0, 0), (1, 1), (2, 2), (3, 3)]
        Normalised Stake: [0.25, 0.25, 0.25, 0.25]
        validator_permits: [true, true, true, true]
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
        Bonds: [[(2, 6225), (3, 6225)], [(2, 6225), (3, 6225)], [], []]
        Bonds (outdatedmask): [[(2, 6225), (3, 6225)], [(2, 6225), (3, 6225)], [], []]
        Bonds: (mask+norm) [[(2, 0.0949874113), (3, 0.0949874113)], [(2, 0.0949874113), (3, 0.0949874113)], [], []]
        Alphas: [0.1, 0.1, 0.1, 0.1]
        weights_for_bonds: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        emaB: [[(2, 0.13548867), (3, 0.13548867)], [(2, 0.13548867), (3, 0.13548867)], [], []]
        total_bonds_per_validator: [0.5, 0.5, 0, 0]
        Dividends: [0.5, 0.5, 0, 0]
        Normalized Server Emission: [0, 0, 0.25, 0.25]
        Server Emission: [0, 0, 250000000, 250000000]
        Normalized Validator Emission: [0.25, 0.25, 0, 0]
        Validator Emission: [250000000, 250000000, 0, 0]
        Normalized Combined Emission: [0.25, 0.25, 0.25, 0.25]
        Combined Emission: [250000000, 250000000, 250000000, 250000000]
        Pruning Scores: [0.25, 0.25, 0.25, 0.25]
        */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 32767);
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 250000000);
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[0][server], I32F32::from_num(8879));
        }
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 1), 32767);
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 1), 250000000);
        for server in ((n / 2) as usize)..n as usize {
            assert_eq!(bonds[1][server], I32F32::from_num(8879));
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
        /*  current_block: 1; activity_cutoff: 5000
        Last update: [1, 1, 1, 1]; Inactive: [false, false, false, false]; Block at registration: [0, 0, 0, 0]
        S: [0.25, 0.25, 0.25, 0.25]; S (mask): [0.25, 0.25, 0.25, 0.25]; S (mask+norm): [0.25, 0.25, 0.25, 0.25]
        validator_permits: [true, true, true, true]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
        W: [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit+diag): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
        W (permit+diag+outdate): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
        W (mask+norm): [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 0.6666632756), (3, 0.3333367242)], [], []]
        R (before): [0, 0, 0.3333316376, 0.166668362]
        C: [0, 0, 0.6666632756, 0.3333367242]
        W: [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 0.6666632756), (3, 0.3333367242)], [], []]
        Tv: [0.9999999998, 0.9999999998, 0, 0]
        R (after): [0, 0, 0.3333316376, 0.166668362]
        T: [0, 0, 1, 1]
        I (=R): [0, 0, 0.6666632756, 0.3333367242]
        B: [[], [], [], []]
        B (outdatedmask): [[], [], [], []]
        B (mask+norm): [[], [], [], []]
        ΔB: [[(2, 0.1666658188), (3, 0.083334181)], [(2, 0.1666658188), (3, 0.083334181)], [], []]
        ΔB (norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        emaB: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
        D: [0.5, 0.5, 0, 0]
        nE: [0.25, 0.25, 0.3333316378, 0.166668362]
        E: [250000000, 250000000, 333331637, 166668361]
        P: [0.25, 0.25, 0.3333316378, 0.166668362]
        P (u16): [49151, 49151, 65535, 32767] */

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
        /*  current_block: 2; activity_cutoff: 5000
        Last update: [2, 1, 1, 1]; Inactive: [false, false, false, false]; Block at registration: [0, 0, 0, 1]
        S: [0.3333333333, 0.3333333333, 0.3333333333, 0]
        S (mask): [0.3333333333, 0.3333333333, 0.3333333333, 0]
        S (mask+norm): [0.3333333333, 0.3333333333, 0.3333333333, 0]
        validator_permits: [true, true, true, false]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
        W: [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [(2, 65535)], [(3, 65535)]]
        W (permit+diag): [[(2, 65535), (3, 32768)], [(2, 65535), (3, 32768)], [], []]
        W (permit+diag+outdate): [[(2, 65535), (3, 32768)], [(2, 65535)], [], []]
        W (mask+norm): [[(2, 0.6666632756), (3, 0.3333367242)], [(2, 1)], [], []]
        R (before): [0, 0, 0.5555544249, 0.1111122412]
        C: [0, 0, 0.6666632756, 0]
        W: [[(2, 0.6666632756)], [(2, 0.6666632756)], [], []]
        Tv: [0.6666632756, 0.6666632756, 0, 0]
        R (after): [0, 0, 0.4444421832, 0]
        T: [0, 0, 0.799997558, 0]
        I (=R): [0, 0, 1, 0]
        B: [[(2, 65535), (3, 65535)], [(2, 65535), (3, 65535)], [], []]
        B (outdatedmask): [[(2, 65535), (3, 65535)], [(2, 65535)], [], []]
        B (mask+norm): [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
        ΔB: [[(2, 0.2222210916)], [(2, 0.2222210916)], [], []]
        ΔB (norm): [[(2, 0.5)], [(2, 0.5)], [], []]
        emaB: [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
        emaB (max-upscale): [[(2, 1), (3, 1)], [(2, 1)], [], []]
        D: [0.5, 0.5, 0, 0]
        nE: [0.25, 0.25, 0.5, 0]
        E: [250000000, 250000000, 500000000, 0]
        P: [0.25, 0.25, 0.5, 0]
        P (u16): [32767, 32767, 65535, 0] */
        let bonds = SubtensorModule::get_bonds(netuid);
        assert_eq!(SubtensorModule::get_dividends_for_uid(netuid, 0), 32767); // Note D = floor(0.5 * 65_535)
        assert_eq!(SubtensorModule::get_emission_for_uid(netuid, 0), 250000000); // Note E = 0.5 * 0.5 * 1_000_000_000 = 249311245
        assert_eq!(bonds[0][2], I32F32::from_num(8300));
        assert_eq!(bonds[0][3], I32F32::from_num(1965));
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
        B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []];
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
        B: [[], []]: B (outdatedmask): [[], []]; B (mask+norm): [[], []]
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
        B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []];
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
        B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []];
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

/// Helper function to assert approximate equality of two vectors of vectors of tuples.
fn assert_approx_eq_vec_of_vec(
    left: &[Vec<(u16, I32F32)>],
    right: &[Vec<(u16, I32F32)>],
    epsilon: I32F32,
) {
    assert_eq!(left.len(), right.len(), "Vectors have different lengths");
    for (left_row, right_row) in left.iter().zip(right.iter()) {
        assert_eq!(
            left_row.len(),
            right_row.len(),
            "Rows have different lengths"
        );
        for ((left_idx, left_val), (right_idx, right_val)) in left_row.iter().zip(right_row.iter())
        {
            assert_eq!(left_idx, right_idx, "Indices are different");
            assert!(
                (left_val - right_val).abs() < epsilon,
                "Values are different: left = {:?}, right = {:?}, epsilon = {:?}",
                left_val,
                right_val,
                epsilon
            );
        }
    }
}

// test Yuma 4 scenarios over a sequence of epochs.
fn setup_yuma_4_scenario(netuid: u16, n: u16, sparse: bool, max_stake: u64, stakes: Vec<u64>) {
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
    // SubtensorModule::set_bonds_moving_average(netuid, 975_000);

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
    SubtensorModule::set_kappa(netuid, u16::MAX / 2);

    SubtensorModule::set_alpha_values_32(netuid, I32F32::from_num(0.1), I32F32::from_num(0.3));

    // === Issue validator permits
    SubtensorModule::set_max_allowed_validators(netuid, 3);
    assert_eq!(SubtensorModule::get_max_allowed_validators(netuid), 3);

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
    let mut bonds = SubtensorModule::get_bonds(netuid);
    bonds = mat_fixed_proportions_to_fixed(bonds.clone());
    let dividends = SubtensorModule::get_dividends(netuid);

    let epsilon = I32F32::from_num(1e-3);
    // Check the bonds
    // server 1
    assert_approx_eq(bonds[0][3], fixed(target_bonds[0][0]), epsilon);
    assert_approx_eq(bonds[1][3], fixed(target_bonds[1][0]), epsilon);
    assert_approx_eq(bonds[2][3], fixed(target_bonds[2][0]), epsilon);
    // server 2
    assert_approx_eq(bonds[0][4], fixed(target_bonds[0][1]), epsilon);
    assert_approx_eq(bonds[1][4], fixed(target_bonds[1][1]), epsilon);
    assert_approx_eq(bonds[2][4], fixed(target_bonds[2][1]), epsilon);

    // Check the dividends
    for (dividend, target_dividend) in dividends.iter().zip(target_dividends.iter()) {
        assert_approx_eq(
            u16_proportion_to_fixed(*dividend),
            fixed(*target_dividend),
            epsilon,
        );
    }
}

fn set_yuma_4_weights(netuid: u16, weights: Vec<Vec<u16>>) {
    for (uid, weight) in weights.iter().enumerate() {
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(U256::from(uid as u64)),
            netuid,
            vec![3, 4],
            weight.to_vec(),
            0
        ));
    }
}

#[test]
fn test_yuma_4_kappa_moves_first() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 5; // 3 validators, 2 servers
        let netuid: u16 = 1;
        let max_stake: u64 = 8;

        // Validator A: kappa / Big validator (0.8) - moves first
        // Validator B: Small eager validator (0.1) - moves second
        // Validator C: Small lazy validator (0.1) - moves last
        let stakes: Vec<u64> = vec![8, 1, 1, 0, 0];

        setup_yuma_4_scenario(netuid, n, sparse, max_stake, stakes);
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
                    set_yuma_4_weights(netuid, vec![vec![u16::MAX, 0]; 3]);
                }
                1 => {
                    // Validator A -> Server 2
                    // Validator B -> Server 1
                    // Validator C -> Server 1
                    set_yuma_4_weights(
                        netuid,
                        vec![vec![0, u16::MAX], vec![u16::MAX, 0], vec![u16::MAX, 0]],
                    );
                }
                2 => {
                    // Validator A -> Server 2
                    // Validator B -> Server 2
                    // Validator C -> Server 1
                    set_yuma_4_weights(
                        netuid,
                        vec![vec![0, u16::MAX], vec![0, u16::MAX], vec![u16::MAX, 0]],
                    );
                }
                3 => {
                    // Subsequent epochs All validators -> Server 2
                    set_yuma_4_weights(netuid, vec![vec![0, u16::MAX]; 3]);
                }
                _ => {}
            };
            run_epoch_and_check_bonds_dividends(netuid, sparse, target_bonds, target_dividends);
        }
    })
}

#[test]
fn test_yuma_4_kappa_moves_second() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = false;
        let n: u16 = 5; // 3 validators, 2 servers
        let netuid: u16 = 1;
        let max_stake: u64 = 8;

        // Validator A: kappa / Big validator (0.8) - moves second
        // Validator B: Small eager validator (0.1) - moves first
        // Validator C: Small lazy validator (0.1) - moves last
        let stakes: Vec<u64> = vec![8, 1, 1, 0, 0];

        setup_yuma_4_scenario(netuid, n, sparse, max_stake, stakes);
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
                    set_yuma_4_weights(netuid, vec![vec![u16::MAX, 0]; 3]);
                }
                1 => {
                    // Validator A -> Server 1
                    // Validator B -> Server 2
                    // Validator C -> Server 1
                    set_yuma_4_weights(
                        netuid,
                        vec![vec![u16::MAX, 0], vec![0, u16::MAX], vec![u16::MAX, 0]],
                    );
                }
                2 => {
                    // Validator A -> Server 2
                    // Validator B -> Server 2
                    // Validator C -> Server 1
                    set_yuma_4_weights(
                        netuid,
                        vec![vec![0, u16::MAX], vec![0, u16::MAX], vec![u16::MAX, 0]],
                    );
                }
                3 => {
                    // Subsequent epochs All validators -> Server 2
                    set_yuma_4_weights(netuid, vec![vec![0, u16::MAX]; 3]);
                }
                _ => {}
            };
            run_epoch_and_check_bonds_dividends(netuid, sparse, target_bonds, target_dividends);
        }
    })
}

#[test]
fn test_yuma_4_kappa_moves_last() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 5; // 3 validators, 2 servers
        let netuid: u16 = 1;
        let max_stake: u64 = 8;

        // Validator A: kappa / Big validator (0.8) - moves last
        // Validator B: Small eager validator (0.1) - moves first
        // Validator C: Small lazy validator (0.1) - moves second
        let stakes: Vec<u64> = vec![8, 1, 1, 0, 0];

        setup_yuma_4_scenario(netuid, n, sparse, max_stake, stakes);
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
                    set_yuma_4_weights(netuid, vec![vec![u16::MAX, 0]; 3]);
                }
                1 => {
                    // Validator A -> Server 1
                    // Validator B -> Server 2
                    // Validator C -> Server 1
                    set_yuma_4_weights(
                        netuid,
                        vec![vec![u16::MAX, 0], vec![0, u16::MAX], vec![u16::MAX, 0]],
                    );
                }
                2 => {
                    // Validator A -> Server 1
                    // Validator B -> Server 2
                    // Validator C -> Server 2
                    set_yuma_4_weights(
                        netuid,
                        vec![vec![u16::MAX, 0], vec![0, u16::MAX], vec![0, u16::MAX]],
                    );
                }
                3 => {
                    // Subsequent epochs All validators -> Server 2
                    set_yuma_4_weights(netuid, vec![vec![0, u16::MAX]; 3]);
                }
                _ => {}
            };
            run_epoch_and_check_bonds_dividends(netuid, sparse, target_bonds, target_dividends);
        }
    })
}

#[test]
fn test_yuma_4_one_epoch_switch() {
    new_test_ext(1).execute_with(|| {
        let sparse: bool = true;
        let n: u16 = 5; // 3 validators, 2 servers
        let netuid: u16 = 1;
        let max_stake: u64 = 8;

        // Equal stake validators
        let stakes: Vec<u64> = vec![33, 33, 34, 0, 0];

        setup_yuma_4_scenario(netuid, n, sparse, max_stake, stakes);

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
                    set_yuma_4_weights(
                        netuid,
                        vec![vec![u16::MAX, 0], vec![u16::MAX, 0], vec![0, u16::MAX]],
                    );
                }
                _ => {
                    // All validators -> Server 1
                    set_yuma_4_weights(netuid, vec![vec![u16::MAX, 0]; 3]);
                }
            };
            run_epoch_and_check_bonds_dividends(netuid, sparse, target_bonds, target_dividends);
        }
    })
}
