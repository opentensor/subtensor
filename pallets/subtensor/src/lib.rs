#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]
// Edit this file to define custom logic or remove it if it is not needed.
// Learn more about FRAME and the core library of Substrate FRAME pallets:
// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_system::{self as system, ensure_signed};

use frame_support::{
    dispatch,
    dispatch::{DispatchError, DispatchInfo, DispatchResult, PostDispatchInfo},
    ensure,
    traits::{tokens::fungible, IsSubType},
};

use codec::{Decode, Encode};
use frame_support::sp_runtime::transaction_validity::InvalidTransaction;
use frame_support::sp_runtime::transaction_validity::ValidTransaction;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
};
use sp_std::marker::PhantomData;

// ============================
//	==== Benchmark Imports =====
// ============================
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

// =========================
//	==== Pallet Imports =====
// =========================
mod block_step;

mod epoch;
mod math;
mod registration;
mod root;
mod serving;
mod staking;
mod uids;
mod utils;
mod weights;

pub mod delegate_info;
pub mod neuron_info;
pub mod stake_info;
pub mod subnet_info;

// apparently this is stabilized since rust 1.36
extern crate alloc;
pub mod migration;

#[frame_support::pallet]
pub mod pallet {

    use frame_support::{
        dispatch::GetDispatchInfo,
        pallet_prelude::{DispatchResult, StorageMap, ValueQuery, *},
        sp_std::vec,
        sp_std::vec::Vec,
        traits::{tokens::fungible, UnfilteredDispatchable},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::TrailingZeroInput;

    #[cfg(not(feature = "std"))]
    use alloc::boxed::Box;
    #[cfg(feature = "std")]
    use sp_std::prelude::Box;

    // Tracks version for migrations. Should be monotonic with respect to the
    // order of migrations. (i.e. always increasing)
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A sudo-able call.
        type SudoRuntimeCall: Parameter
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo;

        /// Origin checking for council majority
        type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // --- Currency type that will be used to place deposits on neurons
        type Currency: fungible::Balanced<Self::AccountId, Balance = u64>
            + fungible::Mutate<Self::AccountId>;

        type SenateMembers: crate::MemberManagement<Self::AccountId>;

        type TriumvirateInterface: crate::CollectiveInterface<Self::AccountId, Self::Hash, u32>;

        // =================================
        // ==== Initial Value Constants ====
        // =================================
        #[pallet::constant] // Initial currency issuance.
        type InitialIssuance: Get<u64>;
        #[pallet::constant] // Initial min allowed weights setting.
        type InitialMinAllowedWeights: Get<u16>;
        #[pallet::constant] // Initial Emission Ratio.
        type InitialEmissionValue: Get<u16>;
        #[pallet::constant] // Initial max weight limit.
        type InitialMaxWeightsLimit: Get<u16>;
        #[pallet::constant] // Tempo for each network.
        type InitialTempo: Get<u16>;
        #[pallet::constant] // Initial Difficulty.
        type InitialDifficulty: Get<u64>;
        #[pallet::constant] // Initial Max Difficulty.
        type InitialMaxDifficulty: Get<u64>;
        #[pallet::constant] // Initial Min Difficulty.
        type InitialMinDifficulty: Get<u64>;
        #[pallet::constant] // Initial RAO Recycled.
        type InitialRAORecycledForRegistration: Get<u64>;
        #[pallet::constant] // Initial Burn.
        type InitialBurn: Get<u64>;
        #[pallet::constant] // Initial Max Burn.
        type InitialMaxBurn: Get<u64>;
        #[pallet::constant] // Initial Min Burn.
        type InitialMinBurn: Get<u64>;
        #[pallet::constant] // Initial adjustment interval.
        type InitialAdjustmentInterval: Get<u16>;
        #[pallet::constant] // Initial bonds moving average.
        type InitialBondsMovingAverage: Get<u64>;
        #[pallet::constant] // Initial target registrations per interval.
        type InitialTargetRegistrationsPerInterval: Get<u16>;
        #[pallet::constant] // Rho constant.
        type InitialRho: Get<u16>;
        #[pallet::constant] // Kappa constant.
        type InitialKappa: Get<u16>;
        #[pallet::constant] // Max UID constant.
        type InitialMaxAllowedUids: Get<u16>;
        #[pallet::constant] // Initial validator context pruning length.
        type InitialValidatorPruneLen: Get<u64>;
        #[pallet::constant] // Initial scaling law power.
        type InitialScalingLawPower: Get<u16>;
        #[pallet::constant] // Immunity Period Constant.
        type InitialImmunityPeriod: Get<u16>;
        #[pallet::constant] // Activity constant.
        type InitialActivityCutoff: Get<u16>;
        #[pallet::constant] // Initial max registrations per block.
        type InitialMaxRegistrationsPerBlock: Get<u16>;
        #[pallet::constant] // Initial pruning score for each neuron.
        type InitialPruningScore: Get<u16>;
        #[pallet::constant] // Initial maximum allowed validators per network.
        type InitialMaxAllowedValidators: Get<u16>;
        #[pallet::constant] // Initial default delegation take.
        type InitialDefaultTake: Get<u16>;
        #[pallet::constant] // Initial weights version key.
        type InitialWeightsVersionKey: Get<u64>;
        #[pallet::constant] // Initial serving rate limit.
        type InitialServingRateLimit: Get<u64>;
        #[pallet::constant] // Initial transaction rate limit.
        type InitialTxRateLimit: Get<u64>;
        #[pallet::constant] // Initial percentage of total stake required to join senate.
        type InitialSenateRequiredStakePercentage: Get<u64>;
        #[pallet::constant] // Initial adjustment alpha on burn and pow.
        type InitialAdjustmentAlpha: Get<u64>;
        #[pallet::constant] // Initial network immunity period
        type InitialNetworkImmunityPeriod: Get<u64>;
        #[pallet::constant] // Initial minimum allowed network UIDs
        type InitialNetworkMinAllowedUids: Get<u16>;
        #[pallet::constant] // Initial network minimum burn cost
        type InitialNetworkMinLockCost: Get<u64>;
        #[pallet::constant] // Initial network subnet cut.
        type InitialSubnetOwnerCut: Get<u16>;
        #[pallet::constant] // Initial lock reduction interval.
        type InitialNetworkLockReductionInterval: Get<u64>;
        #[pallet::constant] // Initial max allowed subnets
        type InitialSubnetLimit: Get<u16>;
        #[pallet::constant] // Initial network creation rate limit
        type InitialNetworkRateLimit: Get<u64>;
        #[pallet::constant] // Initial target stakes per interval issuance.
        type InitialTargetStakesPerInterval: Get<u64>;
    }

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    // Senate requirements
    #[pallet::type_value]
    pub fn DefaultSenateRequiredStakePercentage<T: Config>() -> u64 {
        T::InitialSenateRequiredStakePercentage::get()
    }

    #[pallet::storage]
    pub(super) type SenateRequiredStakePercentage<T> =
        StorageValue<_, u64, ValueQuery, DefaultSenateRequiredStakePercentage<T>>;

