use crate::mock::*;
use rand::{Rng, thread_rng, SeedableRng, rngs::StdRng, seq::SliceRandom, distributions::Uniform};
use substrate_fixed::types::{I32F32, I64F64};
use substrate_fixed::transcendental::{PI, cos, ln, sqrt};
use frame_system::Config;
use frame_support::assert_ok;
use std::time::Instant;
mod mock;

pub fn fixed(val: f32) -> I32F32 { I32F32::from_num(val) }

pub fn fixed_to_u16( x: I32F32 ) -> u16 { x.to_num::<u16>() }

pub fn fixed_proportion_to_u16( x: I32F32 ) -> u16 { fixed_to_u16( x * I32F32::from_num( u16::MAX )) }

// Normalizes (sum to 1 except 0) the input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize( x: &mut Vec<I32F32> ) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num( 0.0 as f32 ){ return }
    for i in 0..x.len() {
        x[i] = x[i]/x_sum;
    }
}

// Inplace normalize the passed positive integer weights so that they sum to u16 max value.
fn normalize_weights(mut weights: Vec<u16>) -> Vec<u16> {
	let sum: u64 = weights.iter().map(|x| *x as u64).sum();
	if sum == 0 { return weights; }
	weights.iter_mut().for_each(|x| { *x = (*x as u64 * u16::max_value() as u64 / sum) as u16; });
	return weights;
}

// Return as usize an I32F32 ratio of a usize input, avoiding the 0% and 100% extremes.
fn non_extreme_fixed_ratio(ratio: I32F32, total: usize) -> usize {
	if total == 0 { return total }
	let mut subset: usize = (ratio * I32F32::from_num(total)).to_num::<usize>();
	if subset == 0 {
		subset = 1;
	}
	else if subset == total {
		subset = total - 1;
	}
	return subset
}

// Box-Muller Transform converting two uniform random samples to a normal random sample.
fn normal(size: usize, rng: &mut StdRng, dist: &Uniform<u16>) -> Vec<I32F32> {
	let max: I32F32 = I32F32::from_num(u16::MAX);
	let two: I32F32 = I32F32::from_num(2);
	let eps: I32F32 = I32F32::from_num(0.000001);
	let pi: I32F32 = I32F32::from_num(PI);

	let uniform_u16: Vec<u16> = (0..(2*size)).map(|_| rng.sample(&dist)).collect();
	let uniform: Vec<I32F32> = uniform_u16.iter().map(|&x| I32F32::from_num(x) / max).collect();
	let mut normal: Vec<I32F32> = vec![ I32F32::from_num(0); size as usize];
	
	for i in 0..size {
		let u1: I32F32 = uniform[i] + eps;
		let u2: I32F32 = uniform[i + size] + eps;
		normal[i] = sqrt::<I32F32, I32F32>(-two * ln::<I32F32, I32F32>(u1).expect("")).expect("") * cos(two * pi * u2);
	}
	normal
}

// Returns validators and servers uids with either blockwise, regular, or random interleaving.
fn distribute_nodes(validators_n: usize, network_n: usize, interleave: usize) -> (Vec<u16>, Vec<u16>) {
	let mut validators: Vec<u16> = vec![];
	let mut servers: Vec<u16> = vec![];
	
	if interleave == 0 { // blockwise [validator_block, server_block]
		validators = (0..validators_n as u16).collect();
		servers = (validators_n as u16..network_n as u16 ).collect();
	}
	else if interleave == 1 { // regular interleaving [val, srv, srv, ..., srv, val, srv, srv, ..., srv, val, srv, ..., srv]
		(validators, servers) = (0..network_n as u16).collect::<Vec<u16>>().iter().partition( | &i | *i as usize % (network_n / validators_n) == 0);
	}
	else if interleave == 2 { // random interleaving
		let mut permuted_uids: Vec<u16> = (0..network_n as u16).collect();
		permuted_uids.shuffle(&mut thread_rng());
		validators = permuted_uids[0..validators_n as usize].into();
		servers = permuted_uids[validators_n as usize..network_n as usize].into();
	}

	return (validators, servers);
}

#[allow(dead_code)]
fn uid_stats(netuid: u16, uid: u16) {
	log::info!( "stake: {:?}", SubtensorModule::get_total_stake_for_hotkey( &(uid as u64) ) );
	log::info!( "rank: {:?}", SubtensorModule::get_rank_for_uid( netuid, uid ) );
	log::info!( "trust: {:?}", SubtensorModule::get_trust_for_uid( netuid, uid ) );
	log::info!( "consensus: {:?}", SubtensorModule::get_consensus_for_uid( netuid, uid ) );
	log::info!( "incentive: {:?}", SubtensorModule::get_incentive_for_uid( netuid, uid ) );
	log::info!( "dividend: {:?}", SubtensorModule::get_dividends_for_uid( netuid, uid ) );
	log::info!( "emission: {:?}", SubtensorModule::get_emission_for_uid( netuid, uid ) );
}

fn init_run_epochs(netuid: u16, n: u16, validators: &Vec<u16>, servers: &Vec<u16>, epochs: u16, stake_per_validator: u64, server_self: bool, input_stake: &Vec<u64>, use_input_stake: bool, input_weights: &Vec<Vec<(u16, u16)>>, use_input_weights: bool, random_weights: bool, random_seed: u64, sparse: bool) {
	// === Create the network
	add_network(netuid, u16::MAX - 1, 0);  // set higher tempo to avoid built-in epoch, then manual epoch instead

	// === Register uids
	SubtensorModule::set_max_allowed_uids( netuid, n );
	for key in 0..n {
		let stake: u64;
		if use_input_stake {
			stake = input_stake[key as usize];
		}
		else {
			stake = if validators.contains(&key) { stake_per_validator } else { 0 }; // only validators receive stake
		}
		// let stake: u64 = 1; // alternative test: all nodes receive stake, should be same outcome, except stake
		SubtensorModule::add_balance_to_coldkey_account( &(key as u64), stake );
		SubtensorModule::append_neuron( netuid, &(key as u64), 0 );
		SubtensorModule::increase_stake_on_coldkey_hotkey_account( &(key as u64), &(key as u64), stake as u64 );
	}
	assert_eq!( SubtensorModule::get_subnetwork_n(netuid), n );

	// === Issue validator permits
	assert_ok!( SubtensorModule::sudo_set_max_allowed_validators(<<Test as Config>::RuntimeOrigin>::root(), netuid, validators.len() as u16) );
    assert_eq!( SubtensorModule::get_max_allowed_validators(netuid), validators.len() as u16);
	SubtensorModule::epoch( netuid, 1_000_000_000 ); // run first epoch to set allowed validators
	run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block

	// === Set weights
	let mut rng = StdRng::seed_from_u64(random_seed); // constant seed so weights over multiple runs are equal
    let range = Uniform::new(0, u16::MAX);
	let mut weights: Vec<u16> = vec![ u16::MAX / n; servers.len() as usize ];
	for uid in validators {
		if random_weights {
			weights = (0..servers.len()).map(|_| rng.sample(&range)).collect();
			weights = normalize_weights(weights);
			// assert_eq!(weights.iter().map(|x| *x as u64).sum::<u64>(), u16::MAX as u64); // normalized weight sum not always u16::MAX
		}
		if use_input_weights {
			let sparse_weights = input_weights[*uid as usize].clone();
			weights = sparse_weights.iter().map(|(_, w)| *w).collect();
			let srvs: Vec<u16> = sparse_weights.iter().map(|(s, _)| *s).collect();
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(*uid as u64), netuid, srvs, weights.clone(), 0));
		}
		else {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(*uid as u64), netuid, servers.clone(), weights.clone(), 0));
		}
	}
	if server_self {
		for uid in servers {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(*uid as u64), netuid, vec![ *uid as u16 ], vec![ u16::MAX ], 0)); // server self-weight
		}
	}

	// === Run the epochs.
	log::info!( "Start {epochs} epoch(s)" );
	let start = Instant::now();
	for _ in 0..epochs {
		if sparse {
			SubtensorModule::epoch( netuid, 1_000_000_000 );
		}
		else {
			SubtensorModule::epoch_dense( netuid, 1_000_000_000 );
		}
	}
	let duration = start.elapsed();
	log::info!( "Time elapsed in (sparse={sparse}) epoch() is: {:?}", duration );

	// let bonds = SubtensorModule::get_bonds( netuid );
	// for (uid, node) in vec![ (validators[0], "validator"), (servers[0], "server") ] {
	// 	log::info!("\n{node}" );
	// 	uid_stats(netuid, uid);
	// 	log::info!("bonds: {:?} (on validator), {:?} (on server)", bonds[uid as usize][0], bonds[uid as usize][servers[0] as usize]);
	// }
}

