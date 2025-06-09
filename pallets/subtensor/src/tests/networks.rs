use super::mock::*;
use crate::*;
use frame_support::{assert_err, assert_ok};
use frame_system::Config;
use sp_core::U256;
use substrate_fixed::types::U64F64;

#[test]
fn test_registration_ok() {
    new_test_ext(1).execute_with(|| {
        let block_number: u64 = 0;
        let netuid: u16 = 2;
        let tempo: u16 = 13;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(0); // Neighbour of the beast, har har
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            block_number,
            129123813,
            &hotkey_account_id,
        );

        //add network
        add_network(netuid, tempo, 0);

        assert_ok!(SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
            netuid,
            block_number,
            nonce,
            work.clone(),
            hotkey_account_id,
            coldkey_account_id
        ));

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        assert!(!SubtensorModule::if_subnet_exist(netuid))
    })
}

#[test]
fn dissolve_no_stakers_no_alpha_no_emission() {
    new_test_ext(0).execute_with(|| {
        let cold = U256::from(1);
        let hot = U256::from(2);
        let net = add_dynamic_network(&hot, &cold);

        SubtensorModule::set_subnet_locked_balance(net, 0);
        SubnetTAO::<Test>::insert(net, 0);
        Emission::<Test>::insert(net, Vec::<u64>::new());

        let before = SubtensorModule::get_coldkey_balance(&cold);
        assert_ok!(SubtensorModule::do_dissolve_network(net));
        let after = SubtensorModule::get_coldkey_balance(&cold);

        // Balance should be unchanged (whatever the network-lock bookkeeping left there)
        assert_eq!(after, before);
        assert!(!SubtensorModule::if_subnet_exist(net));
    });
}

#[test]
fn dissolve_refunds_full_lock_cost_when_no_emission() {
    new_test_ext(0).execute_with(|| {
        let cold = U256::from(3);
        let hot = U256::from(4);
        let net = add_dynamic_network(&hot, &cold);

        let lock = 1_000_000u64;
        SubtensorModule::set_subnet_locked_balance(net, lock);
        SubnetTAO::<Test>::insert(net, 0);
        Emission::<Test>::insert(net, Vec::<u64>::new());

        let before = SubtensorModule::get_coldkey_balance(&cold);
        assert_ok!(SubtensorModule::do_dissolve_network(net));
        let after = SubtensorModule::get_coldkey_balance(&cold);

        assert_eq!(after, before + lock);
    });
}

#[test]
fn dissolve_single_alpha_out_staker_gets_all_tao() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(10);
        let owner_hot = U256::from(20);
        let net = add_dynamic_network(&owner_hot, &owner_cold);

        let s_hot = U256::from(100);
        let s_cold = U256::from(200);

        Alpha::<Test>::insert((s_hot, s_cold, net), U64F64::from_num(5_000u128));
        SubnetTAO::<Test>::insert(net, 99_999);
        SubtensorModule::set_subnet_locked_balance(net, 0);
        Emission::<Test>::insert(net, Vec::<u64>::new());

        let before = SubtensorModule::get_coldkey_balance(&s_cold);
        assert_ok!(SubtensorModule::do_dissolve_network(net));
        let after = SubtensorModule::get_coldkey_balance(&s_cold);

        assert_eq!(after, before + 99_999);
        assert!(Alpha::<Test>::iter().count() == 0);
    });
}

#[test]
fn dissolve_two_stakers_pro_rata_distribution() {
    new_test_ext(0).execute_with(|| {
        let oc = U256::from(50);
        let oh = U256::from(51);
        let net = add_dynamic_network(&oh, &oc);

        // stakers α-out
        let (s1_hot, s1_cold, a1) = (U256::from(201), U256::from(301), 300u128);
        let (s2_hot, s2_cold, a2) = (U256::from(202), U256::from(302), 700u128);

        Alpha::<Test>::insert((s1_hot, s1_cold, net), U64F64::from_num(a1));
        Alpha::<Test>::insert((s2_hot, s2_cold, net), U64F64::from_num(a2));

        SubnetTAO::<Test>::insert(net, 10_000);
        SubtensorModule::set_subnet_locked_balance(net, 5_000);
        Emission::<Test>::insert(net, Vec::<u64>::new());

        let b1 = SubtensorModule::get_coldkey_balance(&s1_cold);
        let b2 = SubtensorModule::get_coldkey_balance(&s2_cold);
        let bo = SubtensorModule::get_coldkey_balance(&oc);

        assert_ok!(SubtensorModule::do_dissolve_network(net));

        let total = a1 + a2;
        let share1: u64 = (10_000u128 * a1 / total) as u64;
        let share2: u64 = (10_000u128 * a2 / total) as u64;

        assert_eq!(SubtensorModule::get_coldkey_balance(&s1_cold), b1 + share1);
        assert_eq!(SubtensorModule::get_coldkey_balance(&s2_cold), b2 + share2);
        assert_eq!(SubtensorModule::get_coldkey_balance(&oc), bo + 5_000);
    });
}

