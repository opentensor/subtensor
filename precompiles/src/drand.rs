use alloc::string::String;
use core::marker::PhantomData;

use pallet_drand::pallet::MigrationKeyMaxLen;
use pallet_evm::PrecompileHandle;
use precompile_utils::{EvmResult, prelude::UnboundedBytes, solidity::Codec};
use sp_core::H256;
use sp_runtime::{BoundedVec, traits::SaturatedConversion};
use sp_std::vec::Vec;

use fp_evm::PrecompileFailure;
use precompile_utils::prelude::PrecompileHandleExt;

use crate::{PrecompileExt, PrecompileHandleExtStorage};

/// Returned by `getPulse` — matches the `DrandPulse` Solidity struct.
#[derive(Codec)]
struct DrandPulse {
    randomness: H256,
    signature: UnboundedBytes,
}

/// Returned by `getBeaconConfig` — matches the `BeaconConfig` Solidity struct.
/// Maps the full on-chain `BeaconConfiguration` struct:
///   public_key (96 bytes)   → publicKey  (bytes)   — no bytes96 in Solidity
///   period                  → period     (uint32)
///   genesis_time            → genesisTime(uint32)
///   hash (exactly 32 bytes) → chainHash  (bytes32)  — fixed-size H256
///   group_hash (32 bytes)   → groupHash  (bytes32)  — fixed-size H256
///   scheme_id (≤32 bytes)   → schemeId   (bytes)    — UTF-8 string bytes
///   metadata.beacon_id      → beaconId   (bytes)    — UTF-8 string bytes
///   is_explicitly_configured is synthetic: true iff the storage key was explicitly written.
#[derive(Codec)]
struct BeaconConfig {
    genesis_time: u32,
    period: u32,
    public_key: UnboundedBytes,
    chain_hash: H256,           // BoundedHash = BoundedVec<u8, ConstU32<32>> → always 32 bytes
    group_hash: H256,           // same
    scheme_id: UnboundedBytes,  // UTF-8 string bytes, variable ≤32
    beacon_id: UnboundedBytes,  // UTF-8 string bytes, variable ≤32
    is_explicitly_configured: bool,
}

pub(crate) struct DrandPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for DrandPrecompile<R>
where
    R: frame_system::Config + pallet_drand::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
{
    const INDEX: u64 = 2065;
}

