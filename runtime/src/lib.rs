#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::Encode;

use pallet_commitments::CanCommit;
use pallet_grandpa::{
    AuthorityId as GrandpaId,
};

use frame_support::pallet_prelude::{DispatchError};
use frame_system::{EnsureNever};

//use pallet_registry::CanRegisterIdentity;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify, AccountIdLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;


use frame_support::genesis_builder_helper::{build_config, create_default_config};
// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, Randomness,
		StorageInfo,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		IdentityFee, Weight,
	},
	StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{CurrencyAdapter, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};


/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("node-subtensor"),
    impl_name: create_runtime_str!("node-subtensor"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 141,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12_000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::with_sensible_defaults(
			Weight::from_parts(4u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
			NORMAL_DISPATCH_RATIO,
		);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(10 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    // The basic call filter to use in dispatchable.
    type BaseCallFilter = frame_support::traits::Everything;
    // Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    // The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    // The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    // The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    // The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    // The type for hashing blocks and tries.
    type Hash = Hash;
    // The hashing algorithm used.
    type Hashing = BlakeTwo256;
    // The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    // The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
    // Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    // The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    // Version of the runtime.
    type Version = Version;
    // This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    // What to do if a new account is created.
    type OnNewAccount = ();
    // What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    // The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    // Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    // This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    // The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;

	type Nonce = u32;
	type Block = Block;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;

	#[cfg(feature = "experimental")]
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type MaxHolds = ();
}

pub struct LinearWeightToFee<C>(sp_std::marker::PhantomData<C>);

use frame_support::weights::{WeightToFeePolynomial, WeightToFeeCoefficient, WeightToFeeCoefficients};
use sp_runtime::traits::Get;
use smallvec::smallvec;

impl<C> WeightToFeePolynomial for LinearWeightToFee<C>
where
    C: Get<Balance>,
{
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let coefficient = WeightToFeeCoefficient {
            coeff_integer: 0,
            coeff_frac: Perbill::from_parts(1),
            negative: false,
            degree: 1,
        };

        smallvec!(coefficient)
    }
}

parameter_types! {
    // Used with LinearWeightToFee conversion.
    pub const FeeWeightRatio: u64 = 1;
    pub const TransactionByteFee: u128 = 1;
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
    type WeightToFee = LinearWeightToFee<FeeWeightRatio>;
	type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
    type OperationalFeeMultiplier = ConstU8<1>;
}

impl pallet_sudo::Config for Runtime 
{
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

pub struct AllowCommitments;
impl CanCommit<AccountId> for AllowCommitments {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn can_commit(netuid: u16, address: &AccountId) -> bool {
        Subtensor::is_hotkey_registered_on_network(netuid, address)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn can_commit(_: u16, _: &AccountId) -> bool {
        true
    }
}

parameter_types! {
    pub const MaxCommitFields: u32 = 1;
    pub const CommitmentInitialDeposit: Balance = 0; // Free
    pub const CommitmentFieldDeposit: Balance = 0; // Free
    pub const CommitmentRateLimit: BlockNumber = 100; // Allow commitment every 100 blocks
}

impl pallet_commitments::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = pallet_commitments::weights::SubstrateWeight<Runtime>;

    type CanCommit = AllowCommitments;

    type MaxFields = MaxCommitFields;
    type InitialDeposit = CommitmentInitialDeposit;
    type FieldDeposit = CommitmentFieldDeposit;
    type RateLimit = CommitmentRateLimit;
}

// Configure the pallet subtensor.
parameter_types! {
    pub const SubtensorInitialRho: u16 = 10;
    pub const SubtensorInitialKappa: u16 = 32_767; // 0.5 = 65535/2
    pub const SubtensorInitialMaxAllowedUids: u16 = 256;
    pub const SubtensorInitialIssuance: u64 = 0;
    pub const SubtensorInitialMinAllowedWeights: u16 = 1;
    pub const SubtensorInitialMaxWeightLimit: u16 = u16::MAX; 
    pub const SubtensorInitialEmissionValue: u16 = 0;
    pub const SubtensorInitialMaxWeightsLimit: u16 = 1000; // 1000/2^16 = 0.015
    pub const SubtensorInitialValidatorPruneLen: u64 = 1;
    pub const SubtensorInitialScalingLawPower: u16 = 50; // 0.5
    pub const SubtensorInitialMaxAllowedValidators: u16 = 64;
    pub const SubtensorInitialTempo: u16 = 99;
    pub const SubtensorInitialDifficulty: u64 = 10_000_000;
    pub const SubtensorInitialAdjustmentInterval: u16 = 360;
    pub const SubtensorInitialAdjustmentAlpha: u64 = 58000; // no weight to previous value.
    pub const SubtensorInitialTargetRegistrationsPerInterval: u16 = 1;
    pub const SubtensorInitialImmunityPeriod: u16 = 5000;
    pub const SubtensorInitialActivityCutoff: u16 = 5000;
    pub const SubtensorInitialMaxRegistrationsPerBlock: u16 = 1;
    pub const SubtensorInitialPruningScore : u16 = u16::MAX;
    pub const SubtensorInitialBondsMovingAverage: u64 = 900_000;
    pub const SubtensorInitialDefaultTake: u16 = 11_796; // 18% honest number.
    pub const SubtensorInitialWeightsVersionKey: u64 = 0;
    pub const SubtensorInitialMinDifficulty: u64 = u64::MAX;
    pub const SubtensorInitialMaxDifficulty: u64 = u64::MAX;
    pub const SubtensorInitialServingRateLimit: u64 = 50;
    pub const SubtensorInitialBurn: u64 = 1_000_000_000; // 1 tao
    pub const SubtensorInitialMinBurn: u64 = 1; // 1 tao
    pub const SubtensorInitialMaxBurn: u64 = 100_000_000_000; // 100 tao
    pub const SubtensorInitialTxRateLimit: u64 = 1000;
    pub const SubtensorInitialRAORecycledForRegistration: u64 = 0; // 0 rao
    pub const SubtensorInitialSenateRequiredStakePercentage: u64 = 0; // 1 percent of total stake
    pub const SubtensorInitialNetworkImmunity: u64 = 7 * 7200;
    pub const SubtensorInitialNetworkRegistrationAllowed: bool = true;
    pub const SubtensorInitialRegistrationAllowed: bool = false;
    pub const SubtensorInitialMinAllowedUids: u16 = 128;
    pub const SubtensorInitialMinLockCost: u64 = 1_000_000_000_000; // 1000 TAO
    pub const SubtensorInitialSubnetOwnerCut: u16 = 11_796; // 18 percent
    pub const SubtensorInitialSubnetLimit: u16 = 12;
    pub const SubtensorInitialNetworkLockReductionInterval: u64 = 14 * 7200;
    pub const SubtensorInitialNetworkRateLimit: u64 = 1 * 7200;
}

impl pallet_subtensor::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SudoRuntimeCall = RuntimeCall;
    type Currency = Balances;
    type CouncilOrigin = EnsureNever<AccountId>;

    type InitialRho = SubtensorInitialRho;
    type InitialKappa = SubtensorInitialKappa;
    type InitialMaxAllowedUids = SubtensorInitialMaxAllowedUids;
    type InitialBondsMovingAverage = SubtensorInitialBondsMovingAverage;
    type InitialIssuance = SubtensorInitialIssuance;
    type InitialMinAllowedWeights = SubtensorInitialMinAllowedWeights;
    type InitialMaxWeightLimit = SubtensorInitialMaxWeightLimit;
    type InitialEmissionValue = SubtensorInitialEmissionValue;
    type InitialMaxWeightsLimit = SubtensorInitialMaxWeightsLimit;
    type InitialValidatorPruneLen = SubtensorInitialValidatorPruneLen;
    type InitialScalingLawPower = SubtensorInitialScalingLawPower;
    type InitialTempo = SubtensorInitialTempo;
    type InitialDifficulty = SubtensorInitialDifficulty;
    type InitialAdjustmentInterval = SubtensorInitialAdjustmentInterval;
    type InitialAdjustmentAlpha = SubtensorInitialAdjustmentAlpha;
    type InitialTargetRegistrationsPerInterval = SubtensorInitialTargetRegistrationsPerInterval;
    type InitialImmunityPeriod = SubtensorInitialImmunityPeriod;
    type InitialActivityCutoff = SubtensorInitialActivityCutoff;
    type InitialMaxRegistrationsPerBlock = SubtensorInitialMaxRegistrationsPerBlock;
    type InitialPruningScore = SubtensorInitialPruningScore;
    type InitialMaxAllowedValidators = SubtensorInitialMaxAllowedValidators;
    type InitialDefaultTake = SubtensorInitialDefaultTake;
    type InitialWeightsVersionKey = SubtensorInitialWeightsVersionKey;
    type InitialMaxDifficulty = SubtensorInitialMaxDifficulty;
    type InitialMinDifficulty = SubtensorInitialMinDifficulty;
    type InitialServingRateLimit = SubtensorInitialServingRateLimit;
    type InitialBurn = SubtensorInitialBurn;
    type InitialMaxBurn = SubtensorInitialMaxBurn;
    type InitialMinBurn = SubtensorInitialMinBurn;
    type InitialTxRateLimit = SubtensorInitialTxRateLimit;
    type InitialRAORecycledForRegistration = SubtensorInitialRAORecycledForRegistration;
    type InitialSenateRequiredStakePercentage = SubtensorInitialSenateRequiredStakePercentage;
    type InitialNetworkImmunityPeriod = SubtensorInitialNetworkImmunity;
    type InitialNetworkRegistrationAllowed = SubtensorInitialNetworkRegistrationAllowed;
    type InitialRegistrationAllowed = SubtensorInitialRegistrationAllowed;
    type InitialNetworkMinAllowedUids = SubtensorInitialMinAllowedUids;
    type InitialNetworkMinLockCost = SubtensorInitialMinLockCost;
    type InitialNetworkLockReductionInterval = SubtensorInitialNetworkLockReductionInterval;
    type InitialSubnetOwnerCut = SubtensorInitialSubnetOwnerCut;
    type InitialSubnetLimit = SubtensorInitialSubnetLimit;
    type InitialNetworkRateLimit = SubtensorInitialNetworkRateLimit;
}

use sp_runtime::BoundedVec;

pub struct AuraPalletIntrf;
impl pallet_admin_utils::AuraInterface<AuraId, ConstU32<32>> for AuraPalletIntrf 
{
    fn change_authorities(new: BoundedVec<AuraId, ConstU32<32>>)
    {
        Aura::change_authorities(new);
    }
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub struct Runtime
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Subtensor: pallet_subtensor,
        Sudo: pallet_sudo,
        Commitments: pallet_commitments,
        AdminUtils: pallet_admin_utils
    }
);

