mod mock;
use codec::Compact;
use mock::*;

use pallet_subtensor::IPInfoOf;
use sp_core::U256;

#[test]
fn test_get_ip_info_hotkey_does_not_exist() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
		let hotkey = U256::from(0);
		let hotkey2 = U256::from(1);

		// Note, no hotkey is in the map
		let ip_info_result = SubtensorModule::get_associated_ip_info(netuid, &hotkey);
		assert_eq!(ip_info_result.is_none(), true);

		// Add IP info for hotkey2
		let ip_info = IPInfoOf {
			ip: Compact::<u128>::from(0),
			ip_type_and_protocol: Compact::<u8>::from(0),
		};
		let ip_info_in_vec = vec![ip_info];
		SubtensorModule::associate_ips_with_hotkey_for_netuid(netuid, hotkey2, 0, ip_info_in_vec.try_into().unwrap());

		// Note: hotkey is still not in the map, but hotkey2 is
		let ip_info_result_2 = SubtensorModule::get_associated_ip_info(netuid, &hotkey);
		assert_eq!(ip_info_result_2.is_none(), true); // Still none
	});
}

#[test]
#[cfg(not(tarpaulin))]
fn test_get_ip_info_some() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
		let hotkey = U256::from(0);
		let hotkey2 = U256::from(1);

		// Note, no hotkey is in the map
		let ip_info_result = SubtensorModule::get_associated_ip_info(netuid, &hotkey);
		assert_eq!(ip_info_result.is_none(), true); // Not in map, so none

		// Add IP info for hotkey
		let ip_info = IPInfoOf {
			ip: Compact::<u128>::from(0),
			ip_type_and_protocol: Compact::<u8>::from(0),
		};
		let ip_info_in_vec = vec![ip_info];
		SubtensorModule::associate_ips_with_hotkey_for_netuid(netuid, hotkey, 0, ip_info_in_vec.try_into().unwrap());

		// Note: hotkey is now in the map
		let ip_info_result_2 = SubtensorModule::get_associated_ip_info(netuid, &hotkey);
		assert_eq!(ip_info_result_2.is_none(), false); // Now in the map, so some

		// Note: hotkey2 is still not in the map
		let ip_info_result_3 = SubtensorModule::get_associated_ip_info(netuid, &hotkey2);
		assert_eq!(ip_info_result_3.is_none(), true); // Still none

		// Add IP info for hotkey2
		let ip_info2 = IPInfoOf {
			ip: Compact::<u128>::from(0),
			ip_type_and_protocol: Compact::<u8>::from(0),
		};
		let ip_info_in_vec2 = vec![ip_info2];
		SubtensorModule::associate_ips_with_hotkey_for_netuid(netuid, hotkey2, 0, ip_info_in_vec2.try_into().unwrap());

		// Note: hotkey2 is now in the map
		let ip_info_result_4 = SubtensorModule::get_associated_ip_info(netuid, &hotkey2);
		assert_eq!(ip_info_result_4.is_none(), false); // Now in the map, so some
	});
}

#[test]
fn test_get_validator_ip_info_for_hotkey_list() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 2;
        let modality: u16 = 2;

		let neuron_count: u16 = 3;

        // add network
        add_network(netuid, tempo, modality);
		// Increase max allowed validators so all neurons can be registered as a validator
		SubtensorModule::set_max_allowed_validators(netuid, neuron_count * 3); // More than neuron count
		// Increase max allowed uids so all neurons can be registered
		SubtensorModule::set_max_allowed_uids(netuid, neuron_count * 3); // More than neuron count

        for index in 0..neuron_count {
            let hotkey = U256::from(0 + index);
            let coldkey = U256::from(0 + index);
            // Register neuron
            let nonce: u64 = 39420842 + index as u64;
            register_ok_neuron(netuid, hotkey, coldkey, nonce);

            // Add IP info for hotkey
            let ip_info = IPInfoOf {
                ip: Compact::<u128>::from(0),
                ip_type_and_protocol: Compact::<u8>::from(0),
            };
            let ip_info_in_vec = vec![ip_info];
            SubtensorModule::associate_ips_with_hotkey_for_netuid(
                netuid,
                hotkey,
                0,
                ip_info_in_vec.try_into().unwrap(),
            );
        }
		// Sanity check, make sure all neurons are registered
		assert_eq!(
			SubtensorModule::get_neurons_lite(netuid).len(),
			neuron_count as usize
		);

		// Step until tempo so we can get vpermits
		step_block(tempo * 2);
		assert_eq!( // Check that all neurons have vpermits
			SubtensorModule::get_neurons_lite(netuid)
				.into_iter()
				.enumerate()
				.filter(|(uid, _n)| SubtensorModule::get_validator_permit_for_uid(netuid, *uid as u16) )
				.count(),
			neuron_count as usize
		);

		// Note: only returns those with VPermit
		let ip_infos = SubtensorModule::get_associated_validator_ip_info_for_subnet(netuid);
		assert_eq!(ip_infos.is_none(), false);
		assert_eq!(ip_infos.unwrap().len(), neuron_count as usize);

    });
}

#[test]
fn test_get_ip_info_empty() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;

        let ip_infos = SubtensorModule::get_associated_validator_ip_info_for_subnet(netuid);
        assert_eq!(ip_infos.is_none(), true);
    });
}
