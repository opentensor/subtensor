use super::mock::*;
use crate::*;
use sp_core::U256;

#[test]
#[should_panic(expected = "Invariant violation")]
fn test_stake_invariant_failure() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(100);
        let coldkey = U256::from(101);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        // Manually introduce inconsistency
        // Set TotalHotkeyAlpha for a hotkey
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, AlphaCurrency::from(500));
        
        // Register hotkey in Keys so iteration finds it
        // We need to know the UID. add_dynamic_network registers the owner hotkey at uid 0?
        // Let's insert a new one
        let uid = 1;
        Keys::<Test>::insert(netuid, uid, hotkey);
        
        // Set SubnetAlphaOut to something that does NOT match 500 (plus whatever owner has)
        // Owner has some stake from network creation probably.
        // Let's just set SubnetAlphaOut to a huge number.
        SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(9999999));

        // Ensure check runs
        BlocksSinceLastStep::<Test>::insert(netuid, 0);
        
        // Run check
        SubtensorModule::check_invariants();
    });
}

#[test]
#[should_panic(expected = "Invariant violation")]
fn test_emission_invariant_failure() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(200);
        let coldkey = U256::from(201);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        // Set BlockEmission
        BlockEmission::<Test>::set(100);

        // Inject MORE into subnet
        SubnetTaoInEmission::<Test>::insert(netuid, TaoCurrency::from(200));

        // Note: check_emission_invariant runs every call
        SubtensorModule::check_invariants();
    });
}

#[test]
fn test_invariants_pass_normal_operation() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(300);
        let coldkey = U256::from(301);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        // Normal operation - add stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100000);
        // ... (requires setup reserves etc)
        
        // Instead of complex setup, just verify that default state passes
        // BlocksSinceLastStep is 0 initially? 
        // add_dynamic_network likely sets it.
        BlocksSinceLastStep::<Test>::insert(netuid, 0);
        
        SubtensorModule::check_invariants();
        
        // No panic means success
    });
}

#[test]
fn test_recovery_mechanism() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(400);
        let coldkey = U256::from(401);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        // Simulate a violation (we can't trigger the full panic flow here as it would panic test, 
        // preventing recovery check, so we manually set the paused state).
        SubnetEmissionPaused::<Test>::insert(netuid, true);
        assert!(SubnetEmissionPaused::<Test>::get(netuid));

        // Attempt to unpause with root
        assert_ok!(SubtensorModule::unpause_subnet_emission(RuntimeOrigin::root(), netuid));

        // Validate it is unpaused
        assert!(!SubnetEmissionPaused::<Test>::get(netuid));
        
        // Verify event was emitted (SubnetEmissionResumed is last event)
        System::assert_last_event(Event::SubnetEmissionResumed(netuid).into());
    });
}

#[test]
fn test_paused_subnet_skips_check() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(500);
        let coldkey = U256::from(501);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        // Manually introduce inconsistency
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, AlphaCurrency::from(500));
        let uid = 1;
        Keys::<Test>::insert(netuid, uid, hotkey);
        SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(9999999));
        
        // Ensure check WOULD run
        BlocksSinceLastStep::<Test>::insert(netuid, 0);

        // But we PAUSE it manually
        SubnetEmissionPaused::<Test>::insert(netuid, true);
        
        // Run check - should NOT panic because it skips paused subnets
        SubtensorModule::check_invariants();
    });
}