    // ============================
    // ==== Staking + Accounts ====
    // ============================
    #[pallet::type_value]
    pub fn TotalSupply<T: Config>() -> u64 {
        21_000_000_000_000_000 // Rao => 21_000_000 Tao
    }
    #[pallet::type_value]
    pub fn DefaultDefaultTake<T: Config>() -> u16 {
        T::InitialDefaultTake::get()
    }
    #[pallet::type_value]
    pub fn DefaultAccountTake<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultStakesPerInterval<T: Config>() -> (u64, u64) {
        (0, 0)
    }
    #[pallet::type_value]
    pub fn DefaultBlockEmission<T: Config>() -> u64 {
        1_000_000_000
    }
    #[pallet::type_value]
    pub fn DefaultAllowsDelegation<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    pub fn DefaultTotalIssuance<T: Config>() -> u64 {
        T::InitialIssuance::get()
    }
    #[pallet::type_value]
    pub fn DefaultAccount<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap()
    }
    #[pallet::type_value]
    pub fn DefaultTargetStakesPerInterval<T: Config>() -> u64 {
        T::InitialTargetStakesPerInterval::get()
    }
    #[pallet::type_value]
    pub fn DefaultStakeInterval<T: Config>() -> u64 {
        360
    }

    #[pallet::storage] // --- ITEM ( total_stake )
    pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;
    #[pallet::storage] // --- ITEM ( default_take )
    pub type DefaultTake<T> = StorageValue<_, u16, ValueQuery, DefaultDefaultTake<T>>;
    #[pallet::storage] // --- ITEM ( global_block_emission )
    pub type BlockEmission<T> = StorageValue<_, u64, ValueQuery, DefaultBlockEmission<T>>;
    #[pallet::storage] // --- ITEM ( total_issuance )
    pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;
    #[pallet::storage] // --- ITEM (target_stakes_per_interval)
    pub type TargetStakesPerInterval<T> =
        StorageValue<_, u64, ValueQuery, DefaultTargetStakesPerInterval<T>>;
    #[pallet::storage] // --- ITEM (default_stake_interval)
    pub type StakeInterval<T> = StorageValue<_, u64, ValueQuery, DefaultStakeInterval<T>>;
    #[pallet::storage] // --- MAP ( hot ) --> stake | Returns the total amount of stake under a hotkey.
    pub type TotalHotkeyStake<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
    #[pallet::storage] // --- MAP ( cold ) --> stake | Returns the total amount of stake under a coldkey.
    pub type TotalColdkeyStake<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
    #[pallet::storage]
    // --- MAP (hot) --> stake | Returns a tuple (u64: stakes, u64: block_number)
    pub type TotalHotkeyStakesThisInterval<T: Config> =
        StorageMap<_, Identity, T::AccountId, (u64, u64), ValueQuery, DefaultStakesPerInterval<T>>;

    #[pallet::storage] // --- MAP ( hot ) --> cold | Returns the controlling coldkey for a hotkey.
    pub type Owner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultAccount<T>>;
    #[pallet::storage] // --- MAP ( hot ) --> take | Returns the hotkey delegation take. And signals that this key is open for delegation.
    pub type Delegates<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultDefaultTake<T>>;
    #[pallet::storage] // --- DMAP ( hot, cold ) --> stake | Returns the stake under a coldkey prefixed by hotkey.
    pub type Stake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        T::AccountId,
        u64,
        ValueQuery,
        DefaultAccountTake<T>,
    >;

    // =====================================
    // ==== Difficulty / Registrations =====
    // =====================================
    #[pallet::type_value]
    pub fn DefaultLastAdjustmentBlock<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultRegistrationsThisBlock<T: Config>() -> u16 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultBurn<T: Config>() -> u64 {
        T::InitialBurn::get()
    }
    #[pallet::type_value]
    pub fn DefaultMinBurn<T: Config>() -> u64 {
        T::InitialMinBurn::get()
    }
    #[pallet::type_value]
    pub fn DefaultMaxBurn<T: Config>() -> u64 {
        T::InitialMaxBurn::get()
    }
    #[pallet::type_value]
    pub fn DefaultDifficulty<T: Config>() -> u64 {
        T::InitialDifficulty::get()
    }
    #[pallet::type_value]
    pub fn DefaultMinDifficulty<T: Config>() -> u64 {
        T::InitialMinDifficulty::get()
    }
    #[pallet::type_value]
    pub fn DefaultMaxDifficulty<T: Config>() -> u64 {
        T::InitialMaxDifficulty::get()
    }
    #[pallet::type_value]
    pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 {
        T::InitialMaxRegistrationsPerBlock::get()
    }
    #[pallet::type_value]
    pub fn DefaultRAORecycledForRegistration<T: Config>() -> u64 {
        T::InitialRAORecycledForRegistration::get()
    }

    #[pallet::storage] // ---- StorageItem Global Used Work.
    pub type UsedWork<T: Config> = StorageMap<_, Identity, Vec<u8>, u64, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> Burn
    pub type Burn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBurn<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Difficulty
    pub type Difficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultDifficulty<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MinBurn
    pub type MinBurn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMinBurn<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MaxBurn
    pub type MaxBurn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxBurn<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MinDifficulty
    pub type MinDifficulty<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMinDifficulty<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> MaxDifficulty
    pub type MaxDifficulty<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxDifficulty<T>>;
    #[pallet::storage] // --- MAP ( netuid ) -->  Block at last adjustment.
    pub type LastAdjustmentBlock<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastAdjustmentBlock<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Registrations of this Block.
    pub type RegistrationsThisBlock<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRegistrationsThisBlock<T>>;
    #[pallet::storage] // --- ITEM( global_max_registrations_per_block )
    pub type MaxRegistrationsPerBlock<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T>>;
    #[pallet::storage] // --- MAP ( netuid, global_RAO_recycled_for_registration )
    pub type RAORecycledForRegistration<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultRAORecycledForRegistration<T>>;

    // ==============================
    // ==== Subnetworks Storage =====
    // ==============================
    #[pallet::type_value]
    pub fn DefaultN<T: Config>() -> u16 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultModality<T: Config>() -> u16 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultHotkeys<T: Config>() -> Vec<u16> {
        vec![]
    }
    #[pallet::type_value]
    pub fn DefaultNeworksAdded<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    pub fn DefaultIsNetworkMember<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    pub fn DefaultRegistrationAllowed<T: Config>() -> bool {
        false
    }
    #[pallet::type_value]
    pub fn DefaultNetworkRegisteredAt<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultNetworkImmunityPeriod<T: Config>() -> u64 {
        T::InitialNetworkImmunityPeriod::get()
    }
    #[pallet::type_value]
    pub fn DefaultNetworkLastRegistered<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultNetworkMinAllowedUids<T: Config>() -> u16 {
        T::InitialNetworkMinAllowedUids::get()
    }
    #[pallet::type_value]
    pub fn DefaultNetworkMinLockCost<T: Config>() -> u64 {
        T::InitialNetworkMinLockCost::get()
    }
    #[pallet::type_value]
    pub fn DefaultNetworkLockReductionInterval<T: Config>() -> u64 {
        T::InitialNetworkLockReductionInterval::get()
    }
    #[pallet::type_value]
    pub fn DefaultSubnetOwnerCut<T: Config>() -> u16 {
        T::InitialSubnetOwnerCut::get()
    }
    #[pallet::type_value]
    pub fn DefaultSubnetLimit<T: Config>() -> u16 {
        T::InitialSubnetLimit::get()
    }
    #[pallet::type_value]
    pub fn DefaultNetworkRateLimit<T: Config>() -> u64 {
        if cfg!(feature = "pow-faucet") {
            return 0;
        }

        T::InitialNetworkRateLimit::get()
    }

    #[pallet::storage] // --- ITEM( maximum_number_of_networks )
    pub type SubnetLimit<T> = StorageValue<_, u16, ValueQuery, DefaultSubnetLimit<T>>;
    #[pallet::storage] // --- ITEM( total_number_of_existing_networks )
    pub type TotalNetworks<T> = StorageValue<_, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> subnetwork_n (Number of UIDs in the network).
    pub type SubnetworkN<T: Config> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultN<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> modality   TEXT: 0, IMAGE: 1, TENSOR: 2
    pub type NetworkModality<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultModality<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> network_is_added
    pub type NetworksAdded<T: Config> =
        StorageMap<_, Identity, u16, bool, ValueQuery, DefaultNeworksAdded<T>>;
    #[pallet::storage] // --- DMAP ( hotkey, netuid ) --> bool
    pub type IsNetworkMember<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        u16,
        bool,
        ValueQuery,
        DefaultIsNetworkMember<T>,
    >;
    #[pallet::storage] // --- MAP ( netuid ) --> network_registration_allowed
    pub type NetworkRegistrationAllowed<T: Config> =
        StorageMap<_, Identity, u16, bool, ValueQuery, DefaultRegistrationAllowed<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> network_pow_allowed
    pub type NetworkPowRegistrationAllowed<T: Config> =
        StorageMap<_, Identity, u16, bool, ValueQuery, DefaultRegistrationAllowed<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> block_created
    pub type NetworkRegisteredAt<T: Config> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultNetworkRegisteredAt<T>>;
    #[pallet::storage] // ITEM( network_immunity_period )
    pub type NetworkImmunityPeriod<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkImmunityPeriod<T>>;
    #[pallet::storage] // ITEM( network_last_registered_block )
    pub type NetworkLastRegistered<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkLastRegistered<T>>;
    #[pallet::storage] // ITEM( network_min_allowed_uids )
    pub type NetworkMinAllowedUids<T> =
        StorageValue<_, u16, ValueQuery, DefaultNetworkMinAllowedUids<T>>;
    #[pallet::storage] // ITEM( min_network_lock_cost )
    pub type NetworkMinLockCost<T> = StorageValue<_, u64, ValueQuery, DefaultNetworkMinLockCost<T>>;
    #[pallet::storage] // ITEM( last_network_lock_cost )
    pub type NetworkLastLockCost<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkMinLockCost<T>>;
    #[pallet::storage] // ITEM( network_lock_reduction_interval )
    pub type NetworkLockReductionInterval<T> =
        StorageValue<_, u64, ValueQuery, DefaultNetworkLockReductionInterval<T>>;
    #[pallet::storage] // ITEM( subnet_owner_cut )
    pub type SubnetOwnerCut<T> = StorageValue<_, u16, ValueQuery, DefaultSubnetOwnerCut<T>>;
    #[pallet::storage] // ITEM( network_rate_limit )
    pub type NetworkRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultNetworkRateLimit<T>>;

    // ==============================
    // ==== Subnetwork Features =====
    // ==============================
    #[pallet::type_value]
    pub fn DefaultEmissionValues<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultPendingEmission<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultBlocksSinceLastStep<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultLastMechanismStepBlock<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultSubnetOwner<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()
    }
    #[pallet::type_value]
    pub fn DefaultSubnetLocked<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultTempo<T: Config>() -> u16 {
        T::InitialTempo::get()
    }

    #[pallet::storage] // --- MAP ( netuid ) --> tempo
    pub type Tempo<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTempo<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> emission_values
    pub type EmissionValues<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultEmissionValues<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> pending_emission
    pub type PendingEmission<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultPendingEmission<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> blocks_since_last_step
    pub type BlocksSinceLastStep<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBlocksSinceLastStep<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> last_mechanism_step_block
    pub type LastMechansimStepBlock<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastMechanismStepBlock<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> subnet_owner
    pub type SubnetOwner<T: Config> =
        StorageMap<_, Identity, u16, T::AccountId, ValueQuery, DefaultSubnetOwner<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> subnet_locked
    pub type SubnetLocked<T: Config> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultSubnetLocked<T>>;

    // =================================
    // ==== Axon / Promo Endpoints =====
    // =================================

    // --- Struct for Axon.
    pub type AxonInfoOf = AxonInfo;

    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct AxonInfo {
        pub block: u64,       // --- Axon serving block.
        pub version: u32,     // --- Axon version
        pub ip: u128,         // --- Axon u128 encoded ip address of type v6 or v4.
        pub port: u16,        // --- Axon u16 encoded port.
        pub ip_type: u8,      // --- Axon ip type, 4 for ipv4 and 6 for ipv6.
        pub protocol: u8,     // --- Axon protocol. TCP, UDP, other.
        pub placeholder1: u8, // --- Axon proto placeholder 1.
        pub placeholder2: u8, // --- Axon proto placeholder 2.
    }

    // --- Struct for Prometheus.
    pub type PrometheusInfoOf = PrometheusInfo;
    #[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
    pub struct PrometheusInfo {
        pub block: u64,   // --- Prometheus serving block.
        pub version: u32, // --- Prometheus version.
        pub ip: u128,     // --- Prometheus u128 encoded ip address of type v6 or v4.
        pub port: u16,    // --- Prometheus u16 encoded port.
        pub ip_type: u8,  // --- Prometheus ip type, 4 for ipv4 and 6 for ipv6.
    }

    // Rate limiting
    #[pallet::type_value]
    pub fn DefaultTxRateLimit<T: Config>() -> u64 {
        T::InitialTxRateLimit::get()
    }
    #[pallet::type_value]
    pub fn DefaultLastTxBlock<T: Config>() -> u64 {
        0
    }

    #[pallet::storage] // --- ITEM ( tx_rate_limit )
    pub(super) type TxRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultTxRateLimit<T>>;
    #[pallet::storage] // --- MAP ( key ) --> last_block
    pub(super) type LastTxBlock<T: Config> =
        StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock<T>>;

    #[pallet::type_value]
    pub fn DefaultServingRateLimit<T: Config>() -> u64 {
        T::InitialServingRateLimit::get()
    }

    #[pallet::storage] // --- MAP ( netuid ) --> serving_rate_limit
    pub type ServingRateLimit<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultServingRateLimit<T>>;
    #[pallet::storage] // --- MAP ( netuid, hotkey ) --> axon_info
    pub(super) type Axons<T: Config> =
        StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, AxonInfoOf, OptionQuery>;
    #[pallet::storage] // --- MAP ( netuid, hotkey ) --> prometheus_info
    pub(super) type Prometheus<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Blake2_128Concat,
        T::AccountId,
        PrometheusInfoOf,
        OptionQuery,
    >;

    // =======================================
    // ==== Subnetwork Hyperparam storage ====
    // =======================================
    #[pallet::type_value]
    pub fn DefaultWeightsSetRateLimit<T: Config>() -> u64 {
        100
    }
    #[pallet::type_value]
    pub fn DefaultBlockAtRegistration<T: Config>() -> u64 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultRho<T: Config>() -> u16 {
        T::InitialRho::get()
    }
    #[pallet::type_value]
    pub fn DefaultKappa<T: Config>() -> u16 {
        T::InitialKappa::get()
    }
    #[pallet::type_value]
    pub fn DefaultMaxAllowedUids<T: Config>() -> u16 {
        T::InitialMaxAllowedUids::get()
    }
    #[pallet::type_value]
    pub fn DefaultImmunityPeriod<T: Config>() -> u16 {
        T::InitialImmunityPeriod::get()
    }
    #[pallet::type_value]
    pub fn DefaultActivityCutoff<T: Config>() -> u16 {
        T::InitialActivityCutoff::get()
    }
    #[pallet::type_value]
    pub fn DefaultMaxWeightsLimit<T: Config>() -> u16 {
        T::InitialMaxWeightsLimit::get()
    }
    #[pallet::type_value]
    pub fn DefaultWeightsVersionKey<T: Config>() -> u64 {
        T::InitialWeightsVersionKey::get()
    }
    #[pallet::type_value]
    pub fn DefaultMinAllowedWeights<T: Config>() -> u16 {
        T::InitialMinAllowedWeights::get()
    }
    #[pallet::type_value]
    pub fn DefaultMaxAllowedValidators<T: Config>() -> u16 {
        T::InitialMaxAllowedValidators::get()
    }
    #[pallet::type_value]
    pub fn DefaultAdjustmentInterval<T: Config>() -> u16 {
        T::InitialAdjustmentInterval::get()
    }
    #[pallet::type_value]
    pub fn DefaultBondsMovingAverage<T: Config>() -> u64 {
        T::InitialBondsMovingAverage::get()
    }
    #[pallet::type_value]
    pub fn DefaultValidatorPruneLen<T: Config>() -> u64 {
        T::InitialValidatorPruneLen::get()
    }
    #[pallet::type_value]
    pub fn DefaultScalingLawPower<T: Config>() -> u16 {
        T::InitialScalingLawPower::get()
    }
    #[pallet::type_value]
    pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u16 {
        T::InitialTargetRegistrationsPerInterval::get()
    }
    #[pallet::type_value]
    pub fn DefaultAdjustmentAlpha<T: Config>() -> u64 {
        T::InitialAdjustmentAlpha::get()
    }
    #[pallet::type_value]
    pub fn DefaultWeightsMinStake<T: Config>() -> u64 {
        0
    }

    #[pallet::storage] // ITEM( weights_min_stake )
    pub type WeightsMinStake<T> = StorageValue<_, u64, ValueQuery, DefaultWeightsMinStake<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Rho
    pub type Rho<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRho<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> Kappa
    pub type Kappa<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultKappa<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> uid, we use to record uids to prune at next epoch.
    pub type NeuronsToPruneAtNextEpoch<T: Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> registrations_this_interval
    pub type RegistrationsThisInterval<T: Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> pow_registrations_this_interval
    pub type POWRegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> burn_registrations_this_interval
    pub type BurnRegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> max_allowed_uids
    pub type MaxAllowedUids<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedUids<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> immunity_period
    pub type ImmunityPeriod<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultImmunityPeriod<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> activity_cutoff
    pub type ActivityCutoff<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultActivityCutoff<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> max_weight_limit
    pub type MaxWeightsLimit<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxWeightsLimit<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> weights_version_key
    pub type WeightsVersionKey<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightsVersionKey<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MinAllowedWeights<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> max_allowed_validators
    pub type MaxAllowedValidators<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedValidators<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> adjustment_interval
    pub type AdjustmentInterval<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultAdjustmentInterval<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> bonds_moving_average
    pub type BondsMovingAverage<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBondsMovingAverage<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> weights_set_rate_limit
    pub type WeightsSetRateLimit<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightsSetRateLimit<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> validator_prune_len
    pub type ValidatorPruneLen<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultValidatorPruneLen<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> scaling_law_power
    pub type ScalingLawPower<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultScalingLawPower<T>>;
    #[pallet::storage] // --- MAP ( netuid ) --> target_registrations_this_interval
    pub type TargetRegistrationsPerInterval<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTargetRegistrationsPerInterval<T>>;
    #[pallet::storage] // --- DMAP ( netuid, uid ) --> block_at_registration
    pub type BlockAtRegistration<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Identity,
        u16,
        u64,
        ValueQuery,
        DefaultBlockAtRegistration<T>,
    >;
    #[pallet::storage] // --- DMAP ( netuid ) --> adjustment_alpha
    pub type AdjustmentAlpha<T: Config> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultAdjustmentAlpha<T>>;

    // =======================================
    // ==== Subnetwork Consensus Storage  ====
    // =======================================
    #[pallet::type_value]
    pub fn EmptyU16Vec<T: Config>() -> Vec<u16> {
        vec![]
    }
    #[pallet::type_value]
    pub fn EmptyU64Vec<T: Config>() -> Vec<u64> {
        vec![]
    }
    #[pallet::type_value]
    pub fn EmptyBoolVec<T: Config>() -> Vec<bool> {
        vec![]
    }
    #[pallet::type_value]
    pub fn DefaultBonds<T: Config>() -> Vec<(u16, u16)> {
        vec![]
    }
    #[pallet::type_value]
    pub fn DefaultWeights<T: Config>() -> Vec<(u16, u16)> {
        vec![]
    }
    #[pallet::type_value]
    pub fn DefaultKey<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()
    }

    #[pallet::storage] // --- DMAP ( netuid, hotkey ) --> uid
    pub(super) type Uids<T: Config> =
        StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, u16, OptionQuery>;
    #[pallet::storage] // --- DMAP ( netuid, uid ) --> hotkey
    pub(super) type Keys<T: Config> =
        StorageDoubleMap<_, Identity, u16, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> (hotkey, se, ve)
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<_, Identity, u16, Vec<(T::AccountId, u64, u64)>, OptionQuery>;

    #[pallet::storage] // --- DMAP ( netuid ) --> active
    pub(super) type Active<T: Config> =
        StorageMap<_, Identity, u16, Vec<bool>, ValueQuery, EmptyBoolVec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> rank
    pub(super) type Rank<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> trust
    pub(super) type Trust<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> consensus
    pub(super) type Consensus<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> incentive
    pub(super) type Incentive<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> dividends
    pub(super) type Dividends<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> emission
    pub(super) type Emission<T: Config> =
        StorageMap<_, Identity, u16, Vec<u64>, ValueQuery, EmptyU64Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> last_update
    pub(super) type LastUpdate<T: Config> =
        StorageMap<_, Identity, u16, Vec<u64>, ValueQuery, EmptyU64Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> validator_trust
    pub(super) type ValidatorTrust<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> pruning_scores
    pub(super) type PruningScores<T: Config> =
        StorageMap<_, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
    #[pallet::storage] // --- DMAP ( netuid ) --> validator_permit
    pub(super) type ValidatorPermit<T: Config> =
        StorageMap<_, Identity, u16, Vec<bool>, ValueQuery, EmptyBoolVec<T>>;

    #[pallet::storage] // --- DMAP ( netuid, uid ) --> weights
    pub(super) type Weights<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Identity,
        u16,
        Vec<(u16, u16)>,
        ValueQuery,
        DefaultWeights<T>,
    >;
    #[pallet::storage] // --- DMAP ( netuid, uid ) --> bonds
    pub(super) type Bonds<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Identity,
        u16,
        Vec<(u16, u16)>,
        ValueQuery,
        DefaultBonds<T>,
    >;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/main-docs/build/events-errors/
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // Event documentation should end with an array that provides descriptive names for event
        // parameters. [something, who]
        NetworkAdded(u16, u16), // --- Event created when a new network is added.
        NetworkRemoved(u16),    // --- Event created when a network is removed.
        StakeAdded(T::AccountId, u64), // --- Event created when stake has been transferred from the a coldkey account onto the hotkey staking account.
        StakeRemoved(T::AccountId, u64), // --- Event created when stake has been removed from the hotkey staking account onto the coldkey account.
        WeightsSet(u16, u16), // ---- Event created when a caller successfully sets their weights on a subnetwork.
        NeuronRegistered(u16, u16, T::AccountId), // --- Event created when a new neuron account has been registered to the chain.
        BulkNeuronsRegistered(u16, u16), // --- Event created when multiple uids have been concurrently registered.
        BulkBalancesSet(u16, u16),       // --- FIXME: Not used yet
        MaxAllowedUidsSet(u16, u16), // --- Event created when max allowed uids has been set for a subnetwork.
        MaxWeightLimitSet(u16, u16), // --- Event created when the max weight limit has been set for a subnetwork.
        DifficultySet(u16, u64), // --- Event created when the difficulty has been set for a subnet.
        AdjustmentIntervalSet(u16, u16), // --- Event created when the adjustment interval is set for a subnet.
        RegistrationPerIntervalSet(u16, u16), // --- Event created when registration per interval is set for a subnet.
        MaxRegistrationsPerBlockSet(u16, u16), // --- Event created when we set max registrations per block.
        ActivityCutoffSet(u16, u16), // --- Event created when an activity cutoff is set for a subnet.
        RhoSet(u16, u16),            // --- Event created when Rho value is set.
        KappaSet(u16, u16),          // --- Event created when Kappa is set for a subnet.
        MinAllowedWeightSet(u16, u16), // --- Event created when minimum allowed weight is set for a subnet.
        ValidatorPruneLenSet(u16, u64), // --- Event created when the validator pruning length has been set.
        ScalingLawPowerSet(u16, u16), // --- Event created when the scaling law power has been set for a subnet.
        WeightsSetRateLimitSet(u16, u64), // --- Event created when weights set rate limit has been set for a subnet.
        ImmunityPeriodSet(u16, u16), // --- Event created when immunity period is set for a subnet.
        BondsMovingAverageSet(u16, u64), // --- Event created when bonds moving average is set for a subnet.
        MaxAllowedValidatorsSet(u16, u16), // --- Event created when setting the max number of allowed validators on a subnet.
        AxonServed(u16, T::AccountId), // --- Event created when the axon server information is added to the network.
        PrometheusServed(u16, T::AccountId), // --- Event created when the prometheus server information is added to the network.
        EmissionValuesSet(), // --- Event created when emission ratios for all networks is set.
        DelegateAdded(T::AccountId, T::AccountId, u16), // --- Event created to signal that a hotkey has become a delegate.
        DefaultTakeSet(u16), // --- Event created when the default take is set.
        WeightsVersionKeySet(u16, u64), // --- Event created when weights version key is set for a network.
        MinDifficultySet(u16, u64), // --- Event created when setting min difficulty on a network.
        MaxDifficultySet(u16, u64), // --- Event created when setting max difficulty on a network.
        ServingRateLimitSet(u16, u64), // --- Event created when setting the prometheus serving rate limit.
        BurnSet(u16, u64),             // --- Event created when setting burn on a network.
        MaxBurnSet(u16, u64),          // --- Event created when setting max burn on a network.
        MinBurnSet(u16, u64),          // --- Event created when setting min burn on a network.
        TxRateLimitSet(u64),           // --- Event created when setting the transaction rate limit.
        Sudid(DispatchResult),         // --- Event created when a sudo call is done.
        RegistrationAllowed(u16, bool), // --- Event created when registration is allowed/disallowed for a subnet.
        PowRegistrationAllowed(u16, bool), // --- Event created when POW registration is allowed/disallowed for a subnet.
        TempoSet(u16, u16),                // --- Event created when setting tempo on a network
        RAORecycledForRegistrationSet(u16, u64), // Event created when setting the RAO recycled for registration.
        WeightsMinStake(u64), // --- Event created when min stake is set for validators to set weights.
        SenateRequiredStakePercentSet(u64), // Event created when setting the minimum required stake amount for senate registration.
        AdjustmentAlphaSet(u16, u64), // Event created when setting the adjustment alpha on a subnet.
        Faucet(T::AccountId, u64),    // Event created when the faucet it called on the test net.
        SubnetOwnerCutSet(u16),       // Event created when the subnet owner cut is set.
        NetworkRateLimitSet(u64),     // Event created when the network creation rate limit is set.
        NetworkImmunityPeriodSet(u64), // Event created when the network immunity period is set.
        NetworkMinLockCostSet(u64),   // Event created when the network minimum locking cost is set.
        SubnetLimitSet(u16),          // Event created when the maximum number of subnets is set
        NetworkLockCostReductionIntervalSet(u64), // Event created when the lock cost reduction is set
        HotkeySwapped {
            coldkey: T::AccountId,
            old_hotkey: T::AccountId,
            new_hotkey: T::AccountId,
        }, // Event created when a hotkey is swapped
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        NetworkDoesNotExist,                // --- Thrown when the network does not exist.
        NetworkExist,                       // --- Thrown when the network already exists.
        InvalidModality,  // --- Thrown when an invalid modality attempted on serve.
        InvalidIpType, // ---- Thrown when the user tries to serve an axon which is not of type	4 (IPv4) or 6 (IPv6).
        InvalidIpAddress, // --- Thrown when an invalid IP address is passed to the serve function.
        InvalidPort,   // --- Thrown when an invalid port is passed to the serve function.
        NotRegistered, // ---- Thrown when the caller requests setting or removing data from a neuron which does not exist in the active set.
        NonAssociatedColdKey, // ---- Thrown when a stake, unstake or subscribe request is made by a coldkey which is not associated with the hotkey account.
        NotEnoughStaketoWithdraw, // ---- Thrown when the caller requests removing more stake than there exists in the staking account. See: fn remove_stake.
        NotEnoughStakeToSetWeights, // ---- Thrown when the caller requests to set weights but has less than WeightsMinStake
        NotEnoughBalanceToStake, //  ---- Thrown when the caller requests adding more stake than there exists in the cold key account. See: fn add_stake
        BalanceWithdrawalError, // ---- Thrown when the caller tries to add stake, but for some reason the requested amount could not be withdrawn from the coldkey account.
        NoValidatorPermit, // ---- Thrown when the caller attempts to set non-self weights without being a permitted validator.
        WeightVecNotEqualSize, // ---- Thrown when the caller attempts to set the weight keys and values but these vectors have different size.
        DuplicateUids, // ---- Thrown when the caller attempts to set weights with duplicate uids in the weight matrix.
        InvalidUid, // ---- Thrown when a caller attempts to set weight to at least one uid that does not exist in the metagraph.
        NotSettingEnoughWeights, // ---- Thrown when the dispatch attempts to set weights on chain with fewer elements than are allowed.
        TooManyRegistrationsThisBlock, // ---- Thrown when registrations this block exceeds allowed number.
        AlreadyRegistered, // ---- Thrown when the caller requests registering a neuron which already exists in the active set.
        InvalidWorkBlock, // ---- Thrown if the supplied pow hash block is in the future or negative.
        InvalidDifficulty, // ---- Thrown if the supplied pow hash block does not meet the network difficulty.
        InvalidSeal, // ---- Thrown if the supplied pow hash seal does not match the supplied work.
        MaxAllowedUIdsNotAllowed, // ---  Thrown if the value is invalid for MaxAllowedUids.
        CouldNotConvertToBalance, // ---- Thrown when the dispatch attempts to convert between a u64 and T::balance but the call fails.
        CouldNotConvertToU64, // -- Thrown when the dispatch attempts to convert from a T::Balance to a u64 but the call fails.
        StakeAlreadyAdded, // --- Thrown when the caller requests adding stake for a hotkey to the total stake which already added.
        MaxWeightExceeded, // --- Thrown when the dispatch attempts to set weights on chain with where any normalized weight is more than MaxWeightLimit.
        StorageValueOutOfRange, // --- Thrown when the caller attempts to set a storage value outside of its allowed range.
        TempoHasNotSet,         // --- Thrown when tempo has not set.
        InvalidTempo,           // --- Thrown when tempo is not valid.
        EmissionValuesDoesNotMatchNetworks, // --- Thrown when number or received emission rates does not match number of networks.
        InvalidEmissionValues, // --- Thrown when emission ratios are not valid (did not sum up to 10^9).
        AlreadyDelegate, // --- Thrown if the hotkey attempts to become delegate when they are already.
        SettingWeightsTooFast, // --- Thrown if the hotkey attempts to set weights twice within net_tempo/2 blocks.
        IncorrectNetworkVersionKey, // --- Thrown when a validator attempts to set weights from a validator with incorrect code base key.
        ServingRateLimitExceeded, // --- Thrown when an axon or prometheus serving exceeds the rate limit for a registered neuron.
        BalanceSetError,          // --- Thrown when an error occurs while setting a balance.
        MaxAllowedUidsExceeded, // --- Thrown when number of accounts going to be registered exceeds MaxAllowedUids for the network.
        TooManyUids, // ---- Thrown when the caller attempts to set weights with more uids than allowed.
        TxRateLimitExceeded, // --- Thrown when a transactor exceeds the rate limit for transactions.
        StakeRateLimitExceeded, // --- Thrown when a transactor exceeds the rate limit for stakes.
        UnstakeRateLimitExceeded, // --- Thrown when a transactor exceeds the rate limit for unstakes.
        RegistrationDisabled,     // --- Thrown when registration is disabled
        TooManyRegistrationsThisInterval, // --- Thrown when registration attempt exceeds allowed in interval
        BenchmarkingOnly, // --- Thrown when a function is only available for benchmarking
        HotkeyOriginMismatch, // --- Thrown when the hotkey passed is not the origin, but it should be
        // Senate errors
        SenateMember, // --- Thrown when attempting to do something to a senate member that is limited
        NotSenateMember, // --- Thrown when a hotkey attempts to do something only senate members can do
        AlreadySenateMember, // --- Thrown when a hotkey attempts to join the senate while already being a member
        BelowStakeThreshold, // --- Thrown when a hotkey attempts to join the senate without enough stake
        NotDelegate, // --- Thrown when a hotkey attempts to join the senate without being a delegate first
        IncorrectNetuidsLength, // --- Thrown when an incorrect amount of Netuids are passed as input
        FaucetDisabled,         // --- Thrown when the faucet is disabled
        NotSubnetOwner,
        OperationNotPermittedOnRootSubnet,
        StakeTooLowForRoot, // --- Thrown when a hotkey attempts to join the root subnet with too little stake
        AllNetworksInImmunity, // --- Thrown when all subnets are in the immunity period
        NotEnoughBalance,
    }

    // ==================
    // ==== Genesis =====
    // ==================

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub stakes: Vec<(T::AccountId, Vec<(T::AccountId, (u64, u16))>)>,
        pub balances_issuance: u64,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                stakes: Default::default(),
                balances_issuance: 0,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set initial total issuance from balances
            TotalIssuance::<T>::put(self.balances_issuance);

            // Subnet config values
            let netuid: u16 = 3;
            let tempo = 99;
            let max_uids = 4096;

            // The functions for initializing new networks/setting defaults cannot be run directly from genesis functions like extrinsics would
            // --- Set this network uid to alive.
            NetworksAdded::<T>::insert(netuid, true);

            // --- Fill tempo memory item.
            Tempo::<T>::insert(netuid, tempo);

            // --- Fill modality item.
            // Only modality 0 exists (text)
            NetworkModality::<T>::insert(netuid, 0);

            // Make network parameters explicit.
            if !Tempo::<T>::contains_key(netuid) {
                Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));
            }
            if !Kappa::<T>::contains_key(netuid) {
                Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));
            }
            if !Difficulty::<T>::contains_key(netuid) {
                Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));
            }
            if !MaxAllowedUids::<T>::contains_key(netuid) {
                MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));
            }
            if !ImmunityPeriod::<T>::contains_key(netuid) {
                ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));
            }
            if !ActivityCutoff::<T>::contains_key(netuid) {
                ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));
            }
            if !EmissionValues::<T>::contains_key(netuid) {
                EmissionValues::<T>::insert(netuid, EmissionValues::<T>::get(netuid));
            }
            if !MaxWeightsLimit::<T>::contains_key(netuid) {
                MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));
            }
            if !MinAllowedWeights::<T>::contains_key(netuid) {
                MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid));
            }
            if !RegistrationsThisInterval::<T>::contains_key(netuid) {
                RegistrationsThisInterval::<T>::insert(
                    netuid,
                    RegistrationsThisInterval::<T>::get(netuid),
                );
            }
            if !POWRegistrationsThisInterval::<T>::contains_key(netuid) {
                POWRegistrationsThisInterval::<T>::insert(
                    netuid,
                    POWRegistrationsThisInterval::<T>::get(netuid),
                );
            }
            if !BurnRegistrationsThisInterval::<T>::contains_key(netuid) {
                BurnRegistrationsThisInterval::<T>::insert(
                    netuid,
                    BurnRegistrationsThisInterval::<T>::get(netuid),
                );
            }

            // Set max allowed uids
            MaxAllowedUids::<T>::insert(netuid, max_uids);

            let mut next_uid = 0;

            for (coldkey, hotkeys) in self.stakes.iter() {
                for (hotkey, stake_uid) in hotkeys.iter() {
                    let (stake, uid) = stake_uid;

                    // Expand Yuma Consensus with new position.
                    Rank::<T>::mutate(netuid, |v| v.push(0));
                    Trust::<T>::mutate(netuid, |v| v.push(0));
                    Active::<T>::mutate(netuid, |v| v.push(true));
                    Emission::<T>::mutate(netuid, |v| v.push(0));
                    Consensus::<T>::mutate(netuid, |v| v.push(0));
                    Incentive::<T>::mutate(netuid, |v| v.push(0));
                    Dividends::<T>::mutate(netuid, |v| v.push(0));
                    LastUpdate::<T>::mutate(netuid, |v| v.push(0));
                    PruningScores::<T>::mutate(netuid, |v| v.push(0));
                    ValidatorTrust::<T>::mutate(netuid, |v| v.push(0));
                    ValidatorPermit::<T>::mutate(netuid, |v| v.push(false));

                    // Insert account information.
                    Keys::<T>::insert(netuid, uid, hotkey.clone()); // Make hotkey - uid association.
                    Uids::<T>::insert(netuid, hotkey.clone(), uid); // Make uid - hotkey association.
                    BlockAtRegistration::<T>::insert(netuid, uid, 0); // Fill block at registration.
                    IsNetworkMember::<T>::insert(hotkey.clone(), netuid, true); // Fill network is member.

                    // Fill stake information.
                    Owner::<T>::insert(hotkey.clone(), coldkey.clone());

                    TotalHotkeyStake::<T>::insert(hotkey.clone(), stake);
                    TotalColdkeyStake::<T>::insert(
                        coldkey.clone(),
                        TotalColdkeyStake::<T>::get(coldkey).saturating_add(*stake),
                    );

                    // Update total issuance value
                    TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add(*stake));

                    Stake::<T>::insert(hotkey.clone(), coldkey.clone(), stake);

                    next_uid += 1;
                }
            }

            // Set correct length for Subnet neurons
            SubnetworkN::<T>::insert(netuid, next_uid);

            // --- Increase total network count.
            TotalNetworks::<T>::mutate(|n| *n += 1);

            // Get the root network uid.
            let root_netuid: u16 = 0;

            // Set the root network as added.
            NetworksAdded::<T>::insert(root_netuid, true);

            // Increment the number of total networks.
            TotalNetworks::<T>::mutate(|n| *n += 1);

            // Set the number of validators to 1.
            SubnetworkN::<T>::insert(root_netuid, 0);

            // Set the maximum number to the number of senate members.
            MaxAllowedUids::<T>::insert(root_netuid, 64u16);

            // Set the maximum number to the number of validators to all members.
            MaxAllowedValidators::<T>::insert(root_netuid, 64u16);

            // Set the min allowed weights to zero, no weights restrictions.
            MinAllowedWeights::<T>::insert(root_netuid, 0);

            // Set the max weight limit to infitiy, no weight restrictions.
            MaxWeightsLimit::<T>::insert(root_netuid, u16::MAX);

            // Add default root tempo.
            Tempo::<T>::insert(root_netuid, 100);

            // Set the root network as open.
            NetworkRegistrationAllowed::<T>::insert(root_netuid, true);

            // Set target registrations for validators as 1 per block.
            TargetRegistrationsPerInterval::<T>::insert(root_netuid, 1);
        }
    }

    // ================
    // ==== Hooks =====
    // ================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // ---- Called on the initialization of this pallet. (the order of on_finalize calls is determined in the runtime)
        //
        // # Args:
        // 	* 'n': (BlockNumberFor<T>):
        // 		- The number of the block we are initializing.
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            let block_step_result = Self::block_step();
            match block_step_result {
                Ok(_) => {
                    // --- If the block step was successful, return the weight.
                    log::info!("Successfully ran block step.");
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                }
                Err(e) => {
                    // --- If the block step was unsuccessful, return the weight anyway.
                    log::error!("Error while stepping block: {:?}", e);
                    Weight::from_parts(110_634_229_000_u64, 0)
                        .saturating_add(T::DbWeight::get().reads(8304_u64))
                        .saturating_add(T::DbWeight::get().writes(110_u64))
                }
            }
        }

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            // --- Migrate storage
            use crate::migration;
            let mut weight = frame_support::weights::Weight::from_parts(0, 0);

            // Hex encoded foundation coldkey
            let hex = hex_literal::hex![
                "feabaafee293d3b76dae304e2f9d885f77d2b17adab9e17e921b321eccd61c77"
            ];
            weight = weight
                .saturating_add(migration::migrate_to_v1_separate_emission::<T>())
                .saturating_add(migration::migrate_to_v2_fixed_total_stake::<T>())
                .saturating_add(migration::migrate_create_root_network::<T>())
                .saturating_add(migration::migrate_transfer_ownership_to_foundation::<T>(
                    hex,
                ))
                .saturating_add(migration::migrate_delete_subnet_3::<T>())
                .saturating_add(migration::migrate_delete_subnet_21::<T>())
                .saturating_add(migration::migration5_total_issuance::<T>(false));

            weight
        }
    }

    // Dispatchable functions allow users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // --- Sets the caller weights for the incentive mechanism. The call can be
        // made from the hotkey account so is potentially insecure, however, the damage
        // of changing weights is minimal if caught early. This function includes all the
        // checks that the passed weights meet the requirements. Stored as u16s they represent
        // rational values in the range [0,1] which sum to 1 and can be interpreted as
        // probabilities. The specific weights determine how inflation propagates outward
        // from this peer.
        //
        // Note: The 16 bit integers weights should represent 1.0 as the max u16.
        // However, the function normalizes all integers to u16_max anyway. This means that if the sum of all
        // elements is larger or smaller than the amount of elements * u16_max, all elements
        // will be corrected for this deviation.
        //
        // # Args:
        // 	* `origin`: (<T as frame_system::Config>Origin):
        // 		- The caller, a hotkey who wishes to set their weights.
        //
        // 	* `netuid` (u16):
        // 		- The network uid we are setting these weights on.
        //
        // 	* `dests` (Vec<u16>):
        // 		- The edge endpoint for the weight, i.e. j for w_ij.
        //
        // 	* 'weights' (Vec<u16>):
        // 		- The u16 integer encoded weights. Interpreted as rational
        // 		values in the range [0,1]. They must sum to in32::MAX.
        //
        // 	* 'version_key' ( u64 ):
        // 		- The network version key to check if the validator is up to date.
        //
        // # Event:
        // 	* WeightsSet;
        // 		- On successfully setting the weights on chain.
        //
        // # Raises:
        // 	* 'NetworkDoesNotExist':
        // 		- Attempting to set weights on a non-existent network.
        //
        // 	* 'NotRegistered':
        // 		- Attempting to set weights from a non registered account.
        //
        // 	* 'WeightVecNotEqualSize':
        // 		- Attempting to set weights with uids not of same length.
        //
        // 	* 'DuplicateUids':
        // 		- Attempting to set weights with duplicate uids.
        //
        //     * 'TooManyUids':
        // 		- Attempting to set weights above the max allowed uids.
        //
        // 	* 'InvalidUid':
        // 		- Attempting to set weights with invalid uids.
        //
        // 	* 'NotSettingEnoughWeights':
        // 		- Attempting to set weights with fewer weights than min.
        //
        // 	* 'MaxWeightExceeded':
        // 		- Attempting to set weights with max value exceeding limit.
        #[pallet::call_index(0)]
        #[pallet::weight((Weight::from_parts(10_151_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(4104))
		.saturating_add(T::DbWeight::get().writes(2)), DispatchClass::Normal, Pays::No))]
        pub fn set_weights(
            origin: OriginFor<T>,
            netuid: u16,
            dests: Vec<u16>,
            weights: Vec<u16>,
            version_key: u64,
        ) -> DispatchResult {
            Self::do_set_weights(origin, netuid, dests, weights, version_key)
        }

        // --- Sets the key as a delegate.
        //
        // # Args:
        // 	* 'origin': (<T as frame_system::Config>Origin):
        // 		- The signature of the caller's coldkey.
        //
        // 	* 'hotkey' (T::AccountId):
        // 		- The hotkey we are delegating (must be owned by the coldkey.)
        //
        // 	* 'take' (u64):
        // 		- The stake proportion that this hotkey takes from delegations.
        //
        // # Event:
        // 	* DelegateAdded;
        // 		- On successfully setting a hotkey as a delegate.
        //
        // # Raises:
        // 	* 'NotRegistered':
        // 		- The hotkey we are delegating is not registered on the network.
        //
        // 	* 'NonAssociatedColdKey':
        // 		- The hotkey we are delegating is not owned by the calling coldket.
        //
        //
        #[pallet::call_index(1)]
        #[pallet::weight((0, DispatchClass::Normal, Pays::No))]
        pub fn become_delegate(origin: OriginFor<T>, hotkey: T::AccountId) -> DispatchResult {
            Self::do_become_delegate(origin, hotkey, Self::get_default_take())
        }

        // --- Adds stake to a hotkey. The call is made from the
        // coldkey account linked in the hotkey.
        // Only the associated coldkey is allowed to make staking and
        // unstaking requests. This protects the neuron against
        // attacks on its hotkey running in production code.
        //
        // # Args:
        // 	* 'origin': (<T as frame_system::Config>Origin):
        // 		- The signature of the caller's coldkey.
        //
        // 	* 'hotkey' (T::AccountId):
        // 		- The associated hotkey account.
        //
        // 	* 'amount_staked' (u64):
        // 		- The amount of stake to be added to the hotkey staking account.
        //
        // # Event:
        // 	* StakeAdded;
        // 		- On the successfully adding stake to a global account.
        //
        // # Raises:
        // 	* 'CouldNotConvertToBalance':
        // 		- Unable to convert the passed stake value to a balance.
        //
        // 	* 'NotEnoughBalanceToStake':
        // 		- Not enough balance on the coldkey to add onto the global account.
        //
        // 	* 'NonAssociatedColdKey':
        // 		- The calling coldkey is not associated with this hotkey.
        //
        // 	* 'BalanceWithdrawalError':
        // 		- Errors stemming from transaction pallet.
        //
        //
        #[pallet::call_index(2)]
        #[pallet::weight((Weight::from_parts(65_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(8))
		.saturating_add(T::DbWeight::get().writes(6)), DispatchClass::Normal, Pays::No))]
        pub fn add_stake(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            amount_staked: u64,
        ) -> DispatchResult {
            Self::do_add_stake(origin, hotkey, amount_staked)
        }

        // ---- Remove stake from the staking account. The call must be made
        // from the coldkey account attached to the neuron metadata. Only this key
        // has permission to make staking and unstaking requests.
        //
        // # Args:
        // 	* 'origin': (<T as frame_system::Config>Origin):
        // 		- The signature of the caller's coldkey.
        //
        // 	* 'hotkey' (T::AccountId):
        // 		- The associated hotkey account.
        //
        // 	* 'amount_unstaked' (u64):
        // 		- The amount of stake to be added to the hotkey staking account.
        //
        // # Event:
        // 	* StakeRemoved;
        // 		- On the successfully removing stake from the hotkey account.
        //
        // # Raises:
        // 	* 'NotRegistered':
        // 		- Thrown if the account we are attempting to unstake from is non existent.
        //
        // 	* 'NonAssociatedColdKey':
        // 		- Thrown if the coldkey does not own the hotkey we are unstaking from.
        //
        // 	* 'NotEnoughStaketoWithdraw':
        // 		- Thrown if there is not enough stake on the hotkey to withdwraw this amount.
        //
        // 	* 'CouldNotConvertToBalance':
        // 		- Thrown if we could not convert this amount to a balance.
        //
        //
        #[pallet::call_index(3)]
        #[pallet::weight((Weight::from_parts(63_000_000, 0)
		.saturating_add(Weight::from_parts(0, 43991))
		.saturating_add(T::DbWeight::get().reads(14))
		.saturating_add(T::DbWeight::get().writes(9)), DispatchClass::Normal, Pays::No))]
        pub fn remove_stake(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            amount_unstaked: u64,
        ) -> DispatchResult {
            Self::do_remove_stake(origin, hotkey, amount_unstaked)
        }

        // ---- Serves or updates axon /promethteus information for the neuron associated with the caller. If the caller is
        // already registered the metadata is updated. If the caller is not registered this call throws NotRegistered.
        //
        // # Args:
        // 	* 'origin': (<T as frame_system::Config>Origin):
        // 		- The signature of the caller.
        //
        // 	* 'netuid' (u16):
        // 		- The u16 network identifier.
        //
        // 	* 'version' (u64):
        // 		- The bittensor version identifier.
        //
        // 	* 'ip' (u64):
        // 		- The endpoint ip information as a u128 encoded integer.
        //
        // 	* 'port' (u16):
        // 		- The endpoint port information as a u16 encoded integer.
        //
        // 	* 'ip_type' (u8):
        // 		- The endpoint ip version as a u8, 4 or 6.
        //
        // 	* 'protocol' (u8):
        // 		- UDP:1 or TCP:0
        //
        // 	* 'placeholder1' (u8):
        // 		- Placeholder for further extra params.
        //
        // 	* 'placeholder2' (u8):
        // 		- Placeholder for further extra params.
        //
        // # Event:
        // 	* AxonServed;
        // 		- On successfully serving the axon info.
        //
        // # Raises:
        // 	* 'NetworkDoesNotExist':
        // 		- Attempting to set weights on a non-existent network.
        //
        // 	* 'NotRegistered':
        // 		- Attempting to set weights from a non registered account.
        //
        // 	* 'InvalidIpType':
        // 		- The ip type is not 4 or 6.
        //
        // 	* 'InvalidIpAddress':
        // 		- The numerically encoded ip address does not resolve to a proper ip.
        //
        // 	* 'ServingRateLimitExceeded':
        // 		- Attempting to set prometheus information withing the rate limit min.
        //
        #[pallet::call_index(4)]
        #[pallet::weight((Weight::from_parts(19_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(2))
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Normal, Pays::No))]
        pub fn serve_axon(
            origin: OriginFor<T>,
            netuid: u16,
            version: u32,
            ip: u128,
            port: u16,
            ip_type: u8,
            protocol: u8,
            placeholder1: u8,
            placeholder2: u8,
        ) -> DispatchResult {
            Self::do_serve_axon(
                origin,
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2,
            )
        }

        #[pallet::call_index(5)]
        #[pallet::weight((Weight::from_parts(17_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(2))
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Normal, Pays::No))]
        pub fn serve_prometheus(
            origin: OriginFor<T>,
            netuid: u16,
            version: u32,
            ip: u128,
            port: u16,
            ip_type: u8,
        ) -> DispatchResult {
            Self::do_serve_prometheus(origin, netuid, version, ip, port, ip_type)
        }

        // ---- Registers a new neuron to the subnetwork.
        //
        // # Args:
        // 	* 'origin': (<T as frame_system::Config>Origin):
        // 		- The signature of the calling hotkey.
        //
        // 	* 'netuid' (u16):
        // 		- The u16 network identifier.
        //
        // 	* 'block_number' ( u64 ):
        // 		- Block hash used to prove work done.
        //
        // 	* 'nonce' ( u64 ):
        // 		- Positive integer nonce used in POW.
        //
        // 	* 'work' ( Vec<u8> ):
        // 		- Vector encoded bytes representing work done.
        //
        // 	* 'hotkey' ( T::AccountId ):
        // 		- Hotkey to be registered to the network.
        //
        // 	* 'coldkey' ( T::AccountId ):
        // 		- Associated coldkey account.
        //
        // # Event:
        // 	* NeuronRegistered;
        // 		- On successfully registereing a uid to a neuron slot on a subnetwork.
        //
        // # Raises:
        // 	* 'NetworkDoesNotExist':
        // 		- Attempting to registed to a non existent network.
        //
        // 	* 'TooManyRegistrationsThisBlock':
        // 		- This registration exceeds the total allowed on this network this block.
        //
        // 	* 'AlreadyRegistered':
        // 		- The hotkey is already registered on this network.
        //
        // 	* 'InvalidWorkBlock':
        // 		- The work has been performed on a stale, future, or non existent block.
        //
        // 	* 'InvalidDifficulty':
        // 		- The work does not match the difficutly.
        //
        // 	* 'InvalidSeal':
        // 		- The seal is incorrect.
        //
        #[pallet::call_index(6)]
        #[pallet::weight((Weight::from_parts(91_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(27))
		.saturating_add(T::DbWeight::get().writes(22)), DispatchClass::Normal, Pays::No))]
        pub fn register(
            origin: OriginFor<T>,
            netuid: u16,
            block_number: u64,
            nonce: u64,
            work: Vec<u8>,
            hotkey: T::AccountId,
            coldkey: T::AccountId,
        ) -> DispatchResult {
            Self::do_registration(origin, netuid, block_number, nonce, work, hotkey, coldkey)
        }

        #[pallet::call_index(62)]
        #[pallet::weight((Weight::from_parts(120_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(23))
		.saturating_add(T::DbWeight::get().writes(20)), DispatchClass::Normal, Pays::No))]
        pub fn root_register(origin: OriginFor<T>, hotkey: T::AccountId) -> DispatchResult {
            Self::do_root_register(origin, hotkey)
        }

        #[pallet::call_index(7)]
        #[pallet::weight((Weight::from_parts(89_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(27))
		.saturating_add(T::DbWeight::get().writes(22)), DispatchClass::Normal, Pays::No))]
        pub fn burned_register(
            origin: OriginFor<T>,
            netuid: u16,
            hotkey: T::AccountId,
        ) -> DispatchResult {
            Self::do_burned_registration(origin, netuid, hotkey)
        }

        #[pallet::call_index(70)]
        #[pallet::weight((0, DispatchClass::Operational, Pays::No))]
        pub fn swap_hotkey(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            new_hotkey: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            Self::do_swap_hotkey(origin, &hotkey, &new_hotkey)
        }

        // ---- SUDO ONLY FUNCTIONS ------------------------------------------------------------

        // ==================================
        // ==== Parameter Sudo calls ========
        // ==================================
        // Each function sets the corresponding hyper paramter on the specified network
        // Args:
        // 	* 'origin': (<T as frame_system::Config>Origin):
        // 		- The caller, must be sudo.
        //
        // 	* `netuid` (u16):
        // 		- The network identifier.
        //
        // 	* `hyperparameter value` (u16):
        // 		- The value of the hyper parameter.
        //

        /// Authenticates a council proposal and dispatches a function call with `Root` origin.
        ///
        /// The dispatch origin for this call must be a council majority.
        ///
        /// ## Complexity
        /// - O(1).
        #[pallet::call_index(51)]
        #[pallet::weight((Weight::from_parts(0, 0), DispatchClass::Operational, Pays::No))]
        pub fn sudo(
            origin: OriginFor<T>,
            call: Box<T::SudoRuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            // This is a public call, so we ensure that the origin is a council majority.
            T::CouncilOrigin::ensure_origin(origin)?;

            let result = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
            let error = result.map(|_| ()).map_err(|e| e.error);
            Self::deposit_event(Event::Sudid(error));

            return result;
        }

        /// Authenticates a council proposal and dispatches a function call with `Root` origin.
        /// This function does not check the weight of the call, and instead allows the
        /// user to specify the weight of the call.
        ///
        /// The dispatch origin for this call must be a council majority.
        ///
        /// ## Complexity
        /// - O(1).
        #[pallet::call_index(52)]
        #[pallet::weight((*_weight, call.get_dispatch_info().class, Pays::No))]
        pub fn sudo_unchecked_weight(
            origin: OriginFor<T>,
            call: Box<T::SudoRuntimeCall>,
            _weight: Weight,
        ) -> DispatchResultWithPostInfo {
            // This is a public call, so we ensure that the origin is a council majority.
            T::CouncilOrigin::ensure_origin(origin)?;

            let result = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
            let error = result.map(|_| ()).map_err(|e| e.error);
            Self::deposit_event(Event::Sudid(error));

            return result;
        }

        #[pallet::call_index(55)]
        #[pallet::weight((Weight::from_parts(0, 0)
		.saturating_add(Weight::from_parts(0, 0))
		.saturating_add(T::DbWeight::get().reads(0))
		.saturating_add(T::DbWeight::get().writes(0)), DispatchClass::Operational))]
        pub fn vote(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            proposal: T::Hash,
            #[pallet::compact] index: u32,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            Self::do_vote_root(origin, &hotkey, proposal, index, approve)
        }

        #[pallet::call_index(59)]
        #[pallet::weight((Weight::from_parts(85_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(16))
		.saturating_add(T::DbWeight::get().writes(28)), DispatchClass::Operational, Pays::No))]
        pub fn register_network(origin: OriginFor<T>) -> DispatchResult {
            Self::user_add_network(origin)
        }

        #[pallet::call_index(60)]
        #[pallet::weight((Weight::from_parts(91_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(27))
		.saturating_add(T::DbWeight::get().writes(22)), DispatchClass::Normal, Pays::No))]
        pub fn faucet(
            origin: OriginFor<T>,
            block_number: u64,
            nonce: u64,
            work: Vec<u8>,
        ) -> DispatchResult {
            if cfg!(feature = "pow-faucet") {
                return Self::do_faucet(origin, block_number, nonce, work);
            }

            Err(Error::<T>::FaucetDisabled.into())
        }

        #[pallet::call_index(61)]
        #[pallet::weight((Weight::from_parts(70_000_000, 0)
		.saturating_add(T::DbWeight::get().reads(5))
		.saturating_add(T::DbWeight::get().writes(31)), DispatchClass::Operational, Pays::No))]
        pub fn dissolve_network(origin: OriginFor<T>, netuid: u16) -> DispatchResult {
            Self::user_remove_network(origin, netuid)
        }
    }

    // ---- Subtensor helper functions.
    impl<T: Config> Pallet<T> {
        // --- Returns the transaction priority for setting weights.
        pub fn get_priority_set_weights(hotkey: &T::AccountId, netuid: u16) -> u64 {
            if Uids::<T>::contains_key(netuid, hotkey) {
                let uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey.clone()).unwrap();
                let _stake = Self::get_total_stake_for_hotkey(hotkey);
                let current_block_number: u64 = Self::get_current_block_as_u64();
                let default_priority: u64 =
                    current_block_number - Self::get_last_update_for_uid(netuid, uid);
                return default_priority + u32::max_value() as u64;
            }
            0
        }

        // --- Is the caller allowed to set weights
        pub fn check_weights_min_stake(hotkey: &T::AccountId) -> bool {
            // Blacklist weights transactions for low stake peers.
            Self::get_total_stake_for_hotkey(hotkey) >= Self::get_weights_min_stake()
        }

        pub fn checked_allowed_register(netuid: u16) -> bool {
            if netuid == Self::get_root_netuid() {
                return false;
            }
            if !Self::if_subnet_exist(netuid) {
                return false;
            }
            if !Self::get_network_registration_allowed(netuid) {
                return false;
            }
            if Self::get_registrations_this_block(netuid)
                >= Self::get_max_registrations_per_block(netuid)
            {
                return false;
            }
            if Self::get_registrations_this_interval(netuid)
                >= Self::get_target_registrations_per_interval(netuid) * 3
            {
                return false;
            }
            true
        }
    }
}