#[test]
fn dissolve_owner_cut_refund_logic() {
    new_test_ext(0).execute_with(|| {
        let oc = U256::from(70);
        let oh = U256::from(71);
        let net = add_dynamic_network(&oh, &oc);

        // staker
        let sh = U256::from(77);
        let sc = U256::from(88);
        Alpha::<Test>::insert((sh, sc, net), U64F64::from_num(100u128));
        SubnetTAO::<Test>::insert(net, 1_000);

        // lock & emission
        let lock = 2_000;
        SubtensorModule::set_subnet_locked_balance(net, lock);
        Emission::<Test>::insert(net, vec![200u64, 600]);

        // 18 % owner-cut
        SubnetOwnerCut::<Test>::put(11_796u16);
        let frac = 11_796f64 / 65_535f64;
        let owner_em = (800f64 * frac).floor() as u64;
        let expect = lock.saturating_sub(owner_em);

        let before = SubtensorModule::get_coldkey_balance(&oc);
        assert_ok!(SubtensorModule::do_dissolve_network(net));
        let after = SubtensorModule::get_coldkey_balance(&oc);

        assert_eq!(after, before + expect);
    });
}

#[test]
fn dissolve_zero_refund_when_emission_exceeds_lock() {
    new_test_ext(0).execute_with(|| {
        let oc = U256::from(1_000);
        let oh = U256::from(2_000);
        let net = add_dynamic_network(&oh, &oc);

        SubtensorModule::set_subnet_locked_balance(net, 1_000);
        SubnetOwnerCut::<Test>::put(u16::MAX); // 100 %
        Emission::<Test>::insert(net, vec![2_000u64]);

        let before = SubtensorModule::get_coldkey_balance(&oc);
        assert_ok!(SubtensorModule::do_dissolve_network(net));
        let after = SubtensorModule::get_coldkey_balance(&oc);

        assert_eq!(after, before); // no refund
    });
}