// Generate a random graph that is split into a major and minor set, each setting specific weight on itself and the complement on the other.
fn split_graph(major_stake: I32F32, major_weight: I32F32, minor_weight: I32F32, weight_stddev_major: I32F32, weight_stddev_minor: I32F32, validators_n: usize, network_n: usize, interleave: usize) -> (Vec<u16>, Vec<u16>, Vec<u16>, Vec<u16>, Vec<u16>, Vec<u16>, Vec<u64>, Vec<Vec<(u16, u16)>>, I32F32) {
	let servers_n: usize = network_n - validators_n;
	let major_servers_n: usize = non_extreme_fixed_ratio(major_stake, servers_n);
	let major_validators_n: usize = non_extreme_fixed_ratio(major_stake, validators_n);

	let (validators, servers) = distribute_nodes(validators_n, network_n, interleave as usize);
	let major_validators: Vec<u16> = (0..major_validators_n).map(|i| validators[i]).collect();
	let minor_validators: Vec<u16> = (major_validators_n..validators_n).map(|i| validators[i]).collect();
	let major_servers: Vec<u16> = (0..major_servers_n).map(|i| servers[i]).collect();
	let minor_servers: Vec<u16> = (major_servers_n..servers_n).map(|i| servers[i]).collect();

	let zero: I32F32 = I32F32::from_num(0);
	let one: I32F32 = I32F32::from_num(1);
	let stddev: I32F32 = I32F32::from_num(0.3);
	let total_stake: I64F64 = I64F64::from_num(21_000_000_000_000_000 as u64);
	let mut rng = StdRng::seed_from_u64(0); // constant seed so weights over multiple runs are equal
    let dist = Uniform::new(0, u16::MAX);

	let mut stake: Vec<u64> = vec![0; network_n];
	let mut stake_fixed: Vec<I32F32> = vec![zero; network_n];
	for (ratio, vals) in vec![(major_stake, &major_validators), (one - major_stake, &minor_validators)] {
		let mut sample = normal(vals.len(), &mut rng, &dist).iter().map(|x: &I32F32| { let v: I32F32 = (stddev * x) + one; if v < zero {zero} else {v} }).collect();
		inplace_normalize(&mut sample);
		for (i, &val) in vals.iter().enumerate() {
			stake[val as usize] = ( I64F64::from_num(ratio) * I64F64::from_num(sample[i]) * total_stake ).to_num::<u64>();
			stake_fixed[val as usize] = I32F32::from_num(I64F64::from_num(ratio) * I64F64::from_num(sample[i]));
		}
	}
	
	let mut weights: Vec<Vec<(u16, u16)>> = vec![ vec![]; network_n as usize ];
	let mut weights_fixed: Vec<Vec<I32F32>> = vec![ vec![zero; network_n]; network_n ];
	for (first, second, weight_stddev,  vals) in vec![(major_weight, one - major_weight, weight_stddev_major, &major_validators), (one - minor_weight, minor_weight, weight_stddev_minor, &minor_validators)] {
		for &val in vals {
			for (weight, srvs) in vec![(first, &major_servers), (second, &minor_servers)] {
				let mut sample: Vec<I32F32> = normal(srvs.len(), &mut rng, &dist).iter().map(|x: &I32F32| { let v: I32F32 = (weight_stddev * x) + one; if v < zero {zero} else {v} }).collect();
				inplace_normalize(&mut sample);

				for (i, &srv) in srvs.iter().enumerate() {
					weights[val as usize].push( (srv, fixed_proportion_to_u16(weight * sample[i])) );
					weights_fixed[val as usize][srv as usize] = weight * sample[i];
				}
			}
			inplace_normalize(&mut weights_fixed[val as usize]);
		}
	}

	inplace_normalize(&mut stake_fixed);

	// Calculate stake-weighted mean per server
	let mut weight_mean: Vec<I32F32> = vec![ zero; network_n ];
	for val in 0..network_n {
		if stake_fixed[val] > zero {
			for srv in 0..network_n {
				weight_mean[srv] += stake_fixed[val] * weights_fixed[val][srv];
			}
		}
	}

	// Calculate stake-weighted absolute standard deviation
	let mut weight_dev: Vec<I32F32> = vec![ zero; network_n ];
	for val in 0..network_n {
		if stake_fixed[val] > zero {
			for srv in 0..network_n {
				weight_dev[srv] += stake_fixed[val] * (weight_mean[srv] - weights_fixed[val][srv]).abs();
			}
		}
	}

	// Calculate rank-weighted mean of weight_dev
	let avg_weight_dev: I32F32 = weight_dev.iter().sum::<I32F32>() / weight_mean.iter().sum::<I32F32>();
	
	(validators, servers, major_validators, minor_validators, major_servers, minor_servers, stake, weights, avg_weight_dev)
}

// Test consensus guarantees with an epoch on a graph with 4096 nodes, of which the first 128 are validators, the graph is split into a major and minor set, each setting specific weight on itself and the complement on the other. Asserts that the major emission ratio >= major stake ratio.
#[test]
fn test_consensus_guarantees() {
	let netuid: u16 = 0;
	let network_n: u16 = 512;
	let validators_n: u16 = 64;
	let epochs: u16 = 1;
	let interleave = 2;
	log::info!( "test_consensus_guarantees ({network_n:?}, {validators_n:?} validators)" );
	for (major_stake, major_weight, minor_weight, weight_stddev) in vec![(0.51, 1., 1., 0.001), (0.51, 0.03, 0., 0.001), (0.51, 0.51, 0.49, 0.001), (0.51, 0.51, 1., 0.001), (0.51, 0.61, 0.8, 0.1), (0.6, 0.67, 0.65, 0.2), (0.6, 0.74, 0.77, 0.4), (0.6, 0.76, 0.8, 0.4), (0.6, 0.76, 1., 0.4), (0.6, 0.92, 1., 0.4), (0.6, 0.94, 1., 0.4), (0.65, 0.78, 0.85, 0.6), (0.7, 0.81, 0.85, 0.8), (0.7, 0.83, 0.85, 1.)] {
		let (validators, servers, major_validators, minor_validators, major_servers, minor_servers, stake, weights, _avg_weight_dev) = split_graph(fixed(major_stake), fixed(major_weight), fixed(minor_weight), fixed(weight_stddev), fixed(weight_stddev), validators_n as usize, network_n as usize, interleave as usize);

		new_test_ext().execute_with(|| {
			init_run_epochs(netuid, network_n, &validators, &servers, epochs, 1, true, &stake, true, &weights, true, false, 0, false);

			let mut major_emission: I64F64 = I64F64::from_num(0);
			let mut minor_emission: I64F64 = I64F64::from_num(0);
			for set in vec![major_validators, major_servers] {
				for uid in set {
					major_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
				}
			}
			for set in vec![minor_validators, minor_servers] {
				for uid in set {
					minor_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
				}
			}
			let major_ratio: I32F32 = I32F32::from_num(major_emission / (major_emission + minor_emission));
			assert!(major_stake <= major_ratio);
		});
	}
}

