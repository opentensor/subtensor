#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;



#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::{Currency};
	use frame_support::sp_std::vec;
	use serde::{Serialize, Deserialize};
	use serde_with::{serde_as, DisplayFromStr};
	use frame_support::inherent::Vec;


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// --- Currency type that will be used to place deposits on neurons
		type Currency: Currency<Self::AccountId> + Send + Sync;

		/// =================================
		/// ==== Initial Value Constants ====
		/// =================================
		#[pallet::constant] /// Initial currency issuance.
		type InitialIssuance: Get<u64>;
		#[pallet::constant] /// Initial min allowed weights setting.
		type InitialMinAllowedWeights: Get<u16>;
		#[pallet::constant] /// Initial Emission Ratio
		type InitialEmissionValue: Get<u16>;
		#[pallet::constant] /// Initial max weight limit.
		type InitialMaxWeightsLimit: Get<u16>;
		#[pallet::constant] /// Tempo for each network
		type InitialTempo: Get<u16>;
		#[pallet::constant] /// Initial Difficulty.
		type InitialDifficulty: Get<u64>;
		#[pallet::constant] /// Initial Max Difficulty.
		type InitialMaxDifficulty: Get<u64>;
		#[pallet::constant] /// Initial Min Difficulty.
		type InitialMinDifficulty: Get<u64>;
		#[pallet::constant] /// Initial Burn.
		type InitialBurn: Get<u64>;
		#[pallet::constant] /// Initial Max Burn.
		type InitialMaxBurn: Get<u64>;
		#[pallet::constant] /// Initial Min Burn.
		type InitialMinBurn: Get<u64>;
		#[pallet::constant] /// Initial adjustment interval.
		type InitialAdjustmentInterval: Get<u16>;
		#[pallet::constant] /// Initial bonds moving average.
		type InitialBondsMovingAverage: Get<u64>;
		#[pallet::constant] /// Initial target registrations per interval.
		type InitialTargetRegistrationsPerInterval: Get<u16>;
		#[pallet::constant] /// Initial number of weight cuts in epoch.
		type InitialWeightCuts: Get<u16>;
		#[pallet::constant] /// Rho constant
		type InitialRho: Get<u16>;
		#[pallet::constant] /// Kappa constant
		type InitialKappa: Get<u16>;		
		#[pallet::constant] /// Max UID constant.
		type InitialMaxAllowedUids: Get<u16>;
		#[pallet::constant] /// Default Batch size.
		type InitialValidatorBatchSize: Get<u16>;
		#[pallet::constant] /// Default Batch size.
		type InitialValidatorSequenceLen: Get<u16>;
		#[pallet::constant] /// Default Epoch length.
		type InitialValidatorEpochLen: Get<u16>;
		#[pallet::constant] /// Default Reset length.
		type InitialValidatorEpochsPerReset: Get<u16>;
		#[pallet::constant] /// Initial validator exclude quantile.
		type InitialValidatorExcludeQuantile: Get<u16>;
		#[pallet::constant] /// Initial validator logits divergence penalty/threshold.
		type InitialValidatorLogitsDivergence: Get<u64>;
		#[pallet::constant] /// Initial validator context pruning length.
		type InitialValidatorPruneLen: Get<u64>; 
		#[pallet::constant] /// Initial scaling law power.
		type InitialScalingLawPower: Get<u16>;
		#[pallet::constant] /// Initial synergy scaling law power.
		type InitialSynergyScalingLawPower: Get<u16>;
		#[pallet::constant] /// Immunity Period Constant.
		type InitialImmunityPeriod: Get<u16>;
		#[pallet::constant] /// Activity constant
		type InitialActivityCutoff: Get<u16>;
		#[pallet::constant] /// Initial max registrations per block.
		type InitialMaxRegistrationsPerBlock: Get<u16>;
		#[pallet::constant] /// Initial pruning score for each neuron
		type InitialPruningScore: Get<u16>;	
		#[pallet::constant] /// Initial allowed validators per network.
		type InitialMaxAllowedValidators: Get<u16>;
		#[pallet::constant] /// Initial default delegation take.
		type InitialDefaultTake: Get<u16>;
		#[pallet::constant] /// Initial weights version key.
		type InitialWeightsVersionKey: Get<u64>;
		#[pallet::constant] /// Initial serving rate limit.
		type InitialServingRateLimit: Get<u64>;
	}

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	#[derive(Decode, Encode, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
	pub struct DeAccountId { // allows us to de/serialize the account id as a u8 vec
		#[serde(with = "serde_bytes")]
		id: Vec<u8>
	}

	impl From<Vec<u8>> for DeAccountId {
		fn from(v: Vec<u8>) -> Self {
			DeAccountId {
				id: v.clone()
			}
		}
	}

	/// ============================
	/// ==== Staking + Accounts ====
	/// ============================
	#[pallet::type_value] 
	pub fn DefaultDefaultTake<T: Config>() -> u16 { T::InitialDefaultTake::get() }
	#[pallet::type_value] 
	pub fn DefaultAccountTake<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultBlockEmission<T: Config>() -> u64 {1_000_000_000}
	#[pallet::type_value] 
	pub fn DefaultAllowsDelegation<T: Config>() -> bool { false }
	#[pallet::type_value] 
	pub fn DefaultTotalIssuance<T: Config>() -> u64 { T::InitialIssuance::get() }
	#[pallet::type_value] 
	pub fn DefaultAccount<T: Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()}

	#[pallet::storage] /// --- ITEM ( total_stake )
	pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;
	#[pallet::storage] /// --- ITEM ( default_take )
	pub type DefaultTake<T> = StorageValue<_, u16, ValueQuery, DefaultDefaultTake<T>>;
	#[pallet::storage] /// --- ITEM ( global_block_emission )
	pub type BlockEmission<T> = StorageValue<_, u64, ValueQuery, DefaultBlockEmission<T>>;
	#[pallet::storage] /// --- ITEM ( total_issuance )
	pub type TotalIssuance<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;
	#[pallet::storage] /// --- MAP ( hot ) --> stake | Returns the total amount of stake under a hotkey.
    pub type TotalHotkeyStake<T:Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
	#[pallet::storage] /// --- MAP ( cold ) --> stake | Returns the total amount of stake under a coldkey.
    pub type TotalColdkeyStake<T:Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;
	#[pallet::storage] /// --- MAP ( hot ) --> cold | Returns the controlling coldkey for a hotkey.
    pub type Owner<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultAccount<T>>;
	#[pallet::storage] /// --- MAP ( hot ) --> take | Returns the hotkey delegation take. And signals that this key is open for delegation.
    pub type Delegates<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u16, ValueQuery, DefaultDefaultTake<T>>;
	#[pallet::storage] /// --- DMAP ( hot, cold ) --> stake | Returns the stake under a hotkey prefixed by hotkey.
    pub type Stake<T:Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Identity, T::AccountId, u64, ValueQuery, DefaultAccountTake<T>>;

	/// =====================================
	/// ==== Difficulty / Registrations =====
	/// =====================================
	#[pallet::type_value] 
	pub fn DefaultLastAdjustmentBlock<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultRegistrationsThisBlock<T: Config>() ->  u16 { 0}
	#[pallet::type_value]
	pub fn DefaultBurn<T: Config>() -> u64 { T::InitialBurn::get() }
	#[pallet::type_value]
	pub fn DefaultMinBurn<T: Config>() -> u64 { T::InitialMinBurn::get()  }
	#[pallet::type_value]
	pub fn DefaultMaxBurn<T: Config>() -> u64 { T::InitialMaxBurn::get() }
	#[pallet::type_value]
	pub fn DefaultDifficulty<T: Config>() -> u64 { T::InitialDifficulty::get() }
	#[pallet::type_value]
	pub fn DefaultMinDifficulty<T: Config>() -> u64 { T::InitialMinDifficulty::get()  }
	#[pallet::type_value]
	pub fn DefaultMaxDifficulty<T: Config>() -> u64 { T::InitialMaxDifficulty::get() }
	#[pallet::type_value] 
	pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 { T::InitialMaxRegistrationsPerBlock::get() }

	#[pallet::storage] /// ---- StorageItem Global Used Work.
    pub type UsedWork<T:Config> = StorageMap<_, Identity, Vec<u8>, u64, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> Difficulty
	pub type Burn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBurn<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Difficulty
	pub type Difficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultDifficulty<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> MinBurn
	pub type MinBurn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMinBurn<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> MaxBurn
	pub type MaxBurn<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxBurn<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> MinDifficulty
	pub type MinDifficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMinDifficulty<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> MaxDifficulty
	pub type MaxDifficulty<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxDifficulty<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) -->  Block at last adjustment.
	pub type LastAdjustmentBlock<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastAdjustmentBlock<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Registration this Block.
	pub type RegistrationsThisBlock<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRegistrationsThisBlock<T>>;
	#[pallet::storage] /// --- ITEM( global_max_registrations_per_block ) 
	pub type MaxRegistrationsPerBlock<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T> >;

	/// ==============================
	/// ==== Subnetworks Storage =====
	/// ==============================
	#[pallet::type_value] 
	pub fn DefaultN<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultModality<T:Config>() -> u16 { 0 }
	#[pallet::type_value] 
	pub fn DefaultHotkeys<T:Config>() -> Vec<u16> { vec![ ] }
	#[pallet::type_value]
	pub fn DefaultNeworksAdded<T: Config>() ->  bool { false }
	#[pallet::type_value]
	pub fn DefaultIsNetworkMember<T: Config>() ->  bool { false }


	#[pallet::storage] /// --- ITEM( tota_number_of_existing_networks )
	pub type TotalNetworks<T> = StorageValue<_, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> subnetwork_n (Number of UIDs in the network).
	pub type SubnetworkN<T:Config> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultN<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> modality   TEXT: 0, IMAGE: 1, TENSOR: 2
	pub type NetworkModality<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultModality<T>> ;
	#[pallet::storage] /// --- MAP ( netuid ) --> network_is_added
	pub type NetworksAdded<T:Config> = StorageMap<_, Identity, u16, bool, ValueQuery, DefaultNeworksAdded<T>>;	
	#[pallet::storage] /// --- DMAP ( netuid, netuid ) -> registration_requirement
	pub type NetworkConnect<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, u16, OptionQuery>;
	#[pallet::storage] /// --- DMAP ( hotkey, netuid ) --> bool
	pub type IsNetworkMember<T:Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Identity, u16, bool, ValueQuery, DefaultIsNetworkMember<T>>;

	/// ==============================
	/// ==== Subnetwork Features =====
	/// ==============================
	#[pallet::type_value]
	pub fn DefaultEmissionValues<T: Config>() ->  u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultPendingEmission<T: Config>() ->  u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultBlocksSinceLastStep<T: Config>() -> u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultLastMechansimStepBlock<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultTempo<T: Config>() -> u16 { T::InitialTempo::get() }

	#[pallet::storage] /// --- MAP ( netuid ) --> tempo
	pub type Tempo<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTempo<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> emission_values
	pub type EmissionValues<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultEmissionValues<T>>;
	#[pallet::storage] /// --- MAP ( netuid ) --> pending_emission
	pub type PendingEmission<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultPendingEmission<T>>;
	#[pallet::storage] /// --- MAP ( netuid ) --> blocks_since_last_step.
	pub type BlocksSinceLastStep<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBlocksSinceLastStep<T>>;
	#[pallet::storage] /// --- MAP ( netuid ) --> last_mechanism_step_block
	pub type LastMechansimStepBlock<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultLastMechansimStepBlock<T> >;

	/// =================================
	/// ==== Axon / Promo Endpoints =====
	/// =================================
	
	// --- Struct for Axon.
	pub type AxonInfoOf = AxonInfo;
	
	#[serde_as]
	#[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct AxonInfo {
		pub block: u64, // --- Axon serving block.
        pub version: u32, // --- Axon version
		#[serde_as(as = "DisplayFromStr")] // serialize as string, deserialize from string
        pub ip: u128, // --- Axon u128 encoded ip address of type v6 or v4.
        pub port: u16, // --- Axon u16 encoded port.
        pub ip_type: u8, // --- Axon ip type, 4 for ipv4 and 6 for ipv6.
		pub protocol: u8, // --- Axon protocol. TCP, UDP, other.
		pub placeholder1: u8, // --- Axon proto placeholder 1.
		pub placeholder2: u8, // --- Axon proto placeholder 1.
	}

	// --- Struct for Prometheus.
	pub type PrometheusInfoOf = PrometheusInfo;
	#[serde_as]
	#[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
	pub struct PrometheusInfo {
		pub block: u64, // --- Prometheus serving block.
        pub version: u32, // --- Prometheus version.
		#[serde_as(as = "DisplayFromStr")] // serialize as string, deserialize from string
        pub ip: u128, // --- Prometheus u128 encoded ip address of type v6 or v4.
        pub port: u16, // --- Prometheus u16 encoded port.
        pub ip_type: u8, // --- Prometheus ip type, 4 for ipv4 and 6 for ipv6.
	}


	#[pallet::type_value] 
	pub fn DefaultServingRateLimit<T: Config>() -> u64 { T::InitialServingRateLimit::get() }

	#[pallet::storage] /// --- ITEM ( serving_rate_limit )
	pub(super) type ServingRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultServingRateLimit<T>>;
	#[pallet::storage] /// --- MAP ( hotkey ) --> axon_info
	pub(super) type Axons<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, AxonInfoOf, OptionQuery>;
	#[pallet::storage] /// --- MAP ( hotkey ) --> prometheus_info
	pub(super) type Prometheus<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, PrometheusInfoOf, OptionQuery>;

	/// =======================================
	/// ==== Subnetwork Hyperparam storage ====
	/// =======================================	
	#[pallet::type_value] 
	pub fn DefaultWeightsSetRateLimit<T: Config>() -> u64 { 0 }
	#[pallet::type_value] 
	pub fn DefaultBlockAtRegistration<T: Config>() -> u64 { 0 }
	#[pallet::type_value]
	pub fn DefaultWeightCuts<T: Config>() -> u16 { T::InitialWeightCuts::get() }
	#[pallet::type_value]
	pub fn DefaultRho<T: Config>() -> u16 { T::InitialRho::get() }
	#[pallet::type_value]
	pub fn DefaultKappa<T: Config>() -> u16 { T::InitialKappa::get() }
	#[pallet::type_value] 
	pub fn DefaultMaxAllowedUids<T: Config>() -> u16 { T::InitialMaxAllowedUids::get() }
	#[pallet::type_value] 
	pub fn DefaultImmunityPeriod<T: Config>() -> u16 { T::InitialImmunityPeriod::get() }
	#[pallet::type_value] 
	pub fn DefaultActivityCutoff<T: Config>() -> u16 { T::InitialActivityCutoff::get() }
	#[pallet::type_value] 
	pub fn DefaultMaxWeightsLimit<T: Config>() -> u16 { T::InitialMaxWeightsLimit::get() }
	#[pallet::type_value] 
	pub fn DefaultWeightsVersionKey<T: Config>() -> u64 { T::InitialWeightsVersionKey::get() }
	#[pallet::type_value] 
	pub fn DefaultMinAllowedWeights<T: Config>() -> u16 { T::InitialMinAllowedWeights::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorEpochLen<T: Config>() -> u16 { T::InitialValidatorEpochLen::get() }
	#[pallet::type_value] 
	pub fn DefaultMaxAllowedValidators<T: Config>() -> u16 { T::InitialMaxAllowedValidators::get() }
	#[pallet::type_value]
	pub fn DefaultAdjustmentInterval<T: Config>() -> u16 { T::InitialAdjustmentInterval::get() }
	#[pallet::type_value]
	pub fn DefaultBondsMovingAverage<T: Config>() -> u64 { T::InitialBondsMovingAverage::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorPruneLen<T: Config>() -> u64 { T::InitialValidatorPruneLen::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorBatchSize<T: Config>() -> u16 { T::InitialValidatorBatchSize::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorSequenceLen<T: Config>() -> u16 { T::InitialValidatorSequenceLen::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorEpochsPerReset<T: Config>() -> u16 { T::InitialValidatorEpochsPerReset::get() }
	#[pallet::type_value]
	pub fn DefaultValidatorExcludeQuantile<T: Config>() -> u16 { T::InitialValidatorExcludeQuantile::get() }
	#[pallet::type_value] 
	pub fn DefaultValidatorLogitsDivergence<T: Config>() -> u64 { T::InitialValidatorLogitsDivergence::get() }
	#[pallet::type_value]
	pub fn DefaultScalingLawPower<T: Config>() -> u16 { T::InitialScalingLawPower::get() }
	#[pallet::type_value]
	pub fn DefaultSynergyScalingLawPower<T: Config>() -> u16 { T::InitialSynergyScalingLawPower::get() }
	#[pallet::type_value] 
	pub fn DefaultTargetRegistrationsPerInterval<T: Config>() -> u16 { T::InitialTargetRegistrationsPerInterval::get() }


	#[pallet::storage] /// --- MAP ( netuid ) --> WeightCuts
	pub type WeightCuts<T> =  StorageMap<_, Identity, u16, u16, ValueQuery, DefaultWeightCuts<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Rho
	pub type Rho<T> =  StorageMap<_, Identity, u16, u16, ValueQuery, DefaultRho<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> Kappa
	pub type Kappa<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultKappa<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> uid, we use to record uids to prune at next epoch.
    pub type NeuronsToPruneAtNextEpoch<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> registrations_this_interval
	pub type RegistrationsThisInterval<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> pow_registrations_this_interval
	pub type POWRegistrationsThisInterval<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> burn_registrations_this_interval
	pub type BurnRegistrationsThisInterval<T:Config> = StorageMap<_, Identity, u16, u16, ValueQuery>;
	#[pallet::storage] /// --- MAP ( netuid ) --> max_allowed_uids
	pub type MaxAllowedUids<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedUids<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> immunity_period
	pub type ImmunityPeriod<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultImmunityPeriod<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> activity_cutoff
	pub type ActivityCutoff<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultActivityCutoff<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> max_weight_limit
	pub type MaxWeightsLimit<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMaxWeightsLimit<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> weights_version_key
	pub type WeightsVersionKey<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightsVersionKey<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_epoch_len
	pub type ValidatorEpochLen<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorEpochLen<T> >; 
	#[pallet::storage] /// --- MAP ( netuid ) --> min_allowed_weights
	pub type MinAllowedWeights<T> = StorageMap< _, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> max_allowed_validators
	pub type MaxAllowedValidators<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedValidators<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> adjustment_interval
	pub type AdjustmentInterval<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultAdjustmentInterval<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> bonds_moving_average
	pub type BondsMovingAverage<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBondsMovingAverage<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_batch_size
	pub type ValidatorBatchSize<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorBatchSize<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> weights_set_rate_limit
	pub type WeightsSetRateLimit<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultWeightsSetRateLimit<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_prune_len
	pub type ValidatorPruneLen<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultValidatorPruneLen<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_sequence_length
	pub type ValidatorSequenceLength<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorSequenceLen<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_epochs_per_reset
	pub type ValidatorEpochsPerReset<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorEpochsPerReset<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_exclude_quantile
	pub type ValidatorExcludeQuantile<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultValidatorExcludeQuantile<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> validator_logits_divergence
	pub type ValidatorLogitsDivergence<T> = StorageMap<_, Identity, u16, u64, ValueQuery, DefaultValidatorLogitsDivergence<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> scaling_law_power
	pub type ScalingLawPower<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultScalingLawPower<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> synergy_scaling_law_power
	pub type SynergyScalingLawPower<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultSynergyScalingLawPower<T> >;
	#[pallet::storage] /// --- MAP ( netuid ) --> target_registrations_this_interval
	pub type TargetRegistrationsPerInterval<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTargetRegistrationsPerInterval<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> block_at_registration
	pub type BlockAtRegistration<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, u64, ValueQuery, DefaultBlockAtRegistration<T> >;

	/// =======================================
	/// ==== Subnetwork Consensus Storage  ====
	/// =======================================
	#[pallet::type_value] 
	pub fn EmptyU16Vec<T:Config>() -> Vec<u16> { vec![] }
	#[pallet::type_value] 
	pub fn EmptyU64Vec<T:Config>() -> Vec<u64> { vec![] }
	#[pallet::type_value] 
	pub fn EmptyBoolVec<T:Config>() -> Vec<bool> { vec![] }
	#[pallet::type_value] 
	pub fn DefaultBonds<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::type_value] 
	pub fn DefaultWeights<T:Config>() -> Vec<(u16, u16)> { vec![] }
	#[pallet::type_value] 
	pub fn DefaultKey<T:Config>() -> T::AccountId { T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap() }

	#[pallet::storage] /// --- DMAP ( netuid, hotkey ) --> uid
	pub(super) type Uids<T:Config> = StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, u16, OptionQuery>;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> hotkey
	pub(super) type Keys<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T> >;
	#[pallet::storage] /// --- DMAP ( netuid ) --> emission
	pub(super) type LoadedEmission<T:Config> = StorageMap< _, Identity, u16, Vec<(T::AccountId, u64)>, OptionQuery >;

	#[pallet::storage] /// --- DMAP ( netuid ) --> active
	pub(super) type Active<T:Config> = StorageMap< _, Identity, u16, Vec<bool>, ValueQuery, EmptyBoolVec<T> >;
	#[pallet::storage] /// --- DMAP ( netuid ) --> rank
	pub(super) type Rank<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> trust
	pub(super) type Trust<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> consensus
	pub(super) type Consensus<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> incentive
	pub(super) type Incentive<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> dividends
	pub(super) type Dividends<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> dividends
	pub(super) type Emission<T:Config> = StorageMap< _, Identity, u16, Vec<u64>, ValueQuery, EmptyU64Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> last_update
	pub(super) type LastUpdate<T:Config> = StorageMap< _, Identity, u16, Vec<u64>, ValueQuery, EmptyU64Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> validator_trust
	pub(super) type ValidatorTrust<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> weight_consensus
	pub(super) type WeightConsensus<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T>>;
	#[pallet::storage] /// --- DMAP ( netuid ) --> pruning_scores
	pub(super) type PruningScores<T:Config> = StorageMap< _, Identity, u16, Vec<u16>, ValueQuery, EmptyU16Vec<T> >;
	#[pallet::storage] /// --- DMAP ( netuid ) --> validator_permit
    pub(super) type ValidatorPermit<T:Config> = StorageMap<_, Identity, u16, Vec<bool>, ValueQuery, EmptyBoolVec<T> >;

	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> weights
    pub(super) type Weights<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultWeights<T> >;
	#[pallet::storage] /// --- DMAP ( netuid, uid ) --> bonds
    pub(super) type Bonds<T:Config> = StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery, DefaultBonds<T> >;


	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NetworkAdded( u16, u16 ),	// --- Event created when a new network is added.
		NetworkRemoved( u16 ), // --- Event created when a network is removed.
		StakeAdded( T::AccountId, u64 ), // --- Event created when stake has been transfered from the a coldkey account onto the hotkey staking account.
		StakeRemoved( T::AccountId, u64 ), // --- Event created when stake has been removed from the hotkey staking account onto the coldkey account.
		WeightsSet( u16, u16 ), // ---- Event created when a caller successfully set's their weights on a subnetwork.
		NeuronRegistered( u16, u16, T::AccountId ), // --- Event created when a new neuron account has been registered to the chain.
		BulkNeuronsRegistered( u16, u16 ), // --- Event created when multiple uids have been concurrently registered.
		BulkBalancesSet(u16, u16),
		MaxAllowedUidsSet( u16, u16 ), // --- Event created when max allowed uids has been set for a subnetwor.
		MaxWeightLimitSet( u16, u16 ), // --- Event created when the max weight limit has been set.
		DifficultySet( u16, u64 ), // --- Event created when the difficulty has been set for a subnet.
		AdjustmentIntervalSet( u16, u16 ), // --- Event created when the adjustment interval is set for a subnet.
		RegistrationPerIntervalSet( u16, u16 ), // --- Event created when registeration per interval is set for a subnet.
		MaxRegistrationsPerBlockSet( u16, u16), // --- Event created when we set max registrations per block
		ActivityCutoffSet( u16, u16 ), // --- Event created when an activity cutoff is set for a subnet.
		WeightCutsSet( u16, u16 ), // --- Event created when WeightCuts value is set.
		RhoSet( u16, u16 ), // --- Event created when Rho value is set.
		KappaSet( u16, u16 ), // --- Event created when kappa is set for a subnet.
		MinAllowedWeightSet( u16, u16 ), // --- Event created when minimun allowed weight is set for a subnet.
		ValidatorBatchSizeSet( u16, u16 ), // --- Event created when validator batch size is set for a subnet.
		ValidatorSequenceLengthSet( u16, u16 ), // --- Event created when validator sequence length i set for a subnet.
		ValidatorEpochPerResetSet( u16, u16 ), // --- Event created when validator epoch per reset is set for a subnet.
		ValidatorExcludeQuantileSet( u16, u16 ), // --- Event created when the validator exclude quantile has been set for a subnet.
		ValidatorEpochLengthSet( u16, u16 ), // --- Event created when the validator epoch length has been set for a subnet.
		ValidatorLogitsDivergenceSet( u16, u64 ), /// --- Event created when the validator logits divergence value has been set.
		ValidatorPruneLenSet( u16, u64 ), /// --- Event created when the validator pruning length has been set.
		ScalingLawPowerSet( u16, u16 ), // --- Event created when the scaling law power has been set for a subnet.
		SynergyScalingLawPowerSet( u16, u16 ), // --- Event created when the synergy scaling law has been set for a subnet.
		WeightsSetRateLimitSet( u16, u64 ), // --- Event create when weights set rate limit has been set for a subnet.
		ImmunityPeriodSet( u16, u16), // --- Event created when immunity period is set for a subnet.
		BondsMovingAverageSet( u16, u64), // --- Event created when bonds moving average is set for a subnet.
		MaxAllowedValidatorsSet( u16, u16), // --- Event created when setting the max number of allowed validators on a subnet.
		AxonServed( T::AccountId ), // --- Event created when the axon server information is added to the network.
		PrometheusServed( T::AccountId ), // --- Event created when the axon server information is added to the network.
		EmissionValuesSet(), // --- Event created when emission ratios fr all networks is set.
		NetworkConnectionAdded( u16, u16, u16 ), // --- Event created when a network connection requirement is added.
		NetworkConnectionRemoved( u16, u16 ), // --- Event created when a network connection requirement is removed.
		DelegateAdded( T::AccountId, T::AccountId, u16 ), // --- Event created to signal a hotkey has become a delegate.
		DefaultTakeSet( u16 ), // --- Event created when the default take is set.
		WeightsVersionKeySet( u16, u64 ), // --- Event created when weights version key is set for a network.
		MinDifficultySet( u16, u64 ), // --- Event created when setting min difficutly on a network.
		MaxDifficultySet( u16, u64 ), // --- Event created when setting max difficutly on a network.
		ServingRateLimitSet( u64 ), // --- Event created when setting the prometheus serving rate limit.
		BurnSet( u16, u64 ), // --- Event created when setting burn on a network.
		MaxBurnSet( u16, u64 ), // --- Event created when setting max burn on a network.
		MinBurnSet( u16, u64 ), // --- Event created when setting min burn on a network.
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		InvalidConnectionRequirement, // --- Thrown if we are attempting to create an invalid connection requirement.
		NetworkDoesNotExist, // --- Thrown when the network does not exist.
		NetworkExist, // --- Thrown when the network already exist.
		InvalidModality, // --- Thrown when an invalid modality attempted on serve.
		InvalidIpType, // ---- Thrown when the user tries to serve an axon which is not of type	4 (IPv4) or 6 (IPv6).
		InvalidIpAddress, // --- Thrown when an invalid IP address is passed to the serve function.
		NotRegistered, // ---- Thrown when the caller requests setting or removing data from a neuron which does not exist in the active set.
		NonAssociatedColdKey, // ---- Thrown when a stake, unstake or subscribe request is made by a coldkey which is not associated with the hotkey account. 
		NotEnoughStaketoWithdraw, // ---- Thrown when the caller requests removing more stake then there exists in the staking account. See: fn remove_stake.
		NotEnoughBalanceToStake, //  ---- Thrown when the caller requests adding more stake than there exists in the cold key account. See: fn add_stake
		BalanceWithdrawalError, // ---- Thrown when the caller tries to add stake, but for some reason the requested amount could not be withdrawn from the coldkey account
		NoValidatorPermit, // ---- Thrown when the caller attempts to set non-self weights without being a permitted validator.
		WeightVecNotEqualSize, // ---- Thrown when the caller attempts to set the weight keys and values but these vectors have different size.
		DuplicateUids, // ---- Thrown when the caller attempts to set weights with duplicate uids in the weight matrix.
		InvalidUid, // ---- Thrown when a caller attempts to set weight to at least one uid that does not exist in the metagraph.
		NotSettingEnoughWeights, // ---- Thrown when the dispatch attempts to set weights on chain with fewer elements than are allowed.
		TooManyRegistrationsThisBlock, // ---- Thrown when registrations this block exceeds allowed number.
		AlreadyRegistered, // ---- Thrown when the caller requests registering a neuron which already exists in the active set.
		InvalidWorkBlock, // ---- Thrown if the supplied pow hash block is in the future or negative
		WorkRepeated, // ---- Thrown when the caller attempts to use a repeated work.
		InvalidDifficulty, // ---- Thrown if the supplied pow hash block does not meet the network difficulty.
		InvalidSeal, // ---- Thrown if the supplied pow hash seal does not match the supplied work.
		MaxAllowedUIdsNotAllowed, // ---  Thrown if the vaule is invalid for MaxAllowedUids
		CouldNotConvertToBalance, // ---- Thrown when the dispatch attempts to convert between a u64 and T::balance but the call fails.
		StakeAlreadyAdded, // --- Thrown when the caller requests adding stake for a hotkey to the total stake which already added
		MaxWeightExceeded, // --- Thrown when the dispatch attempts to set weights on chain with where any normalized weight is more than MaxWeightLimit.
		StorageValueOutOfRange, // --- Thrown when the caller attempts to set a storage value outside of its allowed range.
		TempoHasNotSet, // --- Thrown when tempo has not set
		InvalidTempo, // --- Thrown when tempo is not valid
		EmissionValuesDoesNotMatchNetworks, // --- Thrown when number or recieved emission rates does not match number of networks
		InvalidEmissionValues, // --- Thrown when emission ratios are not valid (did not sum up to 10^9)
		DidNotPassConnectedNetworkRequirement, // --- Thrown when a hotkey attempts to register into a network without passing the  registration requirment from another network.
		AlreadyDelegate, // --- Thrown if the hotkey attempt to become delegate when they are already.
		SettingWeightsTooFast, // --- Thrown if the hotkey attempts to set weights twice withing net_tempo/2 blocks.
		IncorrectNetworkVersionKey, // --- Thrown of a validator attempts to set weights from a validator with incorrect code base key.
		ServingRateLimitExceeded, // --- Thrown when an axon or prometheus serving exceeds the rate limit for a registered neuron.
		BalanceSetError, // --- Thrown when an error occurs setting a balance
		MaxAllowedUidsExceeded, // --- Thrown when number of accounts going to be registered exceed MaxAllowedUids for the network.
		TooManyUids, // ---- Thrown when the caller attempts to set weights with more uids than allowed.
	}

	/// ================
	/// ==== Hooks =====
	/// ================
	#[pallet::hooks] 
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> { 
		/// ---- Called on the initialization of this pallet. (the order of on_finalize calls is determined in the runtime)
		///
		/// # Args:
		/// 	* 'n': (T::BlockNumber):
		/// 		- The number of the block we are initializing.
		fn on_initialize( _block_number: BlockNumberFor<T> ) -> Weight {
			//Self::block_step();
			
			return Weight::from_ref_time(110_634_229_000 as u64)
						.saturating_add(T::DbWeight::get().reads(8304 as u64))
						.saturating_add(T::DbWeight::get().writes(110 as u64));
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}