#[test]
fn dissolve_nonexistent_subnet_fails() {
    new_test_ext(0).execute_with(|| {
        assert_err!(
            SubtensorModule::do_dissolve_network(9_999),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn dissolve_clears_all_per_subnet_storages() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(123);
        let owner_hot = U256::from(456);
        let net = add_dynamic_network(&owner_hot, &owner_cold);

        // ------------------------------------------------------------------
        // Populate each storage item with a minimal value of the CORRECT type
        // ------------------------------------------------------------------
        SubnetOwner::<Test>::insert(net, owner_cold);
        SubnetworkN::<Test>::insert(net, 0u16);
        NetworkModality::<Test>::insert(net, 0u16);
        NetworksAdded::<Test>::insert(net, true);
        NetworkRegisteredAt::<Test>::insert(net, 0u64);

        Rank::<Test>::insert(net, vec![1u16]);
        Trust::<Test>::insert(net, vec![1u16]);
        Active::<Test>::insert(net, vec![true]);
        Emission::<Test>::insert(net, vec![1u64]);
        Incentive::<Test>::insert(net, vec![1u16]);
        Consensus::<Test>::insert(net, vec![1u16]);
        Dividends::<Test>::insert(net, vec![1u16]);
        PruningScores::<Test>::insert(net, vec![1u16]);
        LastUpdate::<Test>::insert(net, vec![0u64]);

        ValidatorPermit::<Test>::insert(net, vec![true]);
        ValidatorTrust::<Test>::insert(net, vec![1u16]);

        Tempo::<Test>::insert(net, 1u16);
        Kappa::<Test>::insert(net, 1u16);
        Difficulty::<Test>::insert(net, 1u64);

        MaxAllowedUids::<Test>::insert(net, 1u16);
        ImmunityPeriod::<Test>::insert(net, 1u16);
        ActivityCutoff::<Test>::insert(net, 1u16);
        MaxWeightsLimit::<Test>::insert(net, 1u16);
        MinAllowedWeights::<Test>::insert(net, 1u16);

        RegistrationsThisInterval::<Test>::insert(net, 1u16);
        POWRegistrationsThisInterval::<Test>::insert(net, 1u16);
        BurnRegistrationsThisInterval::<Test>::insert(net, 1u16);

        SubnetTAO::<Test>::insert(net, 1u64);
        SubnetAlphaInEmission::<Test>::insert(net, 1u64);
        SubnetAlphaOutEmission::<Test>::insert(net, 1u64);
        SubnetTaoInEmission::<Test>::insert(net, 1u64);
        SubnetVolume::<Test>::insert(net, 1u128);

        // Fields that will be ZEROED (not removed)
        SubnetAlphaIn::<Test>::insert(net, 2u64);
        SubnetAlphaOut::<Test>::insert(net, 3u64);

        // Prefix / double-map collections
        Keys::<Test>::insert(net, 0u16, owner_hot);
        Bonds::<Test>::insert(net, 0u16, vec![(0u16, 1u16)]);
        Weights::<Test>::insert(net, 0u16, vec![(1u16, 1u16)]);
        IsNetworkMember::<Test>::insert(owner_cold, net, true);

        // ------------------------------------------------------------------
        // Dissolve
        // ------------------------------------------------------------------
        assert_ok!(SubtensorModule::do_dissolve_network(net));

        // ------------------------------------------------------------------
        // Items that must be COMPLETELY REMOVED
        // ------------------------------------------------------------------
        assert!(!SubnetOwner::<Test>::contains_key(net));
        assert!(!SubnetworkN::<Test>::contains_key(net));
        assert!(!NetworkModality::<Test>::contains_key(net));
        assert!(!NetworksAdded::<Test>::contains_key(net));
        assert!(!NetworkRegisteredAt::<Test>::contains_key(net));

        assert!(!Rank::<Test>::contains_key(net));
        assert!(!Trust::<Test>::contains_key(net));
        assert!(!Active::<Test>::contains_key(net));
        assert!(!Emission::<Test>::contains_key(net));
        assert!(!Incentive::<Test>::contains_key(net));
        assert!(!Consensus::<Test>::contains_key(net));
        assert!(!Dividends::<Test>::contains_key(net));
        assert!(!PruningScores::<Test>::contains_key(net));
        assert!(!LastUpdate::<Test>::contains_key(net));

        assert!(!ValidatorPermit::<Test>::contains_key(net));
        assert!(!ValidatorTrust::<Test>::contains_key(net));

        assert!(!Tempo::<Test>::contains_key(net));
        assert!(!Kappa::<Test>::contains_key(net));
        assert!(!Difficulty::<Test>::contains_key(net));

        assert!(!MaxAllowedUids::<Test>::contains_key(net));
        assert!(!ImmunityPeriod::<Test>::contains_key(net));
        assert!(!ActivityCutoff::<Test>::contains_key(net));
        assert!(!MaxWeightsLimit::<Test>::contains_key(net));
        assert!(!MinAllowedWeights::<Test>::contains_key(net));

        assert!(!RegistrationsThisInterval::<Test>::contains_key(net));
        assert!(!POWRegistrationsThisInterval::<Test>::contains_key(net));
        assert!(!BurnRegistrationsThisInterval::<Test>::contains_key(net));

        assert!(!SubnetTAO::<Test>::contains_key(net));
        assert!(!SubnetAlphaInEmission::<Test>::contains_key(net));
        assert!(!SubnetAlphaOutEmission::<Test>::contains_key(net));
        assert!(!SubnetTaoInEmission::<Test>::contains_key(net));
        assert!(!SubnetVolume::<Test>::contains_key(net));

        // ------------------------------------------------------------------
        // Items expected to be PRESENT but ZERO
        // ------------------------------------------------------------------
        assert_eq!(SubnetAlphaIn::<Test>::get(net), 0);
        assert_eq!(SubnetAlphaOut::<Test>::get(net), 0);

        // ------------------------------------------------------------------
        // Collections fully cleared
        // ------------------------------------------------------------------
        assert!(Keys::<Test>::iter_prefix(net).next().is_none());
        assert!(Bonds::<Test>::iter_prefix(net).next().is_none());
        assert!(Weights::<Test>::iter_prefix(net).next().is_none());
        assert!(!IsNetworkMember::<Test>::contains_key(owner_hot, net));

        // ------------------------------------------------------------------
        // Final subnet removal confirmation
        // ------------------------------------------------------------------
        assert!(!SubtensorModule::if_subnet_exist(net));
    });
}

#[test]
fn dissolve_alpha_out_but_zero_tao_no_rewards() {
    new_test_ext(0).execute_with(|| {
        let oc = U256::from(21);
        let oh = U256::from(22);
        let net = add_dynamic_network(&oh, &oc);

        let sh = U256::from(23);
        let sc = U256::from(24);

        Alpha::<Test>::insert((sh, sc, net), U64F64::from_num(1_000u128));
        SubnetTAO::<Test>::insert(net, 0u64); // zero TAO
        SubtensorModule::set_subnet_locked_balance(net, 0);
        Emission::<Test>::insert(net, Vec::<u64>::new());

        let before = SubtensorModule::get_coldkey_balance(&sc);
        assert_ok!(SubtensorModule::do_dissolve_network(net));
        let after = SubtensorModule::get_coldkey_balance(&sc);

        // No reward distributed, α-out cleared.
        assert_eq!(after, before);
        assert!(Alpha::<Test>::iter().next().is_none());
    });
}

#[test]
fn dissolve_decrements_total_networks() {
    new_test_ext(0).execute_with(|| {
        let total_before = TotalNetworks::<Test>::get();

        let cold = U256::from(41);
        let hot = U256::from(42);
        let net = add_dynamic_network(&hot, &cold);

        // Sanity: adding network increments the counter.
        assert_eq!(TotalNetworks::<Test>::get(), total_before + 1);

        assert_ok!(SubtensorModule::do_dissolve_network(net));
        assert_eq!(TotalNetworks::<Test>::get(), total_before);
    });
}

#[test]
fn dissolve_rounding_remainder_distribution() {
    new_test_ext(0).execute_with(|| {
        let oc = U256::from(61);
        let oh = U256::from(62);
        let net = add_dynamic_network(&oh, &oc);

        // α-out stakes
        let (s1h, s1c, a1) = (U256::from(63), U256::from(64), 3u128);
        let (s2h, s2c, a2) = (U256::from(65), U256::from(66), 2u128);

        Alpha::<Test>::insert((s1h, s1c, net), U64F64::from_num(a1));
        Alpha::<Test>::insert((s2h, s2c, net), U64F64::from_num(a2));

        // TAO pot = 1
        SubnetTAO::<Test>::insert(net, 1u64);
        SubtensorModule::set_subnet_locked_balance(net, 0);
        Emission::<Test>::insert(net, Vec::<u64>::new());

        let b1 = SubtensorModule::get_coldkey_balance(&s1c);
        let b2 = SubtensorModule::get_coldkey_balance(&s2c);

        assert_ok!(SubtensorModule::do_dissolve_network(net));

        // s1 (larger remainder) receives the single Tao.
        assert_eq!(SubtensorModule::get_coldkey_balance(&s1c), b1 + 1);
        assert_eq!(SubtensorModule::get_coldkey_balance(&s2c), b2);

        // α-records cleared; TAO storage gone.
        assert!(Alpha::<Test>::iter().next().is_none());
        assert!(!SubnetTAO::<Test>::contains_key(net));
    });
}

#[test]
fn destroy_alpha_out_multiple_stakers_pro_rata() {
    new_test_ext(0).execute_with(|| {
        // --------------------------------------------------
        // 1. Subnet owner + subnet creation
        // --------------------------------------------------
        let owner_cold = U256::from(10);
        let owner_hot = U256::from(20);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);

        // --------------------------------------------------
        // 2. Two stakers – register hotkeys on the subnet
        // --------------------------------------------------
        let (c1, h1) = (U256::from(111), U256::from(211));
        let (c2, h2) = (U256::from(222), U256::from(333));
        register_ok_neuron(netuid, h1, c1, 0);
        register_ok_neuron(netuid, h2, c2, 0);

        // --------------------------------------------------
        // 3. Discover protocol-minimum amount (stake + fee)
        // --------------------------------------------------
        let min_stake_total =
            DefaultMinStake::<Test>::get().saturating_add(DefaultStakingFee::<Test>::get());

        // target α-ratio 30 : 70
        let s1 = 3 * min_stake_total;
        let s2 = 7 * min_stake_total;

        // --------------------------------------------------
        // 4. Fund coldkeys sufficiently, then stake via extrinsic
        // --------------------------------------------------
        SubtensorModule::add_balance_to_coldkey_account(&c1, s1 + 50_000);
        SubtensorModule::add_balance_to_coldkey_account(&c2, s2 + 50_000);

        assert_ok!(SubtensorModule::do_add_stake(
            RuntimeOrigin::signed(c1),
            h1,
            netuid,
            s1
        ));
        assert_ok!(SubtensorModule::do_add_stake(
            RuntimeOrigin::signed(c2),
            h2,
            netuid,
            s2
        ));

        // --------------------------------------------------
        // 5. α snapshot
        // --------------------------------------------------
        let a1: u128 = Alpha::<Test>::get((h1, c1, netuid)).saturating_to_num();
        let a2: u128 = Alpha::<Test>::get((h2, c2, netuid)).saturating_to_num();
        let atotal = a1 + a2;

        // --------------------------------------------------
        // 6. TAO pot + subnet lock
        // --------------------------------------------------
        let tao_pot: u64 = 10_000;
        SubnetTAO::<Test>::insert(netuid, tao_pot);
        SubtensorModule::set_subnet_locked_balance(netuid, 5_000);
        Emission::<Test>::insert(netuid, Vec::<u64>::new());

        // --------------------------------------------------
        // 7. Balances before distribution
        // --------------------------------------------------
        let b1 = SubtensorModule::get_coldkey_balance(&c1);
        let b2 = SubtensorModule::get_coldkey_balance(&c2);
        let bo = SubtensorModule::get_coldkey_balance(&owner_cold);

        // --------------------------------------------------
        // 8. Execute payout logic
        // --------------------------------------------------
        assert_ok!(SubtensorModule::destroy_alpha_in_out_stakes(netuid));

        // --------------------------------------------------
        // 9. Expected shares
        // --------------------------------------------------
        let share1: u64 = (tao_pot as u128 * a1 / atotal) as u64;
        let share2: u64 = tao_pot - share1;

        // --------------------------------------------------
        // 10. Assertions
        // --------------------------------------------------
        assert_eq!(SubtensorModule::get_coldkey_balance(&c1), b1 + share1);
        assert_eq!(SubtensorModule::get_coldkey_balance(&c2), b2 + share2);
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&owner_cold),
            bo + 5_000
        );
        assert!(Alpha::<Test>::iter().next().is_none());
    });
}