#[precompile_utils::precompile]
impl<R> DrandPrecompile<R>
where
    R: frame_system::Config + pallet_drand::Config + pallet_evm::Config,
{
    /// Returns the last stored drand round number.
    #[precompile::public("getLastStoredRound()")]
    #[precompile::view]
    fn get_last_stored_round(handle: &mut impl PrecompileHandle) -> EvmResult<u64> {
        let val = pallet_drand::LastStoredRound::<R>::get();
        handle.record_db_read_encoded::<R>(&val)?;
        Ok(val)
    }

    /// Returns the oldest stored drand round number.
    #[precompile::public("getOldestStoredRound()")]
    #[precompile::view]
    fn get_oldest_stored_round(handle: &mut impl PrecompileHandle) -> EvmResult<u64> {
        let val = pallet_drand::OldestStoredRound::<R>::get();
        handle.record_db_read_encoded::<R>(&val)?;
        Ok(val)
    }

    /// Returns the pulse (randomness, signature) for a specific round.
    /// Returns empty bytes for both fields if the round is not found.
    #[precompile::public("getPulse(uint64)")]
    #[precompile::view]
    fn get_pulse(
        handle: &mut impl PrecompileHandle,
        round: u64,
    ) -> EvmResult<DrandPulse> {
        let pulse_opt = pallet_drand::Pulses::<R>::get(round);
        handle.record_db_read_encoded::<R>(&pulse_opt)?;
        match pulse_opt {
            Some(pulse) => {
                let rand = pulse.randomness.into_inner();
                let bounded: [u8; 32] = rand.try_into().unwrap_or([0u8; 32]);
                Ok(DrandPulse {
                    randomness: H256::from(bounded),
                    signature: pulse.signature.into_inner().into(),
                })
            }
            None => Ok(DrandPulse {
                randomness: H256::zero(),
                signature: UnboundedBytes::from(&b""[..]),
            }),
        }
    }

    /// Returns the randomness from the latest stored round as bytes32.
    /// Returns zero bytes if no pulse is stored.
    #[precompile::public("getCurrentRandomness()")]
    #[precompile::view]
    fn get_current_randomness(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
        let last_round = pallet_drand::LastStoredRound::<R>::get();
        handle.record_db_read_encoded::<R>(&last_round)?;
        let pulse_opt = pallet_drand::Pulses::<R>::get(last_round);
        handle.record_db_read_encoded::<R>(&pulse_opt)?;
        match pulse_opt {
            Some(pulse) => {
                let rand = pulse.randomness.into_inner();
                let bounded: [u8; 32] = rand.try_into().unwrap_or([0u8; 32]);
                Ok(H256::from(bounded))
            }
            None => Ok(H256::zero()),
        }
    }

    /// Returns the drand beacon configuration.
    ///
    /// Exposes all fields of the on-chain `BeaconConfiguration` struct.
    /// `isExplicitlyConfigured` is `false` when the storage key has never been written —
    /// in that case the pallet operates with the hardcoded Quicknet default, not an
    /// unconfigured state. An off-chain caller seeing `false` should treat the returned
    /// values as the Quicknet defaults rather than "no config".
    #[precompile::public("getBeaconConfig()")]
    #[precompile::view]
    fn get_beacon_config(
        handle: &mut impl PrecompileHandle,
    ) -> EvmResult<BeaconConfig> {
        // try_get returns Err(()) when the key is absent; .get() would return the
        // DefaultBeaconConfig hook value, hiding whether it was explicitly set.
        let is_explicit = pallet_drand::BeaconConfig::<R>::try_get().is_ok();
        let config = pallet_drand::BeaconConfig::<R>::get();

        if is_explicit {
            // DB hit: charge for the actual bytes read.
            handle.record_db_read_encoded::<R>(&config)?;
        } else {
            // DB miss: charge the 1-byte minimum — nothing was read from storage.
            PrecompileHandleExt::record_db_read::<R>(handle, 1)
                .map_err(|e| PrecompileFailure::Error { exit_status: e })?;
        }

        Ok(BeaconConfig {
            genesis_time: config.genesis_time,
            period: config.period,
            public_key: config.public_key.into_inner().into(),
            chain_hash: {
                let v = config.hash.into_inner();
                let arr: [u8; 32] = v.try_into().unwrap_or([0u8; 32]);
                H256::from(arr)
            },
            group_hash: {
                let v = config.group_hash.into_inner();
                let arr: [u8; 32] = v.try_into().unwrap_or([0u8; 32]);
                H256::from(arr)
            },
            scheme_id: config.scheme_id.into_inner().into(),
            beacon_id: config.metadata.beacon_id.into_inner().into(),
            is_explicitly_configured: is_explicit,
        })
    }

    /// Returns whether a specific migration has run.
    /// migrationName must match the key stored on-chain exactly (e.g. "migrate_set_oldest_round").
    #[precompile::public("getHasMigrationRun(string)")]
    #[precompile::view]
    fn get_has_migration_run(
        handle: &mut impl PrecompileHandle,
        migration_name: UnboundedBytes,
    ) -> EvmResult<bool> {
        let raw: Vec<u8> = migration_name.into();
        // Use the pallet's own MigrationKeyMaxLen so the limit here is always
        // in sync with the storage definition — no magic constant duplication.
        let bounded_key: BoundedVec<u8, MigrationKeyMaxLen> = BoundedVec::try_from(raw)
            .map_err(|_| precompile_utils::prelude::revert("migration key too long"))?;
        let val = pallet_drand::HasMigrationRun::<R>::get(&bounded_key);
        handle.record_db_read_encoded::<R>(&val)?;
        Ok(val)
    }

    /// Returns the block number when the next unsigned transaction will be accepted.
    #[precompile::public("getNextUnsignedAt()")]
    #[precompile::view]
    fn get_next_unsigned_at(handle: &mut impl PrecompileHandle) -> EvmResult<u64> {
        let val = pallet_drand::NextUnsignedAt::<R>::get();
        handle.record_db_read_encoded::<R>(&val)?;
        Ok(val.saturated_into::<u64>())
    }

    /// Returns the current pallet version from storage.
    #[precompile::public("getPalletVersion()")]
    #[precompile::view]
    fn get_pallet_version(_handle: &mut impl PrecompileHandle) -> EvmResult<u16> {
        Ok(
            <pallet_drand::Pallet<R> as frame_support::traits::PalletInfoAccess>::crate_version()
                .major as u16,
        )
    }
}