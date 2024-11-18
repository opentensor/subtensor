/*
 * Copyright 2024 by Ideal Labs, LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::{
    mock::*, BeaconConfig, BeaconConfigurationPayload, BeaconInfoResponse, Call, DrandResponseBody,
    Error, Pulse, Pulses, PulsesPayload,
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

// The round number used to collect drand pulses
pub const ROUND_NUMBER: u64 = 1000;

// Quicknet parameters
#[cfg(not(feature = "mainnet"))]
pub const DRAND_PULSE: &str = "{\"round\":1000,\"randomness\":\"fe290beca10872ef2fb164d2aa4442de4566183ec51c56ff3cd603d930e54fdd\",\"signature\":\"b44679b9a59af2ec876b1a6b1ad52ea9b1615fc3982b19576350f93447cb1125e342b73a8dd2bacbe47e4b6b63ed5e39\"}";
#[cfg(not(feature = "mainnet"))]
pub const DRAND_INFO_RESPONSE: &str = "{\"public_key\":\"83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a\",\"period\":3,\"genesis_time\":1692803367,\"hash\":\"52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971\",\"groupHash\":\"f477d5c89f21a17c863a7f937c6a6d15859414d2be09cd448d4279af331c5d3e\",\"schemeID\":\"bls-unchained-g1-rfc9380\",\"metadata\":{\"beaconID\":\"quicknet\"}}";

// Mainnet parameters
#[cfg(feature = "mainnet")]
pub const DRAND_PULSE: &str = "{\"round\":1000,\"randomness\":\"a40d3e0e7e3c71f28b7da2fd339f47f0bcf10910309f5253d7c323ec8cea3212\",\"signature\":\"99bf96de133c3d3937293cfca10c8152b18ab2d034ccecf115658db324d2edc00a16a2044cd04a8a38e2a307e5ecff3511315be8d282079faf24098f283e0ed2c199663b334d2e84c55c032fe469b212c5c2087ebb83a5b25155c3283f5b79ac\",\"previous_signature\":\"af0d93299a363735fe847f5ea241442c65843dc1bd3a7b79646b3b10072e908bf034d35cd69d378e3341f139100cd4cd03030399864ef8803a5a4f5e64fccc20bbae36d1ca22a6ddc43d2630c41105e90598fab11e5c7456df3925d4b577b113\"}";
#[cfg(feature = "mainnet")]
pub const DRAND_INFO_RESPONSE: &str = "{\"public_key\":\"868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31\",\"period\":30,\"genesis_time\":1595431050,\"hash\":\"8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce\",\"groupHash\":\"176f93498eac9ca337150b46d21dd58673ea4e3581185f869672e59fa4cb390a\",\"schemeID\":\"pedersen-bls-chained\",\"metadata\":{\"beaconID\":\"default\"}}";

#[test]
fn it_fails_to_submit_valid_pulse_when_beacon_config_missing() {
    new_test_ext().execute_with(|| {
        let u_p: DrandResponseBody = serde_json::from_str(DRAND_PULSE).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();

        let alice = sp_keyring::Sr25519Keyring::Alice;

        let block_number = 1;
        System::set_block_number(block_number);

        let pulses_payload = PulsesPayload {
            block_number,
            pulses: vec![p.clone()],
            public: alice.public(),
        };

        // The signature doesn't really matter here because the signature is validated in the
        // transaction validation phase not in the dispatchable itself.
        let signature = None;

        // Dispatch an unsigned extrinsic and expect it to fail.
        assert_noop!(
            Drand::write_pulse(RuntimeOrigin::none(), pulses_payload, signature),
            Error::<Test>::NoneValue
        );

        let pulse = Pulses::<Test>::get(p.round);
        assert_eq!(pulse, None);
    });
}

#[test]
fn it_can_submit_valid_pulse_when_beacon_config_exists() {
    new_test_ext().execute_with(|| {
        let u_p: DrandResponseBody = serde_json::from_str(DRAND_PULSE).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();

        let alice = sp_keyring::Sr25519Keyring::Alice;
        let block_number = 1;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(DRAND_INFO_RESPONSE).unwrap();
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

        let pulses_payload = PulsesPayload {
            pulses: vec![p.clone()],
            block_number,
            public: alice.public(),
        };

        // Dispatch an unsigned extrinsic.
        assert_ok!(Drand::write_pulse(
            RuntimeOrigin::none(),
            pulses_payload,
            signature
        ));

        // Read pallet storage and assert an expected result.
        let pulse = Pulses::<Test>::get(ROUND_NUMBER);
        assert!(pulse.is_some());
        assert_eq!(pulse, Some(p));
    });
}

#[test]
fn it_rejects_invalid_pulse_bad_signature() {
    new_test_ext().execute_with(|| {
        let alice = sp_keyring::Sr25519Keyring::Alice;
        let block_number = 1;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(DRAND_INFO_RESPONSE).unwrap();
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

        // Get a bad pulse
        #[cfg(not(feature = "mainnet"))]
        let bad_http_response = "{\"round\":1000,\"randomness\":\"87f03ef5f62885390defedf60d5b8132b4dc2115b1efc6e99d166a37ab2f3a02\",\"signature\":\"b0a8b04e009cf72534321aca0f50048da596a3feec1172a0244d9a4a623a3123d0402da79854d4c705e94bc73224c341\"}";
        #[cfg(feature = "mainnet")]
        let bad_http_response = "{\"round\":1000,\"randomness\":\"87f03ef5f62885390defedf60d5b8132b4dc2115b1efc6e99d166a37ab2f3a02\",\"signature\":\"b0a8b04e009cf72534321aca0f50048da596a3feec1172a0244d9a4a623a3123d0402da79854d4c705e94bc73224c341\", \"previous_signature\":\"af0d93299a363735fe847f5ea241442c65843dc1bd3a7b79646b3b10072e908bf034d35cd69d378e3341f139100cd4cd03030399864ef8803a5a4f5e64fccc20bbae36d1ca22a6ddc43d2630c41105e90598fab11e5c7456df3925d4b577b113\"}";
        let u_p: DrandResponseBody = serde_json::from_str(bad_http_response).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();

        // Set the pulse
        let pulses_payload = PulsesPayload {
            pulses: vec![p.clone()],
            block_number,
            public: alice.public(),
        };
        let signature = alice.sign(&pulses_payload.encode());
        assert_noop!(
            Drand::write_pulse(RuntimeOrigin::none(), pulses_payload, Some(signature)),
            Error::<Test>::PulseVerificationError
        );
        let pulse = Pulses::<Test>::get(ROUND_NUMBER);
        assert!(pulse.is_none());
    });
}

#[test]
fn it_rejects_pulses_with_non_incremental_round_numbers() {
    new_test_ext().execute_with(|| {
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(DRAND_INFO_RESPONSE).unwrap();
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

        let u_p: DrandResponseBody = serde_json::from_str(DRAND_PULSE).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();
        let pulses_payload = PulsesPayload {
            pulses: vec![p.clone()],
            block_number,
            public: alice.public(),
        };

        // Dispatch an unsigned extrinsic.
        assert_ok!(Drand::write_pulse(
            RuntimeOrigin::none(),
            pulses_payload.clone(),
            signature
        ));
        let pulse = Pulses::<Test>::get(ROUND_NUMBER);
        assert!(pulse.is_some());

        System::set_block_number(2);

        // Attempt to submit the same pulse again, which should fail
        assert_noop!(
            Drand::write_pulse(RuntimeOrigin::none(), pulses_payload, signature),
            Error::<Test>::InvalidRoundNumber,
        );
    });
}

#[test]
fn it_blocks_root_from_submit_beacon_info() {
    new_test_ext().execute_with(|| {
        assert!(BeaconConfig::<Test>::get().is_none());
        let block_number = 1;
        let alice = sp_keyring::Sr25519Keyring::Alice;
        System::set_block_number(block_number);

        // Set the beacon config
        let info: BeaconInfoResponse = serde_json::from_str(DRAND_INFO_RESPONSE).unwrap();
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
        let info: BeaconInfoResponse = serde_json::from_str(DRAND_INFO_RESPONSE).unwrap();
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
                RuntimeOrigin::signed(alice.public()),
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
        let pulses_payload = PulsesPayload {
            block_number,
            pulses: vec![],
            public: alice.public(),
        };
        let signature = alice.sign(&pulses_payload.encode());

        let call = Call::write_pulse {
            pulses_payload: pulses_payload.clone(),
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
        let pulses_payload = PulsesPayload {
            block_number,
            pulses: vec![],
            public: alice.public(),
        };

        // Bad signature
        let signature = <Test as frame_system::offchain::SigningTypes>::Signature::default();
        let call = Call::write_pulse {
            pulses_payload: pulses_payload.clone(),
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
        let pulses_payload = PulsesPayload {
            block_number,
            pulses: vec![],
            public: alice.public(),
        };

        // No signature
        let signature = None;
        let call = Call::write_pulse {
            pulses_payload: pulses_payload.clone(),
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
fn test_not_validate_unsigned_set_beacon_config_by_non_authority() {
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
            uri: "https://drand.cloudflare.com/8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce/info".into(),
            response: Some(DRAND_INFO_RESPONSE.as_bytes().to_vec()),
            sent: true,
            ..Default::default()
        });
        state.expect_request(PendingRequest {
            method: "GET".into(),
            uri: "https://drand.cloudflare.com/8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce/public/latest".into(),
            response: Some(DRAND_PULSE.as_bytes().to_vec()),
            sent: true,
            ..Default::default()
        });
    }

    t.execute_with(|| {
        let actual_config = Drand::fetch_drand_chain_info().unwrap();
        assert_eq!(actual_config, DRAND_INFO_RESPONSE);

        let actual_pulse = Drand::fetch_drand_latest().unwrap();
        assert_eq!(actual_pulse, DRAND_PULSE);
    });
}