#[test]
fn destroy_alpha_out_many_stakers_complex_distribution() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(1_000);
        let owner_hot = U256::from(2_000);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        SubtensorModule::set_max_registrations_per_block(netuid, 1000u16);
        SubtensorModule::set_target_registrations_per_interval(netuid, 1000u16);

        let min_total =
            DefaultMinStake::<Test>::get().saturating_add(DefaultStakingFee::<Test>::get());

        const N: usize = 20;
        let mut cold = [U256::zero(); N];
        let mut hot = [U256::zero(); N];
        let mut stake = [0u64; N];

        for i in 0..N {
            cold[i] = U256::from(10_000 + 2 * i as u32);
            hot[i] = U256::from(10_001 + 2 * i as u32);
            stake[i] = (i as u64 + 1) * min_total;

            register_ok_neuron(netuid, hot[i], cold[i], 0);
            SubtensorModule::add_balance_to_coldkey_account(&cold[i], stake[i] + 100_000);

            assert_ok!(SubtensorModule::do_add_stake(
                RuntimeOrigin::signed(cold[i]),
                hot[i],
                netuid,
                stake[i]
            ));
        }

        let mut alpha = [0u128; N];
        let mut a_sum: u128 = 0;
        for i in 0..N {
            alpha[i] = Alpha::<Test>::get((hot[i], cold[i], netuid)).saturating_to_num();
            a_sum += alpha[i];
        }

        let tao_pot: u64 = 123_456;
        let lock: u64 = 30_000;

        SubnetTAO::<Test>::insert(netuid, tao_pot);
        SubtensorModule::set_subnet_locked_balance(netuid, lock);

        // prior emissions (owner already earned some)
        Emission::<Test>::insert(netuid, vec![1_000u64, 2_000, 1_500]);

        // owner-cut = 50 % exactly
        SubnetOwnerCut::<Test>::put(32_768);

        let mut before = [0u64; N];
        for i in 0..N {
            before[i] = SubtensorModule::get_coldkey_balance(&cold[i]);
        }
        let owner_before = SubtensorModule::get_coldkey_balance(&owner_cold);

        let owner_em: u64 = (4_500u128 * 32_768u128 / 65_535u128) as u64;
        let expected_refund = lock.saturating_sub(owner_em);

        // Compute expected shares per pallet algorithm
        let mut share = [0u64; N];
        let mut rem = [0u128; N];
        let mut paid: u128 = 0;

        for i in 0..N {
            let prod = tao_pot as u128 * alpha[i];
            share[i] = (prod / a_sum) as u64;
            rem[i] = prod % a_sum;
            paid += share[i] as u128;
        }
        let leftover = tao_pot as u128 - paid;
        // distribute +1 Tao to stakers with largest remainders
        let mut idx: Vec<_> = (0..N).collect();
        idx.sort_by_key(|i| std::cmp::Reverse(rem[*i]));
        for i in 0..leftover as usize {
            share[idx[i]] += 1;
        }

        assert_ok!(SubtensorModule::destroy_alpha_in_out_stakes(netuid));

        // Assertions
        for i in 0..N {
            assert_eq!(
                SubtensorModule::get_coldkey_balance(&cold[i]),
                before[i] + share[i],
                "staker {} incorrect payout",
                i + 1
            );
        }
        // b) owner refund is correct
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&owner_cold),
            owner_before + expected_refund
        );
        // c) α cleared and counters reset
        assert!(Alpha::<Test>::iter().next().is_none());
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid), 0);
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid), 0);
        assert_eq!(SubtensorModule::get_subnet_locked_balance(netuid), 0);
    });
}