// Test an epoch on an empty graph.
#[test]
fn test_overflow() {
	new_test_ext().execute_with(|| {
        log::info!( "test_overflow:" );
		let netuid: u16 = 0;
		add_network(netuid, 0, 0); 
		SubtensorModule::set_max_allowed_uids( netuid, 3 ); 
		SubtensorModule::increase_stake_on_coldkey_hotkey_account( &0, &0, 10 );
		SubtensorModule::increase_stake_on_coldkey_hotkey_account( &1, &1, 10 );
		SubtensorModule::increase_stake_on_coldkey_hotkey_account( &2, &2, 10 );
		SubtensorModule::append_neuron( netuid, &0, 0 );
		SubtensorModule::append_neuron( netuid, &1, 0 );
		SubtensorModule::append_neuron( netuid, &2, 0 );
		SubtensorModule::set_validator_permit_for_uid(0, 0, true);
		SubtensorModule::set_validator_permit_for_uid(0, 1, true);
		SubtensorModule::set_validator_permit_for_uid(0, 2, true);
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(0), netuid, vec![ 0, 1, 2 ], vec![ u16::MAX/3, u16::MAX/3, u16::MAX ], 0));
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(1), netuid, vec![ 1, 2 ], vec![ u16::MAX/2, u16::MAX/2 ], 0));
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(2), netuid, vec![ 2 ], vec![ u16::MAX ], 0));
		SubtensorModule::epoch( 0, u64::MAX );
	});
}

// Test an epoch on an empty graph.
#[test]
fn test_nill_epoch_subtensor() {
	new_test_ext().execute_with(|| {
        log::info!( "test_nill_epoch:" );
		SubtensorModule::epoch( 0, 0 );
	});
}

// Test an epoch on a graph with a single item.
#[test]
fn test_1_graph() {
	new_test_ext().execute_with(|| {
    	log::info!( "test_1_graph:" );
		let netuid: u16 = 0;
		let coldkey: u64 = 0;
		let hotkey: u64 = 0;
		let uid: u16 = 0;
		let stake_amount: u64 = 1;
		add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead
		SubtensorModule::set_max_allowed_uids( netuid, 1 ); 
		SubtensorModule::add_balance_to_coldkey_account( &coldkey, stake_amount );
 		SubtensorModule::increase_stake_on_coldkey_hotkey_account( &coldkey, &hotkey, stake_amount );
		 SubtensorModule::append_neuron( netuid, &hotkey, 0 );
		assert_eq!( SubtensorModule::get_subnetwork_n(netuid), 1 );
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(uid as u64), netuid, vec![ uid as u16 ], vec![ u16::MAX ], 0));
		// SubtensorModule::set_weights_for_testing( netuid, i as u16, vec![ ( 0, u16::MAX )]); // doesn't set update status
		// SubtensorModule::set_bonds_for_testing( netuid, uid, vec![ ( 0, u16::MAX )]); // rather, bonds are calculated in epoch
		SubtensorModule::epoch( 0, 1_000_000_000 );
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey ), stake_amount );
		assert_eq!( SubtensorModule::get_rank_for_uid( netuid, uid ), 0 );
		assert_eq!( SubtensorModule::get_trust_for_uid( netuid, uid ), 0 );
		assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, uid ), 0 );
		assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, uid ), 0 );
		assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, uid ), 0 );
		assert_eq!( SubtensorModule::get_emission_for_uid( netuid, uid ), 1_000_000_000 );
	});
}

// Test an epoch on a graph with two items.
#[test]
fn test_10_graph() {
	new_test_ext().execute_with(|| {
    	log::info!("test_10_graph" );
		// Function for adding a nodes to the graph.
		pub fn add_node( 
				netuid: u16,
				coldkey: u64, 
				hotkey:u64, 
				uid: u16, 
				stake_amount: u64
			){
			log::info!(
				"+Add net:{:?} coldkey:{:?} hotkey:{:?} uid:{:?} stake_amount: {:?} subn: {:?}", 
				netuid,
				coldkey,
				hotkey,
				uid,
				stake_amount,
				SubtensorModule::get_subnetwork_n(netuid),
			);
			SubtensorModule::increase_stake_on_coldkey_hotkey_account( &coldkey, &hotkey, stake_amount );
			SubtensorModule::append_neuron( netuid, &hotkey, 0 );
			assert_eq!( SubtensorModule::get_subnetwork_n(netuid) - 1 , uid );
		}
		// Build the graph with 10 items 
		// each with 1 stake and self weights.
		let n: usize = 10;
		let netuid: u16 = 0;
		add_network(netuid, u16::MAX - 1, 0); // set higher tempo to avoid built-in epoch, then manual epoch instead
		SubtensorModule::set_max_allowed_uids( netuid, n as u16 ); 
		for i in 0..10 {
			add_node(
				netuid,
				i as u64,
				i as u64,
				i as u16,
				1
			)
		}
		assert_eq!( SubtensorModule::get_subnetwork_n(netuid), 10 );
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		for i in 0..10 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(i), netuid, vec![ i as u16 ], vec![ u16::MAX ], 0));
		}
		// Run the epoch.
		SubtensorModule::epoch( 0, 1_000_000_000 );
		// Check return values.
		for i in 0..n {
			assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &(i as u64) ), 1 );
			assert_eq!( SubtensorModule::get_rank_for_uid( netuid, i as u16 ), 0 );
			assert_eq!( SubtensorModule::get_trust_for_uid( netuid, i as u16 ), 0 );
			assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, i as u16 ), 0 );
			assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, i as u16 ), 0 );
			assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, i as u16 ), 0 );
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, i as u16 ), 99999999 );
		}
	});
}

