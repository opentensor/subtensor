use crate::{
    mock::*, BeaconConfig, BeaconConfigurationPayload, BeaconInfoResponse, Call, DrandResponseBody,
    Error, Event, Pulse, PulsePayload, Pulses,
};
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::{InvalidTransaction, TransactionSource},
};
use sp_runtime::{
    offchain::{
        testing::{PendingRequest, TestOffchainExt},
        OffchainWorkerExt,
    },
    traits::ValidateUnsigned,
};

pub const DRAND_RESPONSE: &str = "{\"round\":9683710,\"randomness\":\"87f03ef5f62885390defedf60d5b8132b4dc2115b1efc6e99d166a37ab2f3a02\",\"signature\":\"b0a8b04e009cf72534321aca0f50048da596a3feec1172a0244d9a4a623a3123d0402da79854d4c705e94bc73224c342\"}";
pub const QUICKNET_INFO_RESPONSE: &str = "{\"public_key\":\"83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a\",\"period\":3,\"genesis_time\":1692803367,\"hash\":\"52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971\",\"groupHash\":\"f477d5c89f21a17c863a7f937c6a6d15859414d2be09cd448d4279af331c5d3e\",\"schemeID\":\"bls-unchained-g1-rfc9380\",\"metadata\":{\"beaconID\":\"quicknet\"}}";

#[test]
fn can_fail_submit_valid_pulse_when_beacon_config_missing() {
    new_test_ext().execute_with(|| {
        let u_p: DrandResponseBody = serde_json::from_str(DRAND_RESPONSE).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();

        let alice = sp_keyring::Sr25519Keyring::Alice;

        let block_number = 1;
        System::set_block_number(block_number);

        let pulse_payload = PulsePayload {
            block_number,
            pulse: p.clone(),
            public: alice.public(),
        };

        // The signature doesn't really matter here because the signature is validated in the
        // transaction validation phase not in the dispatchable itself.
        let signature = None;

        // Dispatch an unsigned extrinsic.
        assert_ok!(Drand::write_pulse(
            RuntimeOrigin::none(),
            pulse_payload,
            signature
        ));
        // Read pallet storage and assert an expected result.
        let pulse = Pulses::<Test>::get(1);
        assert_eq!(pulse, None);
    });
}

#[test]
fn can_submit_valid_pulse_when_beacon_config_exists() {
    new_test_ext().execute_with(|| {
        let u_p: DrandResponseBody = serde_json::from_str(DRAND_RESPONSE).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();

        let alice = sp_keyring::Sr25519Keyring::Alice;
        let block_number = 1;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(QUICKNET_INFO_RESPONSE).unwrap();
        let config_payload = BeaconConfigurationPayload {
            block_number,
            config: info.clone().try_into_beacon_config().unwrap(),
            public: alice.public(),
        };

        // The signature doesn't really matter here because the signature is validated in the
        // transaction validation phase not in the dispatchable itself.
        let signature = None;
        assert_ok!(Drand::set_beacon_config(
            RuntimeOrigin::none(),
            config_payload,
            signature
        ));

        let pulse_payload = PulsePayload {
            pulse: p.clone(),
            block_number,
            public: alice.public(),
        };

        // Dispatch an unsigned extrinsic.
        assert_ok!(Drand::write_pulse(
            RuntimeOrigin::none(),
            pulse_payload,
            signature
        ));

        // Read pallet storage and assert an expected result.
        let pulse = Pulses::<Test>::get(1);
        assert!(pulse.is_some());
        assert_eq!(pulse, Some(p));
        // Assert that the correct event was deposited
        System::assert_last_event(Event::NewPulse { round: 9683710 }.into());
    });
}

#[test]
fn rejects_invalid_pulse_bad_signature() {
    new_test_ext().execute_with(|| {
		let alice = sp_keyring::Sr25519Keyring::Alice;
		let block_number = 1;
		System::set_block_number(block_number);

		// Set the beacon config
		let info: BeaconInfoResponse = serde_json::from_str(QUICKNET_INFO_RESPONSE).unwrap();
		let config_payload = BeaconConfigurationPayload {
			block_number,
			config: info.clone().try_into_beacon_config().unwrap(),
			public: alice.public(),
		};
		// The signature doesn't really matter here because the signature is validated in the
		// transaction validation phase not in the dispatchable itself.
		let signature = None;
		assert_ok!(Drand::set_beacon_config(RuntimeOrigin::none(), config_payload, signature));

		// Get a bad pulse
		let bad_http_response = "{\"round\":9683710,\"randomness\":\"87f03ef5f62885390defedf60d5b8132b4dc2115b1efc6e99d166a37ab2f3a02\",\"signature\":\"b0a8b04e009cf72534321aca0f50048da596a3feec1172a0244d9a4a623a3123d0402da79854d4c705e94bc73224c341\"}";
		let u_p: DrandResponseBody = serde_json::from_str(bad_http_response).unwrap();
		let p: Pulse = u_p.try_into_pulse().unwrap();

		// Set the pulse
		let pulse_payload = PulsePayload {
			pulse: p.clone(),
			block_number,
			public: alice.public(),
		};
		let signature = alice.sign(&pulse_payload.encode());
		assert_noop!(Drand::write_pulse(
			RuntimeOrigin::none(),
			pulse_payload,
			Some(signature)),
			Error::<Test>::PulseVerificationError
		);
		let pulse = Pulses::<Test>::get(1);
		assert!(pulse.is_none());
	});
}