pub struct SubtensorInterface;

impl pallet_admin_utils::SubtensorInterface<AccountId, <pallet_balances::Pallet<Runtime> as frame_support::traits::Currency<AccountId>>::Balance, RuntimeOrigin> for SubtensorInterface
{
    fn set_default_take(default_take: u16)
    {
        Subtensor::set_default_take(default_take);
    }

	fn set_tx_rate_limit(rate_limit: u64)
    {
        Subtensor::set_tx_rate_limit(rate_limit);
    }

	fn set_serving_rate_limit(netuid: u16, rate_limit: u64)
    {
        Subtensor::set_serving_rate_limit(netuid, rate_limit);
    }

	fn set_max_burn(netuid: u16, max_burn: u64)
    {
        Subtensor::set_max_burn(netuid, max_burn);
    }

	fn set_min_burn(netuid: u16, min_burn: u64)
    {
        Subtensor::set_min_burn(netuid, min_burn);
    }

	fn set_burn(netuid: u16, burn: u64)
    {
        Subtensor::set_burn(netuid, burn);
    }

	fn set_max_difficulty(netuid: u16, max_diff: u64)
    {
        Subtensor::set_max_difficulty(netuid, max_diff);
    }

	fn set_min_difficulty(netuid: u16, min_diff: u64)
    {
        Subtensor::set_min_difficulty(netuid, min_diff);
    }