/************************************************************
    CallType definition
************************************************************/
#[derive(Debug, PartialEq, Default)]
pub enum CallType {
    SetWeights,
    AddStake,
    RemoveStake,
    AddDelegate,
    Register,
    Serve,
    RegisterNetwork,
    #[default]
    Other,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct SubtensorSignedExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> Default for SubtensorSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config + Send + Sync + TypeInfo> SubtensorSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get_priority_vanilla() -> u64 {
        // Return high priority so that every extrinsic except set_weights function will
        // have a higher priority than the set_weights call
        u64::max_value()
    }

    pub fn get_priority_set_weights(who: &T::AccountId, netuid: u16) -> u64 {
        Pallet::<T>::get_priority_set_weights(who, netuid)
    }

    pub fn check_weights_min_stake(who: &T::AccountId) -> bool {
        Pallet::<T>::check_weights_min_stake(who)
    }

    pub fn u64_to_balance(
        input: u64,
    ) -> Option<
        <<T as Config>::Currency as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance,
    >{
        input.try_into().ok()
    }
}

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for SubtensorSignedExtension<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "SubtensorSignedExtension")
    }
}

impl<T: Config + Send + Sync + TypeInfo> SignedExtension for SubtensorSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "SubtensorSignedExtension";

    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = (CallType, u64, Self::AccountId);

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        match call.is_sub_type() {
            Some(Call::set_weights { netuid, .. }) => {
                if Self::check_weights_min_stake(who) {
                    let priority: u64 = Self::get_priority_set_weights(who, *netuid);
                    Ok(ValidTransaction {
                        priority,
                        longevity: 1,
                        ..Default::default()
                    })
                } else {
                    Err(InvalidTransaction::Call.into())
                }
            }
            Some(Call::add_stake { hotkey, .. }) => {
                let stakes_this_interval = Pallet::<T>::get_stakes_this_interval_for_hotkey(hotkey);
                let max_stakes_per_interval = Pallet::<T>::get_target_stakes_per_interval();

                if stakes_this_interval >= max_stakes_per_interval {
                    return InvalidTransaction::ExhaustsResources.into();
                }

                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
            Some(Call::remove_stake { hotkey, .. }) => {
                let stakes_this_interval = Pallet::<T>::get_stakes_this_interval_for_hotkey(hotkey);
                let max_stakes_per_interval = Pallet::<T>::get_target_stakes_per_interval();

                if stakes_this_interval >= max_stakes_per_interval {
                    return InvalidTransaction::ExhaustsResources.into();
                }

                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
            Some(Call::register { netuid, .. } | Call::burned_register { netuid, .. }) => {
                let registrations_this_interval =
                    Pallet::<T>::get_registrations_this_interval(*netuid);
                let max_registrations_per_interval =
                    Pallet::<T>::get_target_registrations_per_interval(*netuid);
                if registrations_this_interval >= max_registrations_per_interval {
                    // If the registration limit for the interval is exceeded, reject the transaction
                    return InvalidTransaction::ExhaustsResources.into();
                }
                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
            Some(Call::register_network { .. }) => Ok(ValidTransaction {
                priority: Self::get_priority_vanilla(),
                ..Default::default()
            }),
            _ => Ok(ValidTransaction {
                priority: Self::get_priority_vanilla(),
                ..Default::default()
            }),
        }
    }

    // NOTE: Add later when we put in a pre and post dispatch step.
    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        match call.is_sub_type() {
            Some(Call::add_stake { .. }) => {
                let transaction_fee = 100000;
                Ok((CallType::AddStake, transaction_fee, who.clone()))
            }
            Some(Call::remove_stake { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::RemoveStake, transaction_fee, who.clone()))
            }
            Some(Call::set_weights { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::SetWeights, transaction_fee, who.clone()))
            }
            Some(Call::register { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::Register, transaction_fee, who.clone()))
            }
            Some(Call::serve_axon { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::Serve, transaction_fee, who.clone()))
            }
            Some(Call::register_network { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::RegisterNetwork, transaction_fee, who.clone()))
            }
            _ => {
                let transaction_fee = 0;
                Ok((CallType::Other, transaction_fee, who.clone()))
            }
        }
    }

    fn post_dispatch(
        maybe_pre: Option<Self::Pre>,
        _info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        _result: &dispatch::DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        if let Some((call_type, _transaction_fee, _who)) = maybe_pre {
            match call_type {
                CallType::SetWeights => {
                    log::debug!("Not Implemented!");
                }
                CallType::AddStake => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::RemoveStake => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::Register => {
                    log::debug!("Not Implemented!");
                }
                _ => {
                    log::debug!("Not Implemented!");
                }
            }
        }
        Ok(())
    }
}