#[test]
fn prune_none_with_no_networks() {
    new_test_ext(0).execute_with(|| {
        assert_eq!(SubtensorModule::get_network_to_prune(), None);
    });
}

#[test]
fn prune_none_when_all_networks_immune() {
    new_test_ext(0).execute_with(|| {
        // two fresh networks → still inside immunity window
        let n1 = add_dynamic_network(&U256::from(2), &U256::from(1));
        let _n2 = add_dynamic_network(&U256::from(4), &U256::from(3));

        // emissions don’t matter while immune
        Emission::<Test>::insert(n1, vec![10u64]);

        assert_eq!(SubtensorModule::get_network_to_prune(), None);
    });
}

#[test]
fn prune_selects_network_with_lowest_emission() {
    new_test_ext(0).execute_with(|| {
        let n1 = add_dynamic_network(&U256::from(20), &U256::from(10));
        let n2 = add_dynamic_network(&U256::from(40), &U256::from(30));

        // make both networks eligible (past immunity)
        let imm = SubtensorModule::get_network_immunity_period();
        System::set_block_number(imm + 10);

        // n1 has lower total emission
        Emission::<Test>::insert(n1, vec![5u64]);
        Emission::<Test>::insert(n2, vec![100u64]);

        assert_eq!(SubtensorModule::get_network_to_prune(), Some(n1));
    });
}