	fn set_difficulty(netuid: u16, diff: u64)
    {
        Subtensor::set_difficulty(netuid, diff);
    }

	fn set_weights_rate_limit(netuid: u16, rate_limit: u64)
    {
        Subtensor::set_weights_set_rate_limit(netuid, rate_limit);
    }

	fn set_weights_version_key(netuid: u16, version: u64)
    {
        Subtensor::set_weights_version_key(netuid, version);
    }

	fn set_bonds_moving_average(netuid: u16, moving_average: u64)
    {
        Subtensor::set_bonds_moving_average(netuid, moving_average);
    }

	fn set_max_allowed_validators(netuid: u16, max_validators: u16)
    {
        Subtensor::set_max_allowed_validators(netuid, max_validators);
    }

	fn get_root_netuid() -> u16
    {
        return Subtensor::get_root_netuid();
    }

	fn if_subnet_exist(netuid: u16) -> bool
    {
        return Subtensor::if_subnet_exist(netuid);
    }

	fn create_account_if_non_existent(coldkey: &AccountId, hotkey: &AccountId)
    {
        return Subtensor::create_account_if_non_existent(coldkey, hotkey);
    }

	fn coldkey_owns_hotkey(coldkey: &AccountId, hotkey: &AccountId) -> bool
    {
        return Subtensor::coldkey_owns_hotkey(coldkey, hotkey);
    }