#[test]
fn rejects_pulses_with_non_incremental_round_numbers() {
    new_test_ext().execute_with(|| {
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(QUICKNET_INFO_RESPONSE).unwrap();
        let config_payload = BeaconConfigurationPayload {
            block_number,
            config: info.clone().try_into_beacon_config().unwrap(),
            public: alice.public(),
        };
        // The signature doesn't really matter here because the signature is validated in the
        // transaction validation phase not in the dispatchable itself.
        let signature = None;
        assert_ok!(Drand::set_beacon_config(
            RuntimeOrigin::none(),
            config_payload,
            signature
        ));

        let u_p: DrandResponseBody = serde_json::from_str(DRAND_RESPONSE).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();
        let pulse_payload = PulsePayload {
            pulse: p.clone(),
            block_number,
            public: alice.public(),
        };

        // Dispatch an unsigned extrinsic.
        assert_ok!(Drand::write_pulse(
            RuntimeOrigin::none(),
            pulse_payload.clone(),
            signature
        ));
        let pulse = Pulses::<Test>::get(1);
        assert!(pulse.is_some());

        System::assert_last_event(Event::NewPulse { round: 9683710 }.into());
        System::set_block_number(2);

        assert_noop!(
            Drand::write_pulse(RuntimeOrigin::none(), pulse_payload, signature),
            Error::<Test>::InvalidRoundNumber,
        );
    });
}

#[test]
fn root_cannot_submit_beacon_info() {
    new_test_ext().execute_with(|| {
        assert!(BeaconConfig::<Test>::get().is_none());
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(QUICKNET_INFO_RESPONSE).unwrap();
        let config_payload = BeaconConfigurationPayload {
            block_number,
            config: info.clone().try_into_beacon_config().unwrap(),
            public: alice.public(),
        };
        // The signature doesn't really matter here because the signature is validated in the
        // transaction validation phase not in the dispatchable itself.
        let signature = None;
        assert_noop!(
            Drand::set_beacon_config(RuntimeOrigin::root(), config_payload, signature),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn signed_cannot_submit_beacon_info() {
    new_test_ext().execute_with(|| {
        assert!(BeaconConfig::<Test>::get().is_none());
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(QUICKNET_INFO_RESPONSE).unwrap();
        let config_payload = BeaconConfigurationPayload {
            block_number,
            config: info.clone().try_into_beacon_config().unwrap(),
            public: alice.public(),
        };
        // The signature doesn't really matter here because the signature is validated in the
        // transaction validation phase not in the dispatchable itself.
        let signature = None;
        // Dispatch a signed extrinsic
        assert_noop!(
            Drand::set_beacon_config(
                RuntimeOrigin::signed(alice.public().clone()),
                config_payload,
                signature
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_validate_unsigned_write_pulse() {
    new_test_ext().execute_with(|| {
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);
        let payload = PulsePayload {
            block_number,
            pulse: Default::default(),
            public: alice.public(),
        };
        let signature = alice.sign(&payload.encode());

        let call = Call::write_pulse {
            pulse_payload: payload.clone(),
            signature: Some(signature),
        };

        let source = TransactionSource::External;
        let validity = Drand::validate_unsigned(source, &call);

        assert_ok!(validity);
    });
}

#[test]
fn test_not_validate_unsigned_write_pulse_with_bad_proof() {
    new_test_ext().execute_with(|| {
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);
        let payload = PulsePayload {
            block_number,
            pulse: Default::default(),
            public: alice.public(),
        };

        // bad signature
        let signature = <Test as frame_system::offchain::SigningTypes>::Signature::default();
        let call = Call::write_pulse {
            pulse_payload: payload.clone(),
            signature: Some(signature),
        };

        let source = TransactionSource::External;
        let validity = Drand::validate_unsigned(source, &call);

        assert_noop!(validity, InvalidTransaction::BadProof);
    });
}

#[test]
fn test_not_validate_unsigned_write_pulse_with_no_payload_signature() {
    new_test_ext().execute_with(|| {
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);
        let payload = PulsePayload {
            block_number,
            pulse: Default::default(),
            public: alice.public(),
        };

        // no signature
        let signature = None;
        let call = Call::write_pulse {
            pulse_payload: payload.clone(),
            signature,
        };

        let source = TransactionSource::External;
        let validity = Drand::validate_unsigned(source, &call);

        assert_noop!(validity, InvalidTransaction::BadSigner);
    });
}

#[test]
#[ignore]
fn test_validate_unsigned_write_pulse_by_non_authority() {
    // TODO: https://github.com/ideal-lab5/pallet-drand/issues/3
    todo!(
        "the transaction should be validated even if the signer of the payload is not an authority"
    );
}

#[test]
#[ignore]
fn test_not_validate_unsigned_set_beacon_config_by_non_autority() {
    // TODO: https://github.com/ideal-lab5/pallet-drand/issues/3
    todo!(
        "the transaction should not be validated if the signer of the payload is not an authority"
    );
}

#[test]
fn can_execute_and_handle_valid_http_responses() {
    let (offchain, state) = TestOffchainExt::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));

    {
        let mut state = state.write();
        state.expect_request(PendingRequest {
			method: "GET".into(),
			uri: "https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/info".into(),
			response: Some(QUICKNET_INFO_RESPONSE.as_bytes().to_vec()),
			sent: true,
			..Default::default()
		});
        state.expect_request(PendingRequest {
			method: "GET".into(),
			uri: "https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/public/latest".into(),
			response: Some(DRAND_RESPONSE.as_bytes().to_vec()),
			sent: true,
			..Default::default()
		});
    }

    t.execute_with(|| {
        let actual_config = Drand::fetch_drand_chain_info().unwrap();
        assert_eq!(actual_config, QUICKNET_INFO_RESPONSE);

        let actual_pulse = Drand::fetch_drand().unwrap();
        assert_eq!(actual_pulse, DRAND_RESPONSE);
    });
}