#[test]
fn prune_ignores_immune_network_even_if_lower_emission() {
    new_test_ext(0).execute_with(|| {
        // create mature network n1 first
        let n1 = add_dynamic_network(&U256::from(22), &U256::from(11));

        let imm = SubtensorModule::get_network_immunity_period();
        System::set_block_number(imm + 5); // advance → n1 now mature

        // create second network n2 *inside* immunity
        let n2 = add_dynamic_network(&U256::from(44), &U256::from(33));

        // emissions: n1 bigger, n2 smaller but immune
        Emission::<Test>::insert(n1, vec![50u64]);
        Emission::<Test>::insert(n2, vec![1u64]);

        System::set_block_number(imm + 10); // still immune for n2
        assert_eq!(SubtensorModule::get_network_to_prune(), Some(n1));
    });
}

#[test]
fn prune_tie_on_emission_earlier_registration_wins() {
    new_test_ext(0).execute_with(|| {
        // n1 registered first
        let n1 = add_dynamic_network(&U256::from(66), &U256::from(55));

        // advance 1 block, then register n2 (later timestamp)
        System::set_block_number(1);
        let n2 = add_dynamic_network(&U256::from(88), &U256::from(77));

        // push past immunity for both
        let imm = SubtensorModule::get_network_immunity_period();
        System::set_block_number(imm + 20);

        // identical emissions → tie
        Emission::<Test>::insert(n1, vec![123u64]);
        Emission::<Test>::insert(n2, vec![123u64]);

        // earlier (n1) must be chosen
        assert_eq!(SubtensorModule::get_network_to_prune(), Some(n1));
    });
}

