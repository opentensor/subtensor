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

//! # Drand Bridge Pallet
//!
//! A pallet to bridge to [drand](drand.love)'s Quicknet, injecting publicly verifiable randomness
//! into the runtime
//!
//! ## Overview
//!
//! Quicknet chain runs in an 'unchained' mode, producing a fresh pulse of randomness every 3s
//! This pallet implements an offchain worker that consumes pulses from quicket and then sends a
//! signed transaction to encode them in the runtime. The runtime uses the optimized arkworks host
//! functions to efficiently verify the pulse.
//!
//! Run `cargo doc --package pallet-drand --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// Many errors are transformed throughout the pallet.
#![allow(clippy::manual_inspect)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

extern crate alloc;
use crate::alloc::string::ToString;

use alloc::{format, string::String, vec, vec::Vec};
use codec::Encode;
use frame_support::{pallet_prelude::*, traits::Randomness};
use frame_system::{
    offchain::{
        AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
        SigningTypes,
    },
    pallet_prelude::BlockNumberFor,
};
use scale_info::prelude::cmp;
use sha2::{Digest, Sha256};
use sp_runtime::{
    offchain::{http, Duration},
    traits::{Hash, One},
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    KeyTypeId, Saturating,
};

pub mod bls12_381;
pub mod types;
pub mod utils;
pub mod verifier;

use types::*;
use verifier::Verifier;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

/// the main drand api endpoint
pub const API_ENDPOINT: &str = "https://drand.cloudflare.com";
/// the drand quicknet chain hash
/// quicknet uses 'Tiny' BLS381, with small 48-byte sigs in G1 and 96-byte pubkeys in G2
pub const QUICKNET_CHAIN_HASH: &str =
    "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971";
/// the drand mainnet chain hash
/// mainnext uses 'Usual' BLS381, with 96-byte sigs in G2 and 48-byte pubkeys in G1
pub const MAINNET_CHAIN_HASH: &str =
    "8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce";

#[cfg(feature = "mainnet")]
const CHAIN_HASH: &str = MAINNET_CHAIN_HASH;

#[cfg(not(feature = "mainnet"))]
const CHAIN_HASH: &str = QUICKNET_CHAIN_HASH;

pub const MAX_PULSES_TO_FETCH: u64 = 20;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"drnd");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    // implemented for runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for TestAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