// Test an epoch on a graph with 512 nodes, of which the first 64 are validators setting non-self weights, and the rest servers setting only self-weights.
#[test]
fn test_512_graph() {
	let netuid: u16 = 0;
	let network_n: u16 = 512;
	let validators_n: u16 = 64;
	let max_stake_per_validator: u64 = 328_125_000_000_000; // 21_000_000_000_000_000 / 64
	let epochs: u16 = 3;
	log::info!( "test_{network_n:?}_graph ({validators_n:?} validators)" );
	for interleave in 0..3 {
		for server_self in vec![false, true] { // server-self weight off/on
			let (validators, servers) = distribute_nodes(validators_n as usize, network_n as usize, interleave as usize);
			let server: usize = servers[0] as usize;
			let validator: usize = validators[0] as usize;
			new_test_ext().execute_with(|| {
				init_run_epochs(netuid, network_n, &validators, &servers, epochs, max_stake_per_validator, server_self, &vec![], false, &vec![], false, false, 0, false);
				let bonds = SubtensorModule::get_bonds( netuid );
				for uid in validators {
					assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &(uid as u64) ), max_stake_per_validator );
					assert_eq!( SubtensorModule::get_rank_for_uid( netuid, uid ), 0 );
					assert_eq!( SubtensorModule::get_trust_for_uid( netuid, uid ), 0 );
					assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, uid ), 0 );
					assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, uid ), 0 );
					assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, uid ), 1023 ); // Note D = floor(1 / 64 * 65_535) = 1023
					assert_eq!( SubtensorModule::get_emission_for_uid( netuid, uid ), 7812500 ); // Note E = 0.5 / 200 * 1_000_000_000 = 7_812_500
					assert_eq!( bonds[uid as usize][validator], 0.0 );
					assert_eq!( bonds[uid as usize][server], I32F32::from_num(1023) / I32F32::from_num(65_535) ); // Note B_ij = floor(1 / 64 * 65_535) / 65_535 = 1023 / 65_535
				}
				for uid in servers {
					assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &(uid as u64) ), 0 );
					assert_eq!( SubtensorModule::get_rank_for_uid( netuid, uid ), 146 ); // Note R = floor(1 / (512 - 64) * 65_535) = 146
					assert_eq!( SubtensorModule::get_trust_for_uid( netuid, uid ), 65535 );
					assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, uid ), 146 ); // Note C = floor(1 / (512 - 64) * 65_535) = 146
					assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, uid ), 146 ); // Note I = floor(1 / (512 - 64) * 65_535) = 146
					assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, uid ), 0 );
					assert_eq!( SubtensorModule::get_emission_for_uid( netuid, uid ), 1116071 ); // Note E = floor(0.5 / (512 - 64) * 1_000_000_000) = 1_116_071
					assert_eq!( bonds[uid as usize][validator], 0.0 );
					assert_eq!( bonds[uid as usize][server], 0.0 );
				}
			});
		}
	}
}

// Test an epoch on a graph with 4096 nodes, of which the first 256 are validators setting random non-self weights, and the rest servers setting only self-weights.
#[test]
fn test_512_graph_random_weights() {
	let netuid: u16 = 0;
	let network_n: u16 = 512;
	let validators_n: u16 = 64;
	let epochs: u16 = 1;
	log::info!( "test_{network_n:?}_graph_random_weights ({validators_n:?} validators)" );
	for interleave in 0..3 {
		for server_self in vec![false, true] { // server-self weight off/on
			let (validators, servers) = distribute_nodes(validators_n as usize, network_n as usize, interleave as usize);
			let server: usize = servers[0] as usize;
			let validator: usize = validators[0] as usize;
			let (mut rank, mut incentive, mut dividend, mut emission, mut bondv, mut bonds): (Vec<u16>, Vec<u16>, Vec<u16>, Vec<u64>, Vec<I32F32>, Vec<I32F32>) = (vec![], vec![], vec![], vec![], vec![], vec![]);
			
			// Dense epoch
			new_test_ext().execute_with(|| {
				init_run_epochs(netuid, network_n, &validators, &servers, epochs, 1, server_self, &vec![], false, &vec![], false, true, interleave as u64, false);

				let bond = SubtensorModule::get_bonds( netuid );
				for uid in 0..network_n {
					rank.push( SubtensorModule::get_rank_for_uid( netuid, uid ) );
					incentive.push( SubtensorModule::get_incentive_for_uid( netuid, uid ) );
					dividend.push( SubtensorModule::get_dividends_for_uid( netuid, uid ) );
					emission.push( SubtensorModule::get_emission_for_uid( netuid, uid ) );
					bondv.push( bond[uid as usize][validator] );
					bonds.push( bond[uid as usize][server] );
				}
			});

			// Sparse epoch (same random seed as dense)
			new_test_ext().execute_with(|| {
				init_run_epochs(netuid, network_n, &validators, &servers, epochs, 1, server_self, &vec![], false, &vec![], false, true, interleave as u64, true);
				// Assert that dense and sparse epoch results are equal
				let bond = SubtensorModule::get_bonds( netuid );
				for uid in 0..network_n {
					assert_eq!( SubtensorModule::get_rank_for_uid( netuid, uid ), rank[uid as usize] );
					assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, uid ), incentive[uid as usize] );
					assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, uid ), dividend[uid as usize] );
					assert_eq!( SubtensorModule::get_emission_for_uid( netuid, uid ), emission[uid as usize] );
					assert_eq!( bond[uid as usize][validator], bondv[uid as usize] );
					assert_eq!( bond[uid as usize][server], bonds[uid as usize] );
				}
			});
		}
	}
}

// Test an epoch on a graph with 4096 nodes, of which the first 256 are validators setting non-self weights, and the rest servers setting only self-weights.
// #[test]
#[allow(dead_code)]
fn test_4096_graph() {
	let netuid: u16 = 0;
	let network_n: u16 = 4096;
	let validators_n: u16 = 256;
	let epochs: u16 = 1;
	let max_stake_per_validator: u64 = 82_031_250_000_000; // 21_000_000_000_000_000 / 256
	log::info!( "test_{network_n:?}_graph ({validators_n:?} validators)" );
	for interleave in 0..3 {
		let (validators, servers) = distribute_nodes(validators_n as usize, network_n as usize, interleave as usize);
		let server: usize = servers[0] as usize;
		let validator: usize = validators[0] as usize;
		for server_self in vec![false, true] { // server-self weight off/on
			new_test_ext().execute_with(|| {
				init_run_epochs(netuid, network_n, &validators, &servers, epochs, max_stake_per_validator, server_self, &vec![], false, &vec![], false, false, 0, true);
				assert_eq!(SubtensorModule::get_total_stake(), 21_000_000_000_000_000);
				let bonds = SubtensorModule::get_bonds( netuid );
				for uid in &validators {
					assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &(*uid as u64) ), max_stake_per_validator );
					assert_eq!( SubtensorModule::get_rank_for_uid( netuid, *uid ), 0 );
					assert_eq!( SubtensorModule::get_trust_for_uid( netuid, *uid ), 0 );
					assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, *uid ), 0 );
					assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, *uid ), 0 );
					assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, *uid ), 255 ); // Note D = floor(1 / 256 * 65_535)
					assert_eq!( SubtensorModule::get_emission_for_uid( netuid, *uid ), 1953125 ); // Note E = 0.5 / 256 * 1_000_000_000 = 1953125
					assert_eq!( bonds[*uid as usize][validator], 0.0 );
					assert_eq!( bonds[*uid as usize][server], I32F32::from_num(255) / I32F32::from_num(65_535) ); // Note B_ij = floor(1 / 256 * 65_535) / 65_535
				}
				for uid in &servers {
					assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &(*uid as u64) ), 0 );
					assert_eq!( SubtensorModule::get_rank_for_uid( netuid, *uid ), 17 ); // Note R = floor(1 / (4096 - 256) * 65_535) = 17
					assert_eq!( SubtensorModule::get_trust_for_uid( netuid, *uid ), 65535 );
					assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, *uid ), 17 ); // Note C = floor(1 / (4096 - 256) * 65_535) = 17
					assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, *uid ), 17 ); // Note I = floor(1 / (4096 - 256) * 65_535) = 17
					assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, *uid ), 0 );
					assert_eq!( SubtensorModule::get_emission_for_uid( netuid, *uid ), 130208 ); // Note E = floor(0.5 / (4096 - 256) * 1_000_000_000) = 130208
					assert_eq!( bonds[*uid as usize][validator], 0.0 );
					assert_eq!( bonds[*uid as usize][server], 0.0 );
				}
			});
		}
	}
}