// #[test]
// fn test_schedule_dissolve_network_execution() {
//     new_test_ext(1).execute_with(|| {
//         let block_number: u64 = 0;
//         let netuid: u16 = 2;
//         let tempo: u16 = 13;
//         let hotkey_account_id: U256 = U256::from(1);
//         let coldkey_account_id = U256::from(0); // Neighbour of the beast, har har
//         let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
//             netuid,
//             block_number,
//             129123813,
//             &hotkey_account_id,
//         );

//         //add network
//         add_network(netuid, tempo, 0);

//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
//             netuid,
//             block_number,
//             nonce,
//             work.clone(),
//             hotkey_account_id,
//             coldkey_account_id
//         ));

//         assert!(SubtensorModule::if_subnet_exist(netuid));

//         assert_ok!(SubtensorModule::schedule_dissolve_network(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             netuid
//         ));

//         let current_block = System::block_number();
//         let execution_block = current_block + DissolveNetworkScheduleDuration::<Test>::get();

//         System::assert_last_event(
//             Event::DissolveNetworkScheduled {
//                 account: coldkey_account_id,
//                 netuid,
//                 execution_block,
//             }
//             .into(),
//         );

//         run_to_block(execution_block);
//         assert!(!SubtensorModule::if_subnet_exist(netuid));
//     })
// }

// #[test]
// fn test_non_owner_schedule_dissolve_network_execution() {
//     new_test_ext(1).execute_with(|| {
//         let block_number: u64 = 0;
//         let netuid: u16 = 2;
//         let tempo: u16 = 13;
//         let hotkey_account_id: U256 = U256::from(1);
//         let coldkey_account_id = U256::from(0); // Neighbour of the beast, har har
//         let non_network_owner_account_id = U256::from(2); //
//         let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
//             netuid,
//             block_number,
//             129123813,
//             &hotkey_account_id,
//         );

//         //add network
//         add_network(netuid, tempo, 0);

//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
//             netuid,
//             block_number,
//             nonce,
//             work.clone(),
//             hotkey_account_id,
//             coldkey_account_id
//         ));

//         assert!(SubtensorModule::if_subnet_exist(netuid));

//         assert_ok!(SubtensorModule::schedule_dissolve_network(
//             <<Test as Config>::RuntimeOrigin>::signed(non_network_owner_account_id),
//             netuid
//         ));

//         let current_block = System::block_number();
//         let execution_block = current_block + DissolveNetworkScheduleDuration::<Test>::get();

//         System::assert_last_event(
//             Event::DissolveNetworkScheduled {
//                 account: non_network_owner_account_id,
//                 netuid,
//                 execution_block,
//             }
//             .into(),
//         );

//         run_to_block(execution_block);
//         // network exists since the caller is no the network owner
//         assert!(SubtensorModule::if_subnet_exist(netuid));
//     })
// }

// #[test]
// fn test_new_owner_schedule_dissolve_network_execution() {
//     new_test_ext(1).execute_with(|| {
//         let block_number: u64 = 0;
//         let netuid: u16 = 2;
//         let tempo: u16 = 13;
//         let hotkey_account_id: U256 = U256::from(1);
//         let coldkey_account_id = U256::from(0); // Neighbour of the beast, har har
//         let new_network_owner_account_id = U256::from(2); //
//         let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
//             netuid,
//             block_number,
//             129123813,
//             &hotkey_account_id,
//         );

//         //add network
//         add_network(netuid, tempo, 0);

//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
//             netuid,
//             block_number,
//             nonce,
//             work.clone(),
//             hotkey_account_id,
//             coldkey_account_id
//         ));

//         assert!(SubtensorModule::if_subnet_exist(netuid));

//         // the account is not network owner when schedule the call
//         assert_ok!(SubtensorModule::schedule_dissolve_network(
//             <<Test as Config>::RuntimeOrigin>::signed(new_network_owner_account_id),
//             netuid
//         ));

//         let current_block = System::block_number();
//         let execution_block = current_block + DissolveNetworkScheduleDuration::<Test>::get();

//         System::assert_last_event(
//             Event::DissolveNetworkScheduled {
//                 account: new_network_owner_account_id,
//                 netuid,
//                 execution_block,
//             }
//             .into(),
//         );
//         run_to_block(current_block + 1);
//         // become network owner after call scheduled
//         crate::SubnetOwner::<Test>::insert(netuid, new_network_owner_account_id);