use frame_support::sp_std::vec;

// TODO: unravel this rats nest, for some reason rustc thinks this is unused even though it's
// used not 25 lines below
#[allow(unused)]
use sp_std::vec::Vec;

/// Trait for managing a membership pallet instance in the runtime
pub trait MemberManagement<AccountId> {
    /// Add member
    fn add_member(account: &AccountId) -> DispatchResult;

    /// Remove a member
    fn remove_member(account: &AccountId) -> DispatchResult;

    /// Swap member
    fn swap_member(remove: &AccountId, add: &AccountId) -> DispatchResult;

    /// Get all members
    fn members() -> Vec<AccountId>;

    /// Check if an account is apart of the set
    fn is_member(account: &AccountId) -> bool;

    /// Get our maximum member count
    fn max_members() -> u32;
}

impl<T> MemberManagement<T> for () {
    /// Add member
    fn add_member(_: &T) -> DispatchResult {
        Ok(())
    }

    // Remove a member
    fn remove_member(_: &T) -> DispatchResult {
        Ok(())
    }

    // Swap member
    fn swap_member(_: &T, _: &T) -> DispatchResult {
        Ok(())
    }

    // Get all members
    fn members() -> Vec<T> {
        vec![]
    }

    // Check if an account is apart of the set
    fn is_member(_: &T) -> bool {
        false
    }

    fn max_members() -> u32 {
        0
    }
}

/// Trait for interacting with collective pallets
pub trait CollectiveInterface<AccountId, Hash, ProposalIndex> {
    /// Remove vote
    fn remove_votes(hotkey: &AccountId) -> Result<bool, DispatchError>;

    fn add_vote(
        hotkey: &AccountId,
        proposal: Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> Result<bool, DispatchError>;
}

impl<T, H, P> CollectiveInterface<T, H, P> for () {
    fn remove_votes(_: &T) -> Result<bool, DispatchError> {
        Ok(true)
    }

    fn add_vote(_: &T, _: H, _: P, _: bool) -> Result<bool, DispatchError> {
        Ok(true)
    }
}