// Test an epoch_sparse on a graph with 16384 nodes, of which the first 512 are validators setting non-self weights, and the rest servers setting only self-weights.
// #[test]
#[allow(dead_code)]
fn test_16384_graph_sparse() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 0;
		let n: u16 = 16384;
		let validators_n: u16 = 512;
		let validators: Vec<u16> = (0..validators_n).collect();
		let servers: Vec<u16> = (validators_n..n).collect();
		let server: u16 = servers[0];
		let epochs: u16 = 1;
		log::info!( "test_{n:?}_graph ({validators_n:?} validators)" );
		init_run_epochs(netuid, n, &validators, &servers, epochs, 1, false, &vec![], false, &vec![], false, false, 0, true);
		let bonds = SubtensorModule::get_bonds( netuid );
		for uid in validators {
			assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &(uid as u64) ), 1 );
			assert_eq!( SubtensorModule::get_rank_for_uid( netuid, uid ), 0 );
			assert_eq!( SubtensorModule::get_trust_for_uid( netuid, uid ), 0 );
			assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, uid ), 0 );
			assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, uid ), 127 ); // Note D = floor(1 / 512 * 65_535) = 127
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, uid ), 976085 ); // Note E = 0.5 / 512 * 1_000_000_000 = 976_562 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][server as usize], I32F32::from_num(127) / I32F32::from_num(65_535) ); // Note B_ij = floor(1 / 512 * 65_535) / 65_535 = 127 / 65_535
		}
		for uid in servers {
			assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &(uid as u64) ), 0 );
			assert_eq!( SubtensorModule::get_rank_for_uid( netuid, uid ), 4 ); // Note R = floor(1 / (16384 - 512) * 65_535) = 4
			assert_eq!( SubtensorModule::get_trust_for_uid( netuid, uid ), 65535 );
			assert_eq!( SubtensorModule::get_consensus_for_uid( netuid, uid ), 4 ); // Note C = floor(1 / (16384 - 512) * 65_535) = 4
			assert_eq!( SubtensorModule::get_incentive_for_uid( netuid, uid ), 4 ); // Note I = floor(1 / (16384 - 512) * 65_535) = 4
			assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, uid ), 0 );
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, uid ), 31517 ); // Note E = floor(0.5 / (16384 - 512) * 1_000_000_000) = 31502 (discrepancy)
			assert_eq!( bonds[uid as usize][0], 0.0 );
			assert_eq!( bonds[uid as usize][server as usize], 0.0 );
		}
	});
}

// Test that epoch masks out inactive stake of validators with outdated weights beyond activity cutoff.
#[test]
fn test_active_stake() {
	new_test_ext().execute_with(|| {
		let sparse: bool = true;
		let n: u16 = 4;
		let netuid: u16 = 0;
		let tempo: u16 = u16::MAX - 1;  // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
		let block_number: u64 = 0;
		let stake: u64 = 1;
		add_network(netuid, tempo, 0);
		SubtensorModule::set_max_allowed_uids( netuid, n );
		assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
		SubtensorModule::set_max_registrations_per_block( netuid, n );

		// === Register [validator1, validator2, server1, server2]
		for key in 0..n as u64 {
			SubtensorModule::add_balance_to_coldkey_account( &key, stake );
			let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, key * 1_000_000);
			assert_ok!(SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(key), netuid, block_number, nonce, work, key, key));
			SubtensorModule::increase_stake_on_coldkey_hotkey_account( &key, &key, stake );
		}
		assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), n);
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

		// === Issue validator permits
		assert_ok!( SubtensorModule::sudo_set_max_allowed_validators(<<Test as Config>::RuntimeOrigin>::root(), netuid, n) );
		assert_eq!( SubtensorModule::get_max_allowed_validators(netuid), n);
		SubtensorModule::epoch( netuid, 1_000_000_000 ); // run first epoch to set allowed validators
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block

		// === Set weights [val1->srv1: 0.5, val1->srv2: 0.5, val2->srv1: 0.5, val2->srv2: 0.5]
		for uid in 0..(n/2) as u64 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(uid), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize ], 0));
		}
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		let bonds = SubtensorModule::get_bonds( netuid );
		for uid in 0..n as u16 {
			// log::info!("\n{uid}" );
			// uid_stats(netuid, uid);
			// log::info!("bonds: {:?}", bonds[uid as usize]);
			if uid < n/2 {
				assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, uid ), 32767 ); // Note D = floor(0.5 * 65_535)
			}
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, uid ), 250000000 ); // Note E = 0.5 / (n/2) * 1_000_000_000 = 250_000_000
		}
		for validator in 0..(n/2) as usize {
			for on_validator in 0..(n/2) as usize {
				assert_eq!( bonds[validator][on_validator], 0 );
			}
			for server in ((n/2) as usize)..n as usize {
				assert_eq!( bonds[validator][server], I32F32::from_num(32767) / I32F32::from_num(65_535) ); // floor(0.5*(2^16-1))/(2^16-1)
			}
		}
        let activity_cutoff: u64 = SubtensorModule::get_activity_cutoff( netuid ) as u64;
		run_to_block( activity_cutoff + 2 ); // run to block where validator (uid 0, 1) weights become outdated

		// === Update uid 0 weights
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(0), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize ], 0));
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 5002; activity_cutoff: 5000
			Last update: [5002, 1, 0, 0]; Inactive: [false, true, true, true]; Block at registration: [0, 0, 0, 0]
			S: [0.25, 0.25, 0.25, 0.25]; S (mask): [0.25, 0, 0, 0]; S (mask+norm): [1, 0, 0, 0]
			validator_permits: [true, true, true, true]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
			W: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (permit): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (permit+diag): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (permit+diag+outdate): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			R: [0, 0, 0.5, 0.5]
			W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
			T: [0, 0, 1, 1]
			C: [0.006693358, 0.006693358, 0.9933076561, 0.9933076561]
			I: [0, 0, 0.5, 0.5]
			B: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			B (outdatedmask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			B (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			ΔB: [[(2, 0.5), (3, 0.5)], [(2, 0), (3, 0)], [], []]
			ΔB (norm): [[(2, 1), (3, 1)], [(2, 0), (3, 0)], [], []]
			emaB: [[(2, 0.55), (3, 0.55)], [(2, 0.45), (3, 0.45)], [], []]
			D: [0.55, 0.4499999997, 0, 0]
			nE: [0.275, 0.2249999999, 0.25, 0.25]
			E: [274999999, 224999999, 250000000, 250000000]
			P: [0.275, 0.2249999999, 0.25, 0.25]
			P (u16): [65535, 53619, 59577, 59577] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, 0 ), 36044 ); // Note D = floor((0.5 * 0.9 + 0.1) * 65_535)
		assert_eq!( SubtensorModule::get_emission_for_uid( netuid, 0 ), 274999999 ); // Note E = 0.5 * 0.55 * 1_000_000_000 = 275_000_000 (discrepancy)
		for server in ((n/2) as usize)..n as usize {
			assert_eq!( bonds[0][server], I32F32::from_num(36044) / I32F32::from_num(65_535) ); // floor(0.55*(2^16-1))/(2^16-1)
		}
		for validator in 1..(n/2) as u16 {
			assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, validator ), 29490 ); // Note D = floor((0.5 * 0.9) * 65_535)
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, validator  ), 224999999 ); // Note E = 0.5 * 0.45 * 1_000_000_000 = 225_000_000 (discrepancy)
			for server in ((n/2) as usize)..n as usize {
				assert_eq!( bonds[validator as usize][server], I32F32::from_num(29490) / I32F32::from_num(65_535) ); // floor(0.45*(2^16-1))/(2^16-1)
			}
		}

		// === Update uid 1 weights as well
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(1), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize ], 0));
		run_to_block( activity_cutoff + 3 ); // run to block where validator (uid 0, 1) weights become outdated
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 5003; activity_cutoff: 5000
			Last update: [5002, 5002, 0, 0]; Inactive: [false, false, true, true]; Block at registration: [0, 0, 0, 0]
			S: [0.25, 0.25, 0.25, 0.25]; S (mask): [0.25, 0.25, 0, 0]; S (mask+norm): [0.5, 0.5, 0, 0]
			validator_permits: [true, true, true, true]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
			W: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (permit): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (permit+diag): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (permit+diag+outdate): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			W (mask+norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			R: [0, 0, 0.5, 0.5]
			W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
			T: [0, 0, 1, 1]
			C: [0.006693358, 0.006693358, 0.9933076561, 0.9933076561]
			I: [0, 0, 0.5, 0.5]
			B: [[(2, 0.5499961851), (3, 0.5499961851)], [(2, 0.4499885556), (3, 0.4499885556)], [], []]
			B (outdatedmask): [[(2, 0.5499961851), (3, 0.5499961851)], [(2, 0.4499885556), (3, 0.4499885556)], [], []]
			B (mask+norm): [[(2, 0.5500045777), (3, 0.5500045777)], [(2, 0.449995422), (3, 0.449995422)], [], []]
			ΔB: [[(2, 0.25), (3, 0.25)], [(2, 0.25), (3, 0.25)], [], []]
			ΔB (norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			emaB: [[(2, 0.54500412), (3, 0.54500412)], [(2, 0.4549958797), (3, 0.4549958797)], [], []]
			D: [0.5450041203, 0.4549958794, 0, 0]
			nE: [0.27250206, 0.2274979397, 0.25, 0.25]
			E: [272502060, 227497939, 250000000, 250000000]
			P: [0.27250206, 0.2274979397, 0.25, 0.25]
			P (u16): [65535, 54711, 60123, 60123] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, 0 ), 35716 ); // Note D = floor((0.55 * 0.9 + 0.5 * 0.1) * 65_535)
		assert_eq!( SubtensorModule::get_emission_for_uid( netuid, 0 ), 272502060 ); // Note E = 0.5 * (0.55 * 0.9 + 0.5 * 0.1) * 1_000_000_000 = 272_500_000 (discrepancy)
		for server in ((n/2) as usize)..n as usize {
			assert_eq!( bonds[0][server], I32F32::from_num(35716) / I32F32::from_num(65_535) ); // floor((0.55 * 0.9 + 0.5 * 0.1)*(2^16-1))/(2^16-1)
		}
		assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, 1 ), 29818 ); // Note D = floor((0.45 * 0.9 + 0.5 * 0.1) * 65_535)
		assert_eq!( SubtensorModule::get_emission_for_uid( netuid, 1 ), 227497939 ); // Note E = 0.5 * (0.45 * 0.9 + 0.5 * 0.1) * 1_000_000_000 = 227_500_000 (discrepancy)
		for server in ((n/2) as usize)..n as usize {
			assert_eq!( bonds[1][server], I32F32::from_num(29818) / I32F32::from_num(65_535) ); // floor((0.45 * 0.9 + 0.5 * 0.1)*(2^16-1))/(2^16-1)
		}
	});
}