	fn increase_stake_on_coldkey_hotkey_account(coldkey: &AccountId, hotkey: &AccountId, increment: u64)
    {
        Subtensor::increase_stake_on_coldkey_hotkey_account(coldkey, hotkey, increment);
    }

	fn u64_to_balance(input: u64) -> Option<Balance>
    {
        return Subtensor::u64_to_balance(input);
    }

	fn add_balance_to_coldkey_account(coldkey: &AccountId, amount: Balance)
    {
        Subtensor::add_balance_to_coldkey_account(coldkey, amount);
    }

	fn get_current_block_as_u64() -> u64
    {
        return Subtensor::get_current_block_as_u64();
    }

	fn get_subnetwork_n(netuid: u16) -> u16
    {
        return Subtensor::get_subnetwork_n(netuid);
    }

	fn get_max_allowed_uids(netuid: u16) -> u16
    {
        return Subtensor::get_max_allowed_uids(netuid);
    }

	fn append_neuron(netuid: u16, new_hotkey: &AccountId, block_number: u64)
    {
        return Subtensor::append_neuron(netuid, new_hotkey, block_number);
    }

	fn get_neuron_to_prune(netuid: u16) -> u16
    {
        return Subtensor::get_neuron_to_prune(netuid);
    }

	fn replace_neuron(netuid: u16, uid_to_replace: u16, new_hotkey: &AccountId, block_number: u64)
    {
        Subtensor::replace_neuron(netuid, uid_to_replace, new_hotkey, block_number);
    }

	fn set_total_issuance(total_issuance: u64)
    {
        Subtensor::set_total_issuance(total_issuance);
    }

	fn set_network_immunity_period(net_immunity_period: u64)
    {
        Subtensor::set_network_immunity_period(net_immunity_period);
    }

	fn set_network_min_lock(net_min_lock: u64)
    {
        Subtensor::set_network_min_lock(net_min_lock);
    }

    fn set_subnet_limit(limit: u16)
    {
        Subtensor::set_max_subnets(limit);
    }

    fn set_lock_reduction_interval(interval: u64)
    {
        Subtensor::set_lock_reduction_interval(interval);
    }

    fn set_tempo(netuid: u16, tempo: u16)
    {
        Subtensor::set_tempo(netuid, tempo);
    }