//         run_to_block(execution_block);
//         // network exists since the caller is no the network owner
//         assert!(!SubtensorModule::if_subnet_exist(netuid));
//     })
// }

// #[test]
// fn test_schedule_dissolve_network_execution_with_coldkey_swap() {
//     new_test_ext(1).execute_with(|| {
//         let block_number: u64 = 0;
//         let netuid: u16 = 2;
//         let tempo: u16 = 13;
//         let hotkey_account_id: U256 = U256::from(1);
//         let coldkey_account_id = U256::from(0); // Neighbour of the beast, har har
//         let new_network_owner_account_id = U256::from(2); //

//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1000000000000000);

//         let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
//             netuid,
//             block_number,
//             129123813,
//             &hotkey_account_id,
//         );

//         //add network
//         add_network(netuid, tempo, 0);

//         assert_ok!(SubtensorModule::register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
//             netuid,
//             block_number,
//             nonce,
//             work.clone(),
//             hotkey_account_id,
//             coldkey_account_id
//         ));

//         assert!(SubtensorModule::if_subnet_exist(netuid));

//         // the account is not network owner when schedule the call
//         assert_ok!(SubtensorModule::schedule_swap_coldkey(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             new_network_owner_account_id
//         ));

//         let current_block = System::block_number();
//         let execution_block = current_block + ColdkeySwapScheduleDuration::<Test>::get();

//         run_to_block(execution_block - 1);

//         // the account is not network owner when schedule the call
//         assert_ok!(SubtensorModule::schedule_dissolve_network(
//             <<Test as Config>::RuntimeOrigin>::signed(new_network_owner_account_id),
//             netuid
//         ));

//         System::assert_last_event(
//             Event::DissolveNetworkScheduled {
//                 account: new_network_owner_account_id,
//                 netuid,
//                 execution_block: DissolveNetworkScheduleDuration::<Test>::get() + execution_block
//                     - 1,
//             }
//             .into(),
//         );

//         run_to_block(execution_block);
//         assert_eq!(
//             crate::SubnetOwner::<Test>::get(netuid),
//             new_network_owner_account_id
//         );

//         let current_block = System::block_number();
//         let execution_block = current_block + DissolveNetworkScheduleDuration::<Test>::get();

//         run_to_block(execution_block);
//         // network exists since the caller is no the network owner
//         assert!(!SubtensorModule::if_subnet_exist(netuid));
//     })
// }

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::networks::test_register_subnet_low_lock_cost --exact --show-output --nocapture
#[test]
fn test_register_subnet_low_lock_cost() {
    new_test_ext(1).execute_with(|| {
        NetworkMinLockCost::<Test>::set(1_000);
        NetworkLastLockCost::<Test>::set(1_000);

        // Make sure lock cost is lower than 100 TAO
        let lock_cost = SubtensorModule::get_network_lock_cost();
        assert!(lock_cost < 100_000_000_000);

        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        assert!(SubtensorModule::if_subnet_exist(netuid));

        // Ensure that both Subnet TAO and Subnet Alpha In equal to (actual) lock_cost
        assert_eq!(SubnetTAO::<Test>::get(netuid), lock_cost,);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid), lock_cost,);
    })
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::networks::test_register_subnet_high_lock_cost --exact --show-output --nocapture
#[test]
fn test_register_subnet_high_lock_cost() {
    new_test_ext(1).execute_with(|| {
        let lock_cost: u64 = 1_000_000_000_000;
        NetworkMinLockCost::<Test>::set(lock_cost);
        NetworkLastLockCost::<Test>::set(lock_cost);

        // Make sure lock cost is higher than 100 TAO
        let lock_cost = SubtensorModule::get_network_lock_cost();
        assert!(lock_cost >= 1_000_000_000_000);

        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        assert!(SubtensorModule::if_subnet_exist(netuid));

        // Ensure that both Subnet TAO and Subnet Alpha In equal to 100 TAO
        assert_eq!(SubnetTAO::<Test>::get(netuid), lock_cost);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid), lock_cost);
    })
}

#[test]
fn test_tempo_greater_than_weight_set_rate_limit() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_hotkey = U256::from(1);
        let subnet_owner_coldkey = U256::from(2);

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Get tempo
        let tempo = SubtensorModule::get_tempo(netuid);

        let weights_set_rate_limit = SubtensorModule::get_weights_set_rate_limit(netuid);

        assert!(tempo as u64 >= weights_set_rate_limit);
    })
}