// Test that epoch masks out outdated weights and bonds of validators on deregistered servers.
#[test]
fn test_outdated_weights() {
	new_test_ext().execute_with(|| {
		let sparse: bool = true;
		let n: u16 = 4;
		let netuid: u16 = 0;
		let tempo: u16 = u16::MAX - 1;  // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
		let mut block_number: u64 = 0;
		let stake: u64 = 1;
		add_network(netuid, tempo, 0);
		SubtensorModule::set_max_allowed_uids( netuid, n );
		SubtensorModule::set_max_registrations_per_block( netuid, n+1 ); // should be n, but RegistrationsThisBlock is not reset (TODO: Saeideh)

		// === Register [validator1, validator2, server1, server2]
		for key in 0..n as u64 {
			SubtensorModule::add_balance_to_coldkey_account( &key, stake );
			let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, key * 1_000_000);
			assert_ok!(SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(key), netuid, block_number, nonce, work, key, key));
			SubtensorModule::increase_stake_on_coldkey_hotkey_account( &key, &key, stake );
		}
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

		// === Issue validator permits
		assert_ok!( SubtensorModule::sudo_set_max_allowed_validators(<<Test as Config>::RuntimeOrigin>::root(), netuid, n) );
		assert_eq!( SubtensorModule::get_max_allowed_validators(netuid), n);
		SubtensorModule::epoch( netuid, 1_000_000_000 ); // run first epoch to set allowed validators
		run_to_block( 1 ); block_number += 1; // run to next block to ensure weights are set on nodes after their registration block

		// === Set weights [val1->srv1: 2/3, val1->srv2: 1/3, val2->srv1: 2/3, val2->srv2: 1/3, srv1->srv1: 1, srv2->srv2: 1]
		for uid in 0..(n/2) as u64 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(uid), netuid, ((n/2)..n).collect(), vec![ 2 * (u16::MAX / 3), u16::MAX / 3 ], 0));
		}
		for uid in ((n/2) as u64)..n as u64 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(uid), netuid, vec![ uid as u16 ], vec![ u16::MAX ], 0)); // server self-weight
		}
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 1; activity_cutoff: 5000
			Last update: [1, 1, 1, 1]; Inactive: [false, false, false, false]; Block at registration: [0, 0, 0, 0]
			S: [0.25, 0.25, 0.25, 0.25]; S (mask): [0.25, 0.25, 0.25, 0.25]; S (mask+norm): [0.25, 0.25, 0.25, 0.25]
			validator_permits: [true, true, true, true]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
			W: [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], [(3, 1)]]
			W (permit): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], [(3, 1)]]
			W (permit+diag): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			W (permit+diag+outdate): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			W (mask+norm): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			R: [0, 0, 0.6666666665, 0.3333333333]
			W (threshold): [[(2, 1), (3, 1)], [(2, 1), (3, 1)], [], []]
			T: [0, 0, 0.5, 0.5]
			C: [0.000000306, 0.000000306, 0.500057222, 0.500057222]
			I: [0, 0, 0.6666666667, 0.333333333]
			B: [[], [], [], []]
			B (outdatedmask): [[], [], [], []]
			B (mask+norm): [[], [], [], []]
			ΔB: [[(2, 0.1666666665), (3, 0.0833333333)], [(2, 0.1666666665), (3, 0.0833333333)], [], []]
			ΔB (norm): [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			emaB: [[(2, 0.5), (3, 0.5)], [(2, 0.5), (3, 0.5)], [], []]
			D: [0.5, 0.5, 0, 0]
			nE: [0.25, 0.25, 0.3333333333, 0.1666666665]
			E: [250000000, 250000000, 333333333, 166666666]
			P: [0.25, 0.25, 0.3333333333, 0.1666666665]
			P (u16): [49151, 49151, 65535, 32767] */

		// === Dereg server2 at uid3 (least emission) + register new key over uid3
		let new_key: u64 = n as u64; // register a new key while at max capacity, which means the least incentive uid will be deregistered
		let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, 0);
		assert_ok!(SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(new_key), netuid, block_number, nonce, work, new_key, new_key));
		let deregistered_uid: u16 = n-1; // since uid=n-1 only recieved 1/3 of weight, it will get pruned first
		assert_eq!(new_key, SubtensorModule::get_hotkey_for_net_and_uid(netuid, deregistered_uid).expect("Not registered"));
		run_to_block( 2 ); // run to next block to outdate weights and bonds set on deregistered uid

		// === Update weights from only uid=0
		assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(0), netuid, ((n/2)..n).collect(), vec![ 2 * (u16::MAX / 3), u16::MAX / 3 ], 0));
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*  current_block: 2; activity_cutoff: 5000
			Last update: [2, 1, 1, 1]; Inactive: [false, false, false, false]; Block at registration: [0, 0, 0, 1]
			S: [0.3333333333, 0.3333333333, 0.3333333333, 0]
			S (mask): [0.3333333333, 0.3333333333, 0.3333333333, 0]
			S (mask+norm): [0.3333333333, 0.3333333333, 0.3333333333, 0]
			validator_permits: [true, true, true, false]; max_allowed_validators: 4; new_validator_permits: [true, true, true, true]
			W: [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], []]
			W (permit): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], []]
			W (permit+diag): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665), (3, 0.3333333333)], [], []]
			W (permit+diag+outdate): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.6666666665)], [], []]
			W (mask+norm): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 1)], [], []]
			W (0): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.8666666665)], [], []]
			W (1): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.7797101443)], [], []]
			W (2): [[(2, 0.6666666665), (3, 0.3333333333)], [(2, 0.727605936)], [], []]
			R (before): [0, 0, 0.5555555553, 0.111111111]
			C: [0, 0, 0.6666666665, 0]
			W: [[(2, 0.6666666665)], [(2, 0.6666666665)], [], []]
			Tv: [0.6666666665, 0.6666666665, 0, 0]
			R (after): [0, 0, 0.444444444, 0]
			T: [0, 0, 0.7999999996, 0]
			I (=R): [0, 0, 1, 0]
			B: [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704), (3, 0.4999923704)], [], []]
			B (outdatedmask): [[(2, 0.4999923704), (3, 0.4999923704)], [(2, 0.4999923704)], [], []]
			B (mask+norm): [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
			ΔB: [[(2, 0.222222222)], [(2, 0.222222222)], [], []]
			ΔB (norm): [[(2, 0.5)], [(2, 0.5)], [], []]
			emaB: [[(2, 0.5), (3, 1)], [(2, 0.5)], [], []]
			D: [0.5, 0.5, 0, 0]
			nE: [0.25, 0.25, 0.5, 0]
			E: [250000000, 250000000, 500000000, 0]
			P: [0.25, 0.25, 0.5, 0]
			P (u16): [32767, 32767, 65535, 0] */
		let bonds = SubtensorModule::get_bonds( netuid );
		assert_eq!( SubtensorModule::get_dividends_for_uid( netuid, 0 ), 32767 ); // Note D = floor(0.5 * 65_535)
		assert_eq!( SubtensorModule::get_emission_for_uid( netuid, 0 ), 250000000 ); // Note E = 0.5 * 0.5 * 1_000_000_000 = 249311245
		assert_eq!( bonds[0][2], I32F32::from_num(32767) / I32F32::from_num(65_535) ); // floor(0.5*(2^16-1))/(2^16-1)
		assert_eq!( bonds[0][3], I32F32::from_num(1) ); // only uid0 has updated weights for new reg
	});
}

// Test the zero emission handling and fallback under zero effective weight conditions, to ensure non-zero effective emission.
#[test]
fn test_zero_weights() {
	new_test_ext().execute_with(|| {
		let sparse: bool = true;
		let n: u16 = 2;
		let netuid: u16 = 0;
		let tempo: u16 = u16::MAX - 1;  // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
		let mut block_number: u64 = 0;
		let stake: u64 = 1;
		add_network(netuid, tempo, 0);
		SubtensorModule::set_max_allowed_uids( netuid, n );
		SubtensorModule::set_max_registrations_per_block( netuid, n+1 ); // should be n, but RegistrationsThisBlock is not reset (TODO: Saeideh)

		// === Register [validator, server]
		for key in 0..n as u64 {
			let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, key * 1_000_000);
			assert_ok!(SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(key), netuid, block_number, nonce, work, key, key));
		}
		for validator in 0..(n/2) as u64 {
			SubtensorModule::add_balance_to_coldkey_account( &validator, stake );
			SubtensorModule::increase_stake_on_coldkey_hotkey_account( &validator, &validator, stake );
		}
		assert_eq!(SubtensorModule::get_subnetwork_n(netuid), n);

		// === No weights
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*	current_block: 0; activity_cutoff: 5000; Last update: [0, 0]; Inactive: [false, false]
			S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 0]
			W: [[], []]; W (diagmask): [[], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []]
			R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0]
			B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []];
			ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0]
			E: [1000000000, 0]; P: [1, 0] */
		for validator in 0..(n/2) as u16 {
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, validator ), 1000000000 ); // Note E = 1 * 1_000_000_000
		}
		for server in (n/2)..n as u16 {
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, server ), 0 ); // no stake
		}
		run_to_block( 1 ); block_number += 1; // run to next block to ensure weights are set on nodes after their registration block

		// === Self-weights only: set weights [srv->srv: 1]
		for uid in ((n/2) as u64)..n as u64 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(uid), netuid, vec![ uid as u16 ], vec![ u16::MAX ], 0)); // server self-weight
		}
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*	current_block: 1; activity_cutoff: 5000; Last update: [0, 1]; Inactive: [false, false]
			S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 0]
			W: [[], [(1, 1)]]
			W (diagmask): [[], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []]
			R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0]
			B: [[], []]: B (outdatedmask): [[], []]; B (mask+norm): [[], []]
			ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0]
			E: [1000000000, 0]; P: [1, 0] */
		for validator in 0..(n/2) as u16 {
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, validator ), 1000000000 ); // Note E = 1 * 1_000_000_000
		}
		for server in (n/2)..n as u16 {
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, server ), 0 ); // no stake
		}
		run_to_block( 2 ); block_number += 1;

		// === Set weights [val->srv: 1/(n/2)]
		for uid in 0..(n/2) as u64 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(uid), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize ], 0));
		}

		// === Outdate weights by reregistering servers
		for new_key in n..n+(n/2) {// register a new key while at max capacity, which means the least emission uid will be deregistered
			let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, new_key as u64 * 1_000_000);
			assert_ok!(SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(new_key as u64), netuid, block_number, nonce, work, new_key as u64, new_key as u64));
		}
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*	current_block: 2; activity_cutoff: 5000; Last update: [2, 1]; Inactive: [false, false]; 
		S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 2]; 
		W: [[(1, 1)], []]; W (diagmask): [[(1, 1)], []]; W (diag+outdatemask): [[], []]; W (mask+norm): [[], []]; 
		R: [0, 0]; W (threshold): [[], []]; T: [0, 0]; C: [0.006693358, 0.006693358]; I: [0, 0]; 
		B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []]; 
		ΔB: [[], []]; ΔB (norm): [[], []]; emaB: [[], []]; D: [0, 0]; 
		E: [1000000000, 0]; P: [1, 0] */
		for validator in 0..(n/2) as u16 {
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, validator ), 1000000000 ); // Note E = 1 * 1_000_000_000
		}
		for server in (n/2)..n as u16 {
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, server ), 0 ); // no stake
		}
		run_to_block( 3 );

		// === Set new weights [val->srv: 1/(n/2)] to check that updated weights would produce non-zero incentive
		for uid in 0..(n/2) as u64 {
			assert_ok!(SubtensorModule::set_weights(RuntimeOrigin::signed(uid), netuid, ((n/2)..n).collect(), vec![ u16::MAX / (n/2); (n/2) as usize], 0));
		}
		if sparse { SubtensorModule::epoch( netuid, 1_000_000_000 ); }
		else { SubtensorModule::epoch_dense( netuid, 1_000_000_000 ); }
		/*	current_block: 3; activity_cutoff: 5000; Last update: [3, 1]; Inactive: [false, false]; 
		S: [1, 0]; S (mask): [1, 0]; S (mask+norm): [1, 0]; Block at registration: [0, 2]; 
		W: [[(1, 1)], []]; W (diagmask): [[(1, 1)], []]; W (diag+outdatemask): [[(1, 1)], []]; W (mask+norm): [[(1, 1)], []]; 
		R: [0, 1]; W (threshold): [[(1, 1)], []]; T: [0, 1]; C: [0.006693358, 0.9933076561]; I: [0, 1]; 
		B: [[], []]; B (outdatedmask): [[], []]; B (mask+norm): [[], []]; 
		ΔB: [[(1, 1)], []]; ΔB (norm): [[(1, 1)], []]; emaB: [[(1, 1)], []]; D: [1, 0]; 
		E: [500000000, 500000000]; P: [0.5, 0.5] */
		for validator in 0..n as u16 {
			assert_eq!( SubtensorModule::get_emission_for_uid( netuid, validator ), 1000000000 / (n as u64) ); // Note E = 1/2 * 1_000_000_000
		}
	});
}