    fn set_subnet_owner_cut(subnet_owner_cut: u16)
    {
        Subtensor::set_subnet_owner_cut(subnet_owner_cut);
    }

    fn set_network_rate_limit(limit: u64)
    {
        Subtensor::set_network_rate_limit(limit);
    }

    fn set_max_registrations_per_block(netuid: u16, max_registrations_per_block: u16)
    {
        Subtensor::set_max_registrations_per_block(netuid, max_registrations_per_block);
    }

    fn set_adjustment_alpha(netuid: u16, adjustment_alpha: u64)
    {
        Subtensor::set_adjustment_alpha(netuid, adjustment_alpha);
    }

    fn set_target_registrations_per_interval(netuid: u16, target_registrations_per_interval: u16)
    {
        Subtensor::set_target_registrations_per_interval(netuid, target_registrations_per_interval);
    }

    fn set_network_pow_registration_allowed(netuid: u16, registration_allowed: bool)
    {
        Subtensor::set_network_pow_registration_allowed(netuid, registration_allowed);
    }

    fn set_network_registration_allowed(netuid: u16, registration_allowed: bool)
    {
        Subtensor::set_network_registration_allowed(netuid, registration_allowed);
    }

    fn set_activity_cutoff(netuid: u16, activity_cutoff: u16)
    {
        Subtensor::set_activity_cutoff(netuid, activity_cutoff);
    }

    fn ensure_subnet_owner_or_root(o: RuntimeOrigin, netuid: u16) -> Result<(), DispatchError>
    {
        return Subtensor::ensure_subnet_owner_or_root(o, netuid);
    }

    fn set_rho(netuid: u16, rho: u16)
    {
        Subtensor::set_rho(netuid, rho);
    }

    fn set_kappa(netuid: u16, kappa: u16)
    {
        Subtensor::set_kappa(netuid, kappa);
    }

    fn set_max_allowed_uids(netuid: u16, max_allowed: u16)
    {
        Subtensor::set_max_allowed_uids(netuid, max_allowed);
    }

    fn set_min_allowed_weights(netuid: u16, min_allowed_weights: u16)
    {
        Subtensor::set_min_allowed_weights(netuid, min_allowed_weights);
    }

    fn set_immunity_period(netuid: u16, immunity_period: u16)
    {
        Subtensor::set_immunity_period(netuid, immunity_period);
    }

    fn set_max_weight_limit(netuid: u16, max_weight_limit: u16)
    {
        Subtensor::set_max_weight_limit(netuid, max_weight_limit);
    }

    fn set_scaling_law_power(netuid: u16, scaling_law_power: u16)
    {
        Subtensor::set_scaling_law_power(netuid, scaling_law_power);
    }

    fn set_validator_prune_len(netuid: u16, validator_prune_len: u64)
    {
        Subtensor::set_validator_prune_len(netuid, validator_prune_len);
    }

    fn set_adjustment_interval(netuid: u16, adjustment_interval: u16)
    {
        Subtensor::set_adjustment_interval(netuid, adjustment_interval);
    }

    fn set_weights_set_rate_limit(netuid: u16, weights_set_rate_limit: u64)
    {
        Subtensor::set_weights_set_rate_limit(netuid, weights_set_rate_limit);
    }

    fn set_rao_recycled(netuid: u16, rao_recycled: u64)
    {
        Subtensor::set_rao_recycled(netuid, rao_recycled);
    }

    fn is_hotkey_registered_on_network(netuid: u16, hotkey: &AccountId) -> bool
    {
        return Subtensor::is_hotkey_registered_on_network(netuid, hotkey);
    }

    fn init_new_network(netuid: u16, tempo: u16)
    {
        Subtensor::init_new_network(netuid, tempo);
    }
}

impl pallet_admin_utils::Config for Runtime 
{
    type RuntimeEvent = RuntimeEvent;
    type AuthorityId = AuraId;
    type MaxAuthorities = ConstU32<32>;
    type Aura = AuraPalletIntrf;
    type Balance = Balance;
    type Subtensor = SubtensorInterface;
    type WeightInfo = pallet_admin_utils::weights::SubstrateWeight<Runtime>;
}

// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    pallet_subtensor::SubtensorSignedExtension<Runtime>,
    pallet_commitments::CommitmentsSignedExtension<Runtime>
);

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = ();

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_subtensor, Subtensor]
        [pallet_timestamp, Timestamp]
        [pallet_registry, Registry]
        [pallet_commitments, Commitments]
        [pallet_admin_utils, AdminUtils]
    );
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}
	}

	impl subtensor_custom_rpc_runtime_api::DelegateInfoRuntimeApi<Block> for Runtime {
        fn get_delegates() -> Vec<u8> {
			
            let result = Subtensor::get_delegates();
            result.encode()
        }

        fn get_delegate(delegate_account_vec: Vec<u8>) -> Vec<u8> {
			
            let _result = Subtensor::get_delegate(delegate_account_vec);
            if _result.is_some() {
                let result = _result.expect("Could not get DelegateInfo");
                result.encode()
            } else {
                vec![]
            }
        }

        fn get_delegated(delegatee_account_vec: Vec<u8>) -> Vec<u8> {
			
            let result = Subtensor::get_delegated(delegatee_account_vec);
            result.encode()
        }
    }

    impl subtensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi<Block> for Runtime {
        fn get_neurons_lite(netuid: u16) -> Vec<u8> {
			
            let result = Subtensor::get_neurons_lite(netuid);
            result.encode()
        }

        fn get_neuron_lite(netuid: u16, uid: u16) -> Vec<u8> {
						
            let _result = Subtensor::get_neuron_lite(netuid, uid);
            if _result.is_some() {
                let result = _result.expect("Could not get NeuronInfoLite");
                result.encode()
            } else {
                vec![]
            }
        }

        fn get_neurons(netuid: u16) -> Vec<u8> {
			
            let result = Subtensor::get_neurons(netuid);
            result.encode()
        }

        fn get_neuron(netuid: u16, uid: u16) -> Vec<u8> {
			
            let _result = Subtensor::get_neuron(netuid, uid);
            if _result.is_some() {
                let result = _result.expect("Could not get NeuronInfo");
                result.encode()
            } else {
                vec![]
            }
        }
    }

	impl subtensor_custom_rpc_runtime_api::SubnetInfoRuntimeApi<Block> for Runtime {
        fn get_subnet_info(netuid: u16) -> Vec<u8> {
						
            let _result = Subtensor::get_subnet_info(netuid);
            if _result.is_some() {
                let result = _result.expect("Could not get SubnetInfo");
                result.encode()
            } else {
                vec![]
            }
        }

        fn get_subnets_info() -> Vec<u8> {
						
            let result = Subtensor::get_subnets_info();
            result.encode()
        }

        fn get_subnet_hyperparams(netuid: u16) -> Vec<u8> 
        {
            let _result = Subtensor::get_subnet_hyperparams(netuid);
            if _result.is_some() 
            {
                let result = _result.expect("Could not get SubnetHyperparams");
                result.encode()
            } 
            else 
            {
                vec![]
            }
        }
    }

	impl subtensor_custom_rpc_runtime_api::StakeInfoRuntimeApi<Block> for Runtime 
    {
        fn get_stake_info_for_coldkey( coldkey_account_vec: Vec<u8> ) -> Vec<u8> 
        {	
            let result = Subtensor::get_stake_info_for_coldkey( coldkey_account_vec );
            result.encode()
        }

        fn get_stake_info_for_coldkeys( coldkey_account_vecs: Vec<Vec<u8>> ) -> Vec<u8> 
        {	
            let result = Subtensor::get_stake_info_for_coldkeys( coldkey_account_vecs );
            result.encode()
        }
    }

    impl subtensor_custom_rpc_runtime_api::SubnetRegistrationRuntimeApi<Block> for Runtime 
    {
        fn get_network_registration_cost() -> u64 
        {
            Subtensor::get_network_lock_cost()
        }
    }
}