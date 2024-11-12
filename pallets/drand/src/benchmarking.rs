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

//! Benchmarking setup for pallet-drand
use super::*;

#[allow(unused)]
use crate::Pallet as Drand;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[cfg(not(feature = "mainnet"))]
pub const DRAND_PULSE: &str = "{\"round\":1000,\"randomness\":\"fe290beca10872ef2fb164d2aa4442de4566183ec51c56ff3cd603d930e54fdd\",\"signature\":\"b44679b9a59af2ec876b1a6b1ad52ea9b1615fc3982b19576350f93447cb1125e342b73a8dd2bacbe47e4b6b63ed5e39\"}";
#[cfg(not(feature = "mainnet"))]
pub const DRAND_INFO_RESPONSE: &str = "{\"public_key\":\"83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a\",\"period\":3,\"genesis_time\":1692803367,\"hash\":\"52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971\",\"groupHash\":\"f477d5c89f21a17c863a7f937c6a6d15859414d2be09cd448d4279af331c5d3e\",\"schemeID\":\"bls-unchained-g1-rfc9380\",\"metadata\":{\"beaconID\":\"quicknet\"}}";

// mainnet parameters
#[cfg(feature = "mainnet")]
pub const DRAND_PULSE: &str = "{\"round\":1000,\"randomness\":\"a40d3e0e7e3c71f28b7da2fd339f47f0bcf10910309f5253d7c323ec8cea3212\",\"signature\":\"99bf96de133c3d3937293cfca10c8152b18ab2d034ccecf115658db324d2edc00a16a2044cd04a8a38e2a307e5ecff3511315be8d282079faf24098f283e0ed2c199663b334d2e84c55c032fe469b212c5c2087ebb83a5b25155c3283f5b79ac\",\"previous_signature\":\"af0d93299a363735fe847f5ea241442c65843dc1bd3a7b79646b3b10072e908bf034d35cd69d378e3341f139100cd4cd03030399864ef8803a5a4f5e64fccc20bbae36d1ca22a6ddc43d2630c41105e90598fab11e5c7456df3925d4b577b113\"}";
#[cfg(feature = "mainnet")]
pub const DRAND_INFO_RESPONSE: &str = "{\"public_key\":\"868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31\",\"period\":30,\"genesis_time\":1595431050,\"hash\":\"8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce\",\"groupHash\":\"176f93498eac9ca337150b46d21dd58673ea4e3581185f869672e59fa4cb390a\",\"schemeID\":\"pedersen-bls-chained\",\"metadata\":{\"beaconID\":\"default\"}}";

#[benchmarks(
	where
		T::Public: From<sp_core::sr25519::Public>,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_beacon_config() {
        let info: BeaconInfoResponse = serde_json::from_str(DRAND_INFO_RESPONSE).unwrap();
        let config = info.try_into_beacon_config().unwrap();

        let alice = sp_keyring::Sr25519Keyring::Alice.public();

        let config_payload = BeaconConfigurationPayload {
            block_number: 1u32.into(),
            config: config.clone(),
            public: alice.into(),
        };

        #[extrinsic_call]
        set_beacon_config(RawOrigin::None, config_payload.clone(), None);
        assert_eq!(BeaconConfig::<T>::get(), Some(config));
    }

    #[benchmark]
    fn write_pulse() {
        // TODO: bechmkark the longest `write_pulse` branch https://github.com/ideal-lab5/pallet-drand/issues/8

        let u_p: DrandResponseBody = serde_json::from_str(DRAND_PULSE).unwrap();
        let p: Pulse = u_p.try_into_pulse().unwrap();
        let block_number = 1u32.into();
        let alice = sp_keyring::Sr25519Keyring::Alice.public();
        let pulse_payload = PulsePayload {
            block_number,
            pulse: p.clone(),
            public: alice.into(),
        };

        #[extrinsic_call]
        write_pulse(RawOrigin::None, pulse_payload.clone(), None);
        assert_eq!(Pulses::<T>::get(block_number), None);
    }

    impl_benchmark_test_suite!(Drand, crate::mock::new_test_ext(), crate::mock::Test);
}