// Test that epoch assigns validator permits to highest stake uids, varies uid interleaving and stake values.
#[test]
fn test_validator_permits() {
	let netuid: u16 = 0;
	let tempo: u16 = u16::MAX - 1;  // high tempo to skip automatic epochs in on_initialize, use manual epochs instead
	for interleave in 0..3 {
		for (network_n, validators_n) in vec![(2, 1), (4, 2), (8, 4)] {
			for assignment in 0..=1 {
				let (validators, servers) = distribute_nodes(validators_n as usize, network_n as usize, interleave as usize);
				let correct: bool = true;
				let mut stake: Vec<u64> = vec![0; network_n];
				for validator in &validators {
					stake[*validator as usize] = match assignment {
						1 => *validator as u64 + network_n as u64,
						_ => 1
					};
				}
				for server in &servers {
					stake[*server as usize] = match assignment {
						1 => *server as u64,
						_ => 0
					};
				}
				new_test_ext().execute_with(|| {
					let block_number: u64 = 0;
					add_network(netuid, tempo, 0);
					SubtensorModule::set_max_allowed_uids( netuid, network_n as u16 );
					assert_eq!(SubtensorModule::get_max_allowed_uids(netuid), network_n as u16 );
					SubtensorModule::set_max_registrations_per_block( netuid, network_n as u16 );
					SubtensorModule::set_target_registrations_per_interval(netuid, network_n as u16);
			
					// === Register [validator1, validator2, server1, server2]
					for key in 0..network_n as u64 {
						SubtensorModule::add_balance_to_coldkey_account( &key, stake[key as usize] );
						let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, key * 1_000_000);
						assert_ok!(SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(key), netuid, block_number, nonce, work, key, key));
						SubtensorModule::increase_stake_on_coldkey_hotkey_account( &key, &key, stake[key as usize] );
					}
					assert_eq!(SubtensorModule::get_subnetwork_n(netuid), network_n as u16);
			
					// === Issue validator permits
					assert_ok!( SubtensorModule::sudo_set_max_allowed_validators(<<Test as Config>::RuntimeOrigin>::root(), netuid, validators_n as u16) );
					assert_eq!( SubtensorModule::get_max_allowed_validators(netuid), validators_n as u16);
					SubtensorModule::epoch( netuid, 1_000_000_000 ); // run first epoch to set allowed validators
					for validator in &validators {
						assert_eq!(correct, SubtensorModule::get_validator_permit_for_uid(netuid, *validator));
					}
					for server in &servers {
						assert_eq!(!correct, SubtensorModule::get_validator_permit_for_uid(netuid, *server));
					}

					// === Increase server stake above validators
					for server in &servers {
						SubtensorModule::add_balance_to_coldkey_account( &(*server as u64), 2 * network_n as u64 );
						SubtensorModule::increase_stake_on_coldkey_hotkey_account( &(*server as u64), &(*server as u64), 2*network_n as u64 );
					}

					// === Update validator permits
					run_to_block( 1 );
					SubtensorModule::epoch( netuid, 1_000_000_000 );

					// === Check that servers now own permits instead of the validator uids
					for validator in &validators {
						assert_eq!(!correct, SubtensorModule::get_validator_permit_for_uid(netuid, *validator));
					}
					for server in &servers {
						assert_eq!(correct, SubtensorModule::get_validator_permit_for_uid(netuid, *server));
					}
				});
			}
		}
	}
}

// Map the retention graph for consensus guarantees with an single epoch on a graph with 512 nodes, of which the first 64 are validators, the graph is split into a major and minor set, each setting specific weight on itself and the complement on the other.
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
#[test]
fn test_map_consensus_guarantees() {
	let args: Vec<String> = std::env::args().collect();
	let weight_stddev_major: I32F32 = fixed(args[args.len()-2].parse().unwrap_or(0.0));
	let weight_stddev_minor: I32F32 = fixed(args[args.len()-1].parse().unwrap_or(0.0));
	println!("weight_stddev_major: {weight_stddev_major}, weight_stddev_minor: {weight_stddev_minor}");
	let netuid: u16 = 0;
	let network_n: u16 = 512;
	let validators_n: u16 = 64;
	let epochs: u16 = 1;
	let interleave = 0;
	println!("[");
	for _major_stake in vec![0.51, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 0.99] {
		let major_stake: I32F32 = I32F32::from_num(_major_stake);
		for _major_weight in 0..51 {
			let major_weight: I32F32 = I32F32::from_num(50 - _major_weight) / I32F32::from_num(50);
			for _minor_weight in 0..51 {
				let minor_weight: I32F32 = I32F32::from_num(50 - _minor_weight) / I32F32::from_num(50);
				let (validators, servers, major_validators, minor_validators, major_servers, minor_servers, stake, weights, avg_weight_dev) = split_graph(major_stake, major_weight, minor_weight, weight_stddev_major, weight_stddev_minor, validators_n as usize, network_n as usize, interleave as usize);

				new_test_ext().execute_with(|| {
					init_run_epochs(netuid, network_n, &validators, &servers, epochs, 1, true, &stake, true, &weights, true, false, 0, true);

					let mut major_emission: I64F64 = I64F64::from_num(0);
					let mut minor_emission: I64F64 = I64F64::from_num(0);
					for set in vec![major_validators, major_servers] {
						for uid in set {
							major_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
						}
					}
					for set in vec![minor_validators, minor_servers] {
						for uid in set {
							minor_emission += I64F64::from_num(SubtensorModule::get_emission_for_uid( netuid, uid ));
						}
					}
					let major_ratio: I32F32 = I32F32::from_num(major_emission / (major_emission + minor_emission));
					println!("[{major_stake}, {major_weight:.2}, {minor_weight:.2}, {avg_weight_dev:.3}, {major_ratio:.3}], ");
				});
			}	
		}
	}
	println!("]");
}