impl<T: SigningTypes> SignedPayload<T>
    for BeaconConfigurationPayload<T::Public, BlockNumberFor<T>>
{
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

impl<T: SigningTypes> SignedPayload<T> for PulsesPayload<T::Public, BlockNumberFor<T>> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
        /// The identifier type for an offchain worker.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
        /// something that knows how to verify beacon pulses
        type Verifier: Verifier;
        /// A configuration for base priority of unsigned transactions.
        ///
        /// This is exposed so that it can be tuned for particular runtime, when
        /// multiple pallets send unsigned transactions.
        #[pallet::constant]
        type UnsignedPriority: Get<TransactionPriority>;
        /// The maximum number of milliseconds we are willing to wait for the HTTP request to
        /// complete.
        #[pallet::constant]
        type HttpFetchTimeout: Get<u64>;
    }

    /// the drand beacon configuration
    #[pallet::storage]
    pub type BeaconConfig<T: Config> = StorageValue<_, BeaconConfiguration, OptionQuery>;

    /// map round number to pulse
    #[pallet::storage]
    pub type Pulses<T: Config> = StorageMap<_, Blake2_128Concat, RoundNumber, Pulse, OptionQuery>;

    #[pallet::storage]
    pub(super) type LastStoredRound<T: Config> = StorageValue<_, RoundNumber, ValueQuery>;

    /// Defines the block when next unsigned transaction will be accepted.
    ///
    /// To prevent spam of unsigned (and unpaid!) transactions on the network,
    /// we only allow one transaction per block.
    /// This storage entry defines when new transaction is going to be accepted.
    #[pallet::storage]
    pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BeaconConfigChanged,
        /// A user has successfully set a new value.
        NewPulse {
            /// The new value set.
            rounds: Vec<RoundNumber>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The value retrieved was `None` as no value was previously set.
        NoneValue,
        /// There was an attempt to increment the value in storage over `u32::MAX`.
        StorageOverflow,
        /// failed to connect to the
        DrandConnectionFailure,
        /// the pulse is invalid
        UnverifiedPulse,
        /// the round number did not increment
        InvalidRoundNumber,
        /// the pulse could not be verified
        PulseVerificationError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            // if the beacon config isn't available, get it now
            if BeaconConfig::<T>::get().is_none() {
                if let Err(e) = Self::fetch_drand_config_and_send(block_number) {
                    log::debug!(
						"Drand: Failed to fetch chain config from drand, are you sure the chain hash is valid? {:?}",
						e
					);
                }
            } else {
                // otherwise query drand
                if let Err(e) = Self::fetch_drand_pulse_and_send_unsigned(block_number) {
                    log::debug!(
						"Drand: Failed to fetch pulse from drand, are you sure the chain hash is valid? {:?}",
						e
					);
                }
            }
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        /// Validate unsigned call to this module.
        ///
        /// By default unsigned transactions are disallowed, but implementing the validator
        /// here we make sure that some particular calls (the ones produced by offchain worker)
        /// are being whitelisted and marked as valid.
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            match call {
                Call::set_beacon_config {
                    config_payload: ref payload,
                    ref signature,
                } => {
                    let signature = signature.as_ref().ok_or(InvalidTransaction::BadSigner)?;
                    // TODO validate it is a trusted source as any well-formatted config would pass
                    // https://github.com/ideal-lab5/pallet-drand/issues/3
                    Self::validate_signature_and_parameters(
                        payload,
                        signature,
                        &payload.block_number,
                    )
                }
                Call::write_pulse {
                    pulses_payload: ref payload,
                    ref signature,
                } => {
                    let signature = signature.as_ref().ok_or(InvalidTransaction::BadSigner)?;
                    Self::validate_signature_and_parameters(
                        payload,
                        signature,
                        &payload.block_number,
                    )
                }
                _ => InvalidTransaction::Call.into(),
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Verify and write a pulse from the beacon into the runtime
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::write_pulse(pulses_payload.pulses.len() as u32))]
        pub fn write_pulse(
            origin: OriginFor<T>,
            pulses_payload: PulsesPayload<T::Public, BlockNumberFor<T>>,
            _signature: Option<T::Signature>,
        ) -> DispatchResult {
            ensure_none(origin)?;
            let config = BeaconConfig::<T>::get().ok_or(Error::<T>::NoneValue)?;

            let mut last_stored_round = LastStoredRound::<T>::get();
            let mut new_rounds = Vec::new();

            for pulse in &pulses_payload.pulses {
                let is_verified = T::Verifier::verify(config.clone(), pulse.clone())
                    .map_err(|_| Error::<T>::PulseVerificationError)?;

                if is_verified {
                    ensure!(
                        pulse.round > last_stored_round,
                        Error::<T>::InvalidRoundNumber
                    );

                    // Store the pulse
                    Pulses::<T>::insert(pulse.round, pulse.clone());

                    // Update last stored round
                    last_stored_round = pulse.round;

                    // Collect the new round
                    new_rounds.push(pulse.round);
                }
            }

            // Update LastStoredRound storage
            LastStoredRound::<T>::put(last_stored_round);

            // Update the next unsigned block number
            let current_block = frame_system::Pallet::<T>::block_number();
            <NextUnsignedAt<T>>::put(current_block.saturating_add(One::one()));

            // Emit a single event with all new rounds
            if !new_rounds.is_empty() {
                Self::deposit_event(Event::NewPulse { rounds: new_rounds });
            }

            Ok(())
        }
        /// allows the root user to set the beacon configuration
        /// generally this would be called from an offchain worker context.
        /// there is no verification of configurations, so be careful with this.
        ///
        /// * `origin`: the root user
        /// * `config`: the beacon configuration
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::set_beacon_config())]
        pub fn set_beacon_config(
            origin: OriginFor<T>,
            config_payload: BeaconConfigurationPayload<T::Public, BlockNumberFor<T>>,
            _signature: Option<T::Signature>,
        ) -> DispatchResult {
            ensure_none(origin)?;
            BeaconConfig::<T>::put(config_payload.config);

            // now increment the block number at which we expect next unsigned transaction.
            let current_block = frame_system::Pallet::<T>::block_number();
            <NextUnsignedAt<T>>::put(current_block.saturating_add(One::one()));

            Self::deposit_event(Event::BeaconConfigChanged {});
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// query drand's /info endpoint for the quicknet chain
    /// then send a signed transaction to encode it on-chain
    fn fetch_drand_config_and_send(block_number: BlockNumberFor<T>) -> Result<(), &'static str> {
        // Make sure we don't fetch the config if the transaction is going to be rejected
        // anyway.
        let next_unsigned_at = NextUnsignedAt::<T>::get();
        if next_unsigned_at > block_number {
            return Err("Drand: Too early to send unsigned transaction");
        }

        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err(
                "Drand: No local accounts available. Consider adding one via `author_insertKey` RPC.",
            )?;
        }

        let body_str =
            Self::fetch_drand_chain_info().map_err(|_| "Failed to fetch drand chain info")?;
        let beacon_config: BeaconInfoResponse = serde_json::from_str(&body_str)
            .map_err(|_| "Drand: Failed to convert response body to beacon configuration")?;
        let config = beacon_config
            .try_into_beacon_config()
            .map_err(|_| "Drand: Failed to convert BeaconInfoResponse to BeaconConfiguration")?;

        let results = signer.send_unsigned_transaction(
            |account| BeaconConfigurationPayload {
                block_number,
                config: config.clone(),
                public: account.public.clone(),
            },
            |config_payload, signature| Call::set_beacon_config {
                config_payload,
                signature: Some(signature),
            },
        );

        if results.is_empty() {
            log::error!("Drand: Empty result from config: {:?}", config);
        }

        for (acc, res) in &results {
            match res {
                Ok(()) => log::info!("Drand: [{:?}] Submitted new config: {:?}", acc.id, config),
                Err(e) => log::error!(
                    "Drand: [{:?}] Failed to submit transaction: {:?}",
                    acc.id,
                    e
                ),
            }
        }

        Ok(())
    }

    /// fetch the latest public pulse from the configured drand beacon
    /// then send a signed transaction to include it on-chain
    fn fetch_drand_pulse_and_send_unsigned(
        block_number: BlockNumberFor<T>,
    ) -> Result<(), &'static str> {
        // Ensure we can send an unsigned transaction
        let next_unsigned_at = NextUnsignedAt::<T>::get();
        if next_unsigned_at > block_number {
            return Err("Drand: Too early to send unsigned transaction");
        }

        let mut last_stored_round = LastStoredRound::<T>::get();
        let latest_pulse_body = Self::fetch_drand_latest().map_err(|_| "Failed to query drand")?;
        let latest_unbounded_pulse: DrandResponseBody = serde_json::from_str(&latest_pulse_body)
            .map_err(|_| "Drand: Failed to serialize response body to pulse")?;
        let latest_pulse = latest_unbounded_pulse
            .try_into_pulse()
            .map_err(|_| "Drand: Received pulse contains invalid data")?;
        let current_round = latest_pulse.round;

        // If last_stored_round is zero, set it to current_round - 1
        if last_stored_round == 0 {
            last_stored_round = current_round.saturating_sub(1);
            LastStoredRound::<T>::put(last_stored_round);
        }

        if current_round > last_stored_round {
            let rounds_to_fetch = cmp::min(
                current_round.saturating_sub(last_stored_round),
                MAX_PULSES_TO_FETCH,
            );
            let mut pulses = Vec::new();

            for round in (last_stored_round.saturating_add(1))
                ..=(last_stored_round.saturating_add(rounds_to_fetch))
            {
                let pulse_body = Self::fetch_drand_by_round(round)
                    .map_err(|_| "Drand: Failed to query drand for round")?;
                let unbounded_pulse: DrandResponseBody = serde_json::from_str(&pulse_body)
                    .map_err(|_| "Drand: Failed to serialize response body to pulse")?;
                let pulse = unbounded_pulse
                    .try_into_pulse()
                    .map_err(|_| "Drand: Received pulse contains invalid data")?;
                pulses.push(pulse);
            }

            let signer = Signer::<T, T::AuthorityId>::all_accounts();

            let results = signer.send_unsigned_transaction(
                |account| PulsesPayload {
                    block_number,
                    pulses: pulses.clone(),
                    public: account.public.clone(),
                },
                |pulses_payload, signature| Call::write_pulse {
                    pulses_payload,
                    signature: Some(signature),
                },
            );

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::debug!(
                        "Drand: [{:?}] Submitted new pulses up to round: {:?}",
                        acc.id,
                        last_stored_round.saturating_add(rounds_to_fetch)
                    ),
                    Err(e) => log::error!(
                        "Drand: [{:?}] Failed to submit transaction: {:?}",
                        acc.id,
                        e
                    ),
                }
            }
        }

        Ok(())
    }

    /// Query the endpoint `{api}/{chainHash}/info` to receive information about the drand chain
    /// Valid response bodies are deserialized into `BeaconInfoResponse`
    fn fetch_drand_chain_info() -> Result<String, http::Error> {
        let uri: &str = &format!("{}/{}/info", API_ENDPOINT, CHAIN_HASH);
        Self::fetch(uri)
    }
    fn fetch_drand_by_round(round: RoundNumber) -> Result<String, http::Error> {
        let uri: &str = &format!("{}/{}/public/{}", API_ENDPOINT, CHAIN_HASH, round);
        Self::fetch(uri)
    }
    fn fetch_drand_latest() -> Result<String, http::Error> {
        let uri: &str = &format!("{}/{}/public/latest", API_ENDPOINT, CHAIN_HASH);
        Self::fetch(uri)
    }

    /// Fetch a remote URL and return the body of the response as a string.
    fn fetch(uri: &str) -> Result<String, http::Error> {
        let deadline =
            sp_io::offchain::timestamp().add(Duration::from_millis(T::HttpFetchTimeout::get()));
        let request = http::Request::get(uri);
        let pending = request.deadline(deadline).send().map_err(|_| {
            log::warn!("Drand: HTTP IO Error");
            http::Error::IoError
        })?;
        let response = pending.try_wait(deadline).map_err(|_| {
            log::warn!("Drand: HTTP Deadline Reached");
            http::Error::DeadlineReached
        })??;

        if response.code != 200 {
            log::warn!("Drand: Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }
        let body = response.body().collect::<Vec<u8>>();
        let body_str = alloc::str::from_utf8(&body).map_err(|_| {
            log::warn!("Drand: No UTF8 body");
            http::Error::Unknown
        })?;

        Ok(body_str.to_string())
    }

    /// get the randomness at a specific block height
    /// returns [0u8;32] if it does not exist
    pub fn random_at(round: RoundNumber) -> [u8; 32] {
        let pulse = Pulses::<T>::get(round).unwrap_or_default();
        let rand = pulse.randomness.clone();
        let bounded_rand: [u8; 32] = rand.into_inner().try_into().unwrap_or([0u8; 32]);

        bounded_rand
    }

    fn validate_signature_and_parameters(
        payload: &impl SignedPayload<T>,
        signature: &T::Signature,
        block_number: &BlockNumberFor<T>,
    ) -> TransactionValidity {
        let signature_valid =
            SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
        if !signature_valid {
            return InvalidTransaction::BadProof.into();
        }
        Self::validate_transaction_parameters(block_number)
    }

    fn validate_transaction_parameters(block_number: &BlockNumberFor<T>) -> TransactionValidity {
        // Now let's check if the transaction has any chance to succeed.
        let next_unsigned_at = NextUnsignedAt::<T>::get();
        if &next_unsigned_at > block_number {
            return InvalidTransaction::Stale.into();
        }
        // Let's make sure to reject transactions from the future.
        let current_block = frame_system::Pallet::<T>::block_number();
        if &current_block < block_number {
            return InvalidTransaction::Future.into();
        }

        ValidTransaction::with_tag_prefix("DrandOffchainWorker")
            // We set the priority to the value stored at `UnsignedPriority`.
            .priority(T::UnsignedPriority::get())
            // This transaction does not require anything else to go before into the pool.
            // In theory we could require `previous_unsigned_at` transaction to go first,
            // but it's not necessary in our case.
            // We set the `provides` tag to be the same as `next_unsigned_at`. This makes
            // sure only one transaction produced after `next_unsigned_at` will ever
            // get to the transaction pool and will end up in the block.
            // We can still have multiple transactions compete for the same "spot",
            // and the one with higher priority will replace other one in the pool.
            .and_provides(next_unsigned_at)
            // The transaction is only valid for next block. After that it's
            // going to be revalidated by the pool.
            .longevity(1)
            // It's fine to propagate that transaction to other peers, which means it can be
            // created even by nodes that don't produce blocks.
            // Note that sometimes it's better to keep it for yourself (if you are the block
            // producer), since for instance in some schemes others may copy your solution and
            // claim a reward.
            .propagate(true)
            .build()
    }
}

/// construct a message (e.g. signed by drand)
pub fn message(current_round: RoundNumber, prev_sig: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::default();
    hasher.update(prev_sig);
    hasher.update(current_round.to_be_bytes());
    hasher.finalize().to_vec()
}

impl<T: Config> Randomness<T::Hash, BlockNumberFor<T>> for Pallet<T> {
    // this function hashes together the subject with the latest known randomness from quicknet
    fn random(subject: &[u8]) -> (T::Hash, BlockNumberFor<T>) {
        let block_number_minus_one =
            <frame_system::Pallet<T>>::block_number().saturating_sub(One::one());

        let last_stored_round = LastStoredRound::<T>::get();

        let mut entropy = T::Hash::default();
        if let Some(pulse) = Pulses::<T>::get(last_stored_round) {
            entropy = (subject, block_number_minus_one, pulse.randomness.clone())
                .using_encoded(T::Hashing::hash);
        }

        (entropy, block_number_minus_one)
    }
}
