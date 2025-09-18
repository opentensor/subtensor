#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// Some arithmetic operations can't use the saturating equivalent, such as the PerThing types
#![allow(clippy::arithmetic_side_effects)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod bag_thresholds;
pub mod check_nonce;
mod migrations;
pub mod transaction_payment_wrapper;

extern crate alloc;

use crate::transaction_payment_wrapper::ChargeTransactionPaymentWrapper;
use codec::{Compact, Decode, Encode};
use core::num::NonZeroU64;
use frame_election_provider_support::bounds::ElectionBoundsBuilder;
use frame_election_provider_support::{SequentialPhragmen, generate_solution_type, onchain};
use frame_support::pallet_prelude::DispatchClass;
use frame_support::{
    PalletId,
    dispatch::{DispatchResult, DispatchResultWithPostInfo},
    genesis_builder_helper::{build_state, get_preset},
    pallet_prelude::Get,
    traits::{Contains, InsideBoth, LinearStoragePrice, fungible::HoldConsideration},
};
use frame_system::{EnsureNever, EnsureRoot, EnsureRootWithSuccess, RawOrigin};
use pallet_commitments::{CanCommit, OnMetadataCommitment};
use pallet_election_provider_multi_phase::GeometricDepositBase;
use pallet_grandpa::{AuthorityId as GrandpaId, fg_primitives};
use pallet_registry::CanRegisterIdentity;
use pallet_session::historical as session_historical;
use pallet_staking::UseValidatorsMap;
use pallet_subtensor::rpc_info::{
    delegate_info::DelegateInfo,
    dynamic_info::DynamicInfo,
    metagraph::{Metagraph, SelectiveMetagraph},
    neuron_info::{NeuronInfo, NeuronInfoLite},
    show_subnet::SubnetState,
    stake_info::StakeInfo,
    subnet_info::{SubnetHyperparams, SubnetHyperparamsV2, SubnetInfo, SubnetInfov2},
};
use pallet_subtensor_collective as pallet_collective;
use pallet_subtensor_proxy as pallet_proxy;
use pallet_subtensor_swap_runtime_api::SimSwapResult;
use pallet_subtensor_utility as pallet_utility;
use runtime_common::prod_or_fast;
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
    H160, H256, OpaqueMetadata, U256,
    crypto::{ByteArray, KeyTypeId},
};
use sp_runtime::Cow;
use sp_runtime::SaturatedConversion;
use sp_runtime::generic::Era;
use sp_runtime::traits::OpaqueKeys;
use sp_runtime::transaction_validity::TransactionPriority;
use sp_runtime::{
    AccountId32, ApplyExtrinsicResult, ConsensusEngineId, Percent, generic, impl_opaque_keys,
    traits::{
        AccountIdLookup, BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable, One,
        PostDispatchInfoOf, UniqueSaturatedInto, Verify,
    },
    transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
};
use sp_staking::SessionIndex;
use sp_staking::currency_to_vote::SaturatingCurrencyToVote;
use sp_std::cmp::Ordering;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use subtensor_precompiles::Precompiles;
use subtensor_runtime_common::{AlphaCurrency, TaoCurrency, time::*, *};
use subtensor_swap_interface::{OrderType, SwapHandler};

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    StorageValue, construct_runtime, parameter_types,
    traits::{
        ConstBool, ConstU8, ConstU32, ConstU64, ConstU128, FindAuthor, InstanceFilter,
        KeyOwnerProofSystem, OnFinalize, OnTimestampSet, PrivilegeCmp, Randomness, StorageInfo,
    },
    weights::{
        IdentityFee, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
    },
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
use pallet_commitments::GetCommitments;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ConstFeeMultiplier, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
use subtensor_transaction_fee::{SubtensorTxFeeHandler, TransactionFeeHandler};

use core::marker::PhantomData;

// Frontier
use fp_rpc::TransactionStatus;
use pallet_ethereum::{Call::transact, PostLogContent, Transaction as EthereumTransaction};
use pallet_evm::{
    Account as EVMAccount, BalanceConverter, EvmBalance, FeeCalculator, Runner, SubstrateBalance,
};

// Drand
impl pallet_drand::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityId = pallet_drand::crypto::TestAuthId;
    type Verifier = pallet_drand::verifier::QuicknetVerifier;
    type UnsignedPriority = ConstU64<{ 1 << 20 }>;
    type HttpFetchTimeout = ConstU64<1_000>;
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::CreateTransactionBase<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type RuntimeCall = RuntimeCall;
}

impl<LocalCall> frame_system::offchain::CreateInherent<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_inherent(call: RuntimeCall) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_bare(call)
    }
}

impl frame_system::offchain::CreateSignedTransaction<pallet_drand::Call<Runtime>> for Runtime {
    fn create_signed_transaction<
        S: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>,
    >(
        call: RuntimeCall,
        public: Self::Public,
        account: Self::AccountId,
        nonce: Self::Nonce,
    ) -> Option<Self::Extrinsic> {
        use sp_runtime::traits::StaticLookup;

        let address = <Runtime as frame_system::Config>::Lookup::unlookup(account.clone());
        let extra: TransactionExtensions = (
            frame_system::CheckNonZeroSender::<Runtime>::new(),
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(Era::Immortal),
            check_nonce::CheckNonce::<Runtime>::from(nonce).into(),
            frame_system::CheckWeight::<Runtime>::new(),
            ChargeTransactionPaymentWrapper::new(
                pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
            ),
            pallet_subtensor::transaction_extension::SubtensorTransactionExtension::<Runtime>::new(
            ),
            pallet_drand::drand_priority::DrandPriority::<Runtime>::new(),
            frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new(true),
        );

        let raw_payload = SignedPayload::new(call.clone(), extra.clone()).ok()?;
        let signature = raw_payload.using_encoded(|payload| S::sign(payload, public))?;

        Some(UncheckedExtrinsic::new_signed(
            call, address, signature, extra,
        ))
    }
}

// Subtensor module
pub use pallet_scheduler;
pub use pallet_subtensor;

// Member type for membership
type MemberCount = u32;

// Method used to calculate the fee of an extrinsic
pub const fn deposit(items: u32, bytes: u32) -> Balance {
    pub const ITEMS_FEE: Balance = 2_000 * 10_000;
    pub const BYTES_FEE: Balance = 100 * 10_000;
    (items as Balance)
        .saturating_mul(ITEMS_FEE)
        .saturating_add((bytes as Balance).saturating_mul(BYTES_FEE))
}

parameter_types! {
    /// A limit for off-chain phragmen unsigned solution submission.
    ///
    /// We want to keep it as high as possible, but can't risk having it reject,
    /// so we always subtract the base block execution weight.
    pub OffchainSolutionWeightLimit: Weight = BlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic
        .expect("Normal extrinsics have weight limit configured by default; qed")
        .saturating_sub(BlockExecutionWeight::get());

    /// A limit for off-chain phragmen unsigned solution length.
    ///
    /// We allow up to 90% of the block's size to be consumed by the solution.
    pub OffchainSolutionLengthLimit: u32 = Perbill::from_rational(90_u32, 100) *
        *BlockLength::get()
        .max
        .get(DispatchClass::Normal);
}

/// RAO per TAO
pub const UNITS: Balance = 1_000_000_000;

/// Babe epoch duration.
pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = prod_or_fast!(4 * HOURS, MINUTES / 6);

// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
// the specifics of the runtime. They can then be made to be agnostic over specific formats
// of data like extrinsics, allowing for them to continue syncing the network through upgrades
// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    // Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    // Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    // Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub babe: Babe,
            pub grandpa: Grandpa,
        }
    }
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("node-subtensor"),
    impl_name: Cow::Borrowed("node-subtensor"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 317,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

pub const MAXIMUM_BLOCK_WEIGHT: Weight =
    Weight::from_parts(4u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX);

// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    // We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            MAXIMUM_BLOCK_WEIGHT,
            NORMAL_DISPATCH_RATIO,
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(10 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

pub struct NoNestingCallFilter;

impl Contains<RuntimeCall> for NoNestingCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        match call {
            RuntimeCall::Utility(inner) => {
                let calls = match inner {
                    pallet_utility::Call::force_batch { calls } => calls,
                    pallet_utility::Call::batch { calls } => calls,
                    pallet_utility::Call::batch_all { calls } => calls,
                    _ => &Vec::new(),
                };

                !calls.iter().any(|call| {
					matches!(call, RuntimeCall::Utility(inner) if matches!(inner, pallet_utility::Call::force_batch { .. } | pallet_utility::Call::batch_all { .. } | pallet_utility::Call::batch { .. }))
				})
            }
            _ => true,
        }
    }
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    // The basic call filter to use in dispatchable.
    type BaseCallFilter = InsideBoth<SafeMode, NoNestingCallFilter>;
    // Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    // The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    // The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    // The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    // The aggregated runtime tasks.
    type RuntimeTask = RuntimeTask;
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
    // Converts a module to the index of the module in `construct_runtime!`.
    //
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
    type Nonce = Nonce;
    type Block = Block;
    type SingleBlockMigrations = Migrations;
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
    type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
    type MaxNominators = MaxNominators;
    type EquivocationReportSystem =
        pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
    pub EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS as u64;
    pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
    pub ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;

    // session module is the trigger
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;

    type DisabledValidators = Session;

    type WeightInfo = ();

    type MaxAuthorities = MaxAuthorities;
    type MaxNominators = MaxNominators;

    type KeyOwnerProof =
        <Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;

    type EquivocationReportSystem =
        pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type EventHandler = Staking;
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = opaque::SessionKeys;
    type DisablingStrategy = pallet_session::disabling::UpToLimitWithReEnablingDisablingStrategy;
    type WeightInfo = ();
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

parameter_types! {
    // phase durations. 1/4 of the last session for each.
    // in testing: 1min or half of the session for each
    pub SignedPhase: u32 = prod_or_fast!(
        EPOCH_DURATION_IN_SLOTS / 4,
        MINUTES.min(EpochDuration::get().saturated_into::<u32>() / 2),
        "TAO_SIGNED_PHASE"
    );
    pub UnsignedPhase: u32 = prod_or_fast!(
        EPOCH_DURATION_IN_SLOTS / 4,
        MINUTES.min(EpochDuration::get().saturated_into::<u32>() / 2),
        "TAO_UNSIGNED_PHASE"
    );

    // signed config
    pub const SignedMaxSubmissions: u32 = 16;
    pub const SignedMaxRefunds: u32 = 16 / 4;
    pub const SignedFixedDeposit: Balance = deposit(2, 0);
    pub const SignedDepositIncreaseFactor: Percent = Percent::from_percent(10);
    // 0.01 TAO per KB of solution data.
    pub const SignedDepositByte: Balance = deposit(0, 10) / 1024;
    // Each good submission will get 1 TAO as reward
    pub SignedRewardBase: Balance = UNITS;

    // 4 hour session, 1 hour unsigned phase, 32 offchain executions.
    pub OffchainRepeat: BlockNumber = UnsignedPhase::get() / 32;

    pub const MaxElectingVoters: u32 = 22_500;
    /// We take the top 22500 nominators as electing voters and all of the validators as electable
    /// targets. Whilst this is the case, we cannot and shall not increase the size of the
    /// validator intentions.
    pub ElectionBounds: frame_election_provider_support::bounds::ElectionBounds =
        ElectionBoundsBuilder::default().voters_count(MaxElectingVoters::get().into()).build();
    /// Setup election pallet to support maximum winners upto 1200. This will mean Staking Pallet
    /// cannot have active validators higher than this count.
    pub const MaxActiveValidators: u32 = 1200;
}

generate_solution_type!(
    #[compact]
    pub struct NposCompactSolution16::<
        VoterIndex = u32,
        TargetIndex = u16,
        Accuracy = sp_runtime::PerU16,
        MaxVoters = MaxElectingVoters,
    >(16)
);

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
    type System = Runtime;
    type Solver = SequentialPhragmen<AccountId, runtime_common::elections::OnChainAccuracy>;
    type DataProvider = Staking;
    type WeightInfo = ();
    type MaxWinners = MaxActiveValidators;
    type Bounds = ElectionBounds;
}

impl pallet_election_provider_multi_phase::MinerConfig for Runtime {
    type AccountId = AccountId;
    type MaxLength = OffchainSolutionLengthLimit;
    type MaxWeight = OffchainSolutionWeightLimit;
    type Solution = NposCompactSolution16;
    type MaxVotesPerVoter = <
		<Self as pallet_election_provider_multi_phase::Config>::DataProvider
		as
		frame_election_provider_support::ElectionDataProvider
	>::MaxVotesPerVoter;
    type MaxWinners = MaxActiveValidators;

    // The unsigned submissions have to respect the weight of the submit_unsigned call, thus their
    // weight estimate function is wired to this call's weight.
    fn solution_weight(v: u32, t: u32, a: u32, d: u32) -> Weight {
        <
			<Self as pallet_election_provider_multi_phase::Config>::WeightInfo
			as
			pallet_election_provider_multi_phase::WeightInfo
		>::submit_unsigned(v, t, a, d)
    }
}

impl pallet_election_provider_multi_phase::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EstimateCallFee = TransactionPayment;
    type SignedPhase = SignedPhase;
    type UnsignedPhase = UnsignedPhase;
    type SignedMaxSubmissions = SignedMaxSubmissions;
    type SignedMaxRefunds = SignedMaxRefunds;
    type SignedRewardBase = SignedRewardBase;
    type SignedDepositBase =
        GeometricDepositBase<Balance, SignedFixedDeposit, SignedDepositIncreaseFactor>;
    type SignedDepositByte = SignedDepositByte;
    type SignedDepositWeight = ();
    type SignedMaxWeight =
        <Self::MinerConfig as pallet_election_provider_multi_phase::MinerConfig>::MaxWeight;
    type MinerConfig = Self;
    type SlashHandler = (); // burn slashes
    type RewardHandler = (); // nothing to do upon rewards
    type BetterSignedThreshold = ();
    type OffchainRepeat = OffchainRepeat;
    type MinerTxPriority = NposSolutionPriority;
    type DataProvider = Staking;
    #[cfg(any(feature = "fast-runtime", feature = "runtime-benchmarks"))]
    type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
    #[cfg(not(any(feature = "fast-runtime", feature = "runtime-benchmarks")))]
    type Fallback = frame_election_provider_support::NoElection<(
        AccountId,
        BlockNumber,
        Staking,
        MaxActiveValidators,
    )>;
    type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type Solver = SequentialPhragmen<
        AccountId,
        pallet_election_provider_multi_phase::SolutionAccuracyOf<Self>,
        (),
    >;
    type BenchmarkingConfig = runtime_common::elections::BenchmarkConfig;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = ();
    type MaxWinners = MaxActiveValidators;
    type ElectionBounds = ElectionBounds;
}

parameter_types! {
    pub const BagThresholds: &'static [u64] = &bag_thresholds::THRESHOLDS;
}

pub type VoterBagsListInstance = pallet_bags_list::Instance1;
impl pallet_bags_list::Config<VoterBagsListInstance> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ScoreProvider = Staking;
    type WeightInfo = ();
    type BagThresholds = BagThresholds;
    type Score = sp_npos_elections::VoteWeight;
}

parameter_types! {
    // Six sessions in an era (24 hours).
    pub const SessionsPerEra: SessionIndex = prod_or_fast!(6, 2);

    // 28 eras for unbonding (28 days).
    pub BondingDuration: sp_staking::EraIndex = prod_or_fast!(
        28,
        2,
        "BONDING_DURATION"
    );
    pub SlashDeferDuration: sp_staking::EraIndex = prod_or_fast!(
        27,
        1,
        "SLASH_DEFER_DURATION"
    );
    pub const MaxExposurePageSize: u32 = 512;
    // Note: this is not really correct as Max Nominators is (MaxExposurePageSize * page_count) but
    // this is an unbounded number. We just set it to a reasonably high value, 1 full page
    // of nominators.
    pub const MaxNominators: u32 = 512;
    // pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    // 16
    pub const MaxNominations: u32 = <NposCompactSolution16 as frame_election_provider_support::NposSolution>::LIMIT as u32;
}

pub struct EraPayout;
impl pallet_staking::EraPayout<Balance> for EraPayout {
    fn era_payout(
        _total_staked: Balance,
        _total_issuance: Balance,
        _era_duration_millis: u64,
    ) -> (Balance, Balance) {
        let era_emissions = pallet_subtensor::PendingNodeValidatorEmissions::<Runtime>::take();
        (era_emissions.into(), 0u64)
    }
}

/// Restrict accounts allowed to participate in staking to those in the `Invulnerables` storage.
/// This creates a type of "permissioned" NPoS system, where Root controls who is allowed to
/// participate by controlling `Invulnerables`.
///
/// This is a temporary filter slated to be removed in Phase 5 of the NPoS roadmap.
/// See <https://github.com/opentensor/subtensor/issues/1887> for more info.
pub struct StakingBlacklistFilter;
impl Contains<AccountId32> for StakingBlacklistFilter {
    fn contains(account_id: &AccountId32) -> bool {
        let invulnerables = pallet_staking::Invulnerables::<Runtime>::get();
        for a in invulnerables.iter() {
            if account_id == a {
                // Do NOT filter accounts in `Invulnerables` storage.
                return false;
            }
        }
        // Filter out everyone else.
        true
    }
}

impl pallet_staking::Config for Runtime {
    type OldCurrency = Balances;
    type Currency = Balances;
    type CurrencyBalance = Balance;
    type UnixTime = Timestamp;
    type CurrencyToVote = SaturatingCurrencyToVote;
    type RewardRemainder = ();
    // type RewardRemainder = Treasury;
    type RuntimeEvent = RuntimeEvent;
    type Slash = ();
    // type Slash = Treasury;
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type SessionInterface = Self;
    type EraPayout = EraPayout;
    type MaxExposurePageSize = MaxExposurePageSize;
    type NextNewSession = Session;
    type ElectionProvider = ElectionProviderMultiPhase;
    type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
    type VoterList = VoterList;
    type TargetList = UseValidatorsMap<Self>;
    type NominationsQuota = pallet_staking::FixedNominationsQuota<{ MaxNominations::get() }>;
    type MaxUnlockingChunks = frame_support::traits::ConstU32<32>;
    type HistoryDepth = frame_support::traits::ConstU32<84>;
    type MaxControllersInDeprecationBatch = ConstU32<5314>;
    type BenchmarkingConfig = runtime_common::StakingBenchmarkingConfig;
    type RuntimeHoldReason = RuntimeHoldReason;
    // type EventListeners = NominationPools;
    type EventListeners = ();
    type WeightInfo = ();
    type Filter = StakingBlacklistFilter;
}

impl pallet_fast_unstake::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BatchSize = frame_support::traits::ConstU32<16>;
    type Deposit = frame_support::traits::ConstU64<{ UNITS }>;
    type ControlOrigin = EnsureRoot<AccountId>;
    type Staking = Staking;
    type MaxErasToCheckPerBlock = ConstU32<1>;
    type WeightInfo = ();
}

impl pallet_offences::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

parameter_types! {
    pub const MaxAuthorities: u32 = 2_000;
    pub NposSolutionPriority: TransactionPriority =
        Perbill::from_percent(90) * TransactionPriority::MAX;
}

parameter_types! {
    pub const PoolsPalletId: PalletId = PalletId(*b"py/nopls");
    // Allow pools that got slashed up to 90% to remain operational.
    pub const MaxPointsToBalance: u8 = 10;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

parameter_types! {
    pub MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

/// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
/// The choice of is done in accordance to the slot duration and expected target
/// block time, for safely resisting network delays of maximum two seconds.
/// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
///
/// Staging this Babe constant prior to enacting the full Babe upgrade so the node
/// can build itself a `BabeConfiguration` prior to the upgrade taking place.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The BABE epoch configuration at genesis.
///
/// Staging this Babe constant prior to enacting the full Babe upgrade so the node
/// can build itself a `BabeConfiguration` prior to the upgrade taking place.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
    };

impl pallet_timestamp::Config for Runtime {
    // A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const DisallowPermissionlessEnterDuration: BlockNumber = 0;
    pub const DisallowPermissionlessExtendDuration: BlockNumber = 0;

    pub const RootEnterDuration: BlockNumber = 5 * 60 * 24; // 24 hours

    pub const RootExtendDuration: BlockNumber = 5 * 60 * 12; // 12 hours

    pub const DisallowPermissionlessEntering: Option<Balance> = None;
    pub const DisallowPermissionlessExtending: Option<Balance> = None;
    pub const DisallowPermissionlessRelease: Option<BlockNumber> = None;
}

pub struct SafeModeWhitelistedCalls;
impl Contains<RuntimeCall> for SafeModeWhitelistedCalls {
    fn contains(call: &RuntimeCall) -> bool {
        matches!(
            call,
            RuntimeCall::Sudo(_)
                | RuntimeCall::Multisig(_)
                | RuntimeCall::System(_)
                | RuntimeCall::SafeMode(_)
                | RuntimeCall::Timestamp(_)
                | RuntimeCall::SubtensorModule(
                    pallet_subtensor::Call::set_weights { .. }
                        | pallet_subtensor::Call::serve_axon { .. }
                )
                | RuntimeCall::Commitments(pallet_commitments::Call::set_commitment { .. })
        )
    }
}

impl pallet_safe_mode::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type WhitelistedCalls = SafeModeWhitelistedCalls;
    type EnterDuration = DisallowPermissionlessEnterDuration;
    type ExtendDuration = DisallowPermissionlessExtendDuration;
    type EnterDepositAmount = DisallowPermissionlessEntering;
    type ExtendDepositAmount = DisallowPermissionlessExtending;
    type ForceEnterOrigin = EnsureRootWithSuccess<AccountId, RootEnterDuration>;
    type ForceExtendOrigin = EnsureRootWithSuccess<AccountId, RootExtendDuration>;
    type ForceExitOrigin = EnsureRoot<AccountId>;
    type ForceDepositOrigin = EnsureRoot<AccountId>;
    type Notify = ();
    type ReleaseDelay = DisallowPermissionlessRelease;
    type WeightInfo = pallet_safe_mode::weights::SubstrateWeight<Runtime>;
}

// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u64 = 500;

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    // The type for recording an account's balance.
    type Balance = Balance;
    // The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;

    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<50>;
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const OperationalFeeMultiplier: u8 = 5;
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = SubtensorTxFeeHandler<Balances, TransactionFeeHandler<Runtime>>;
    // Convert dispatch weight to a chargeable fee.
    type WeightToFee = subtensor_transaction_fee::LinearWeightToFee;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

// Configure collective pallet for council
parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 12 * HOURS;
    pub const CouncilMaxProposals: u32 = 10;
    pub const CouncilMaxMembers: u32 = 3;
}

// Configure collective pallet for Senate
parameter_types! {
    pub const SenateMaxMembers: u32 = 12;
}

use pallet_collective::{CanPropose, CanVote, GetVotingMembers};
pub struct CanProposeToTriumvirate;
impl CanPropose<AccountId> for CanProposeToTriumvirate {
    fn can_propose(account: &AccountId) -> bool {
        Triumvirate::is_member(account)
    }
}

pub struct CanVoteToTriumvirate;
impl CanVote<AccountId> for CanVoteToTriumvirate {
    fn can_vote(_: &AccountId) -> bool {
        //Senate::is_member(account)
        false // Disable voting from pallet_collective::vote
    }
}

use pallet_subtensor::{
    CollectiveInterface, CommitmentsInterface, MemberManagement, ProxyInterface,
};
pub struct ManageSenateMembers;
impl MemberManagement<AccountId> for ManageSenateMembers {
    fn add_member(account: &AccountId) -> DispatchResultWithPostInfo {
        let who = Address::Id(account.clone());
        SenateMembers::add_member(RawOrigin::Root.into(), who)
    }

    fn remove_member(account: &AccountId) -> DispatchResultWithPostInfo {
        let who = Address::Id(account.clone());
        SenateMembers::remove_member(RawOrigin::Root.into(), who)
    }

    fn swap_member(rm: &AccountId, add: &AccountId) -> DispatchResultWithPostInfo {
        let remove = Address::Id(rm.clone());
        let add = Address::Id(add.clone());

        Triumvirate::remove_votes(rm)?;
        SenateMembers::swap_member(RawOrigin::Root.into(), remove, add)
    }

    fn is_member(account: &AccountId) -> bool {
        SenateMembers::members().contains(account)
    }

    fn members() -> Vec<AccountId> {
        SenateMembers::members().into()
    }

    fn max_members() -> u32 {
        SenateMaxMembers::get()
    }
}

pub struct GetSenateMemberCount;
impl GetVotingMembers<MemberCount> for GetSenateMemberCount {
    fn get_count() -> MemberCount {
        SenateMembers::members().len() as u32
    }
}
impl Get<MemberCount> for GetSenateMemberCount {
    fn get() -> MemberCount {
        SenateMaxMembers::get()
    }
}

pub struct TriumvirateVotes;
impl CollectiveInterface<AccountId, Hash, u32> for TriumvirateVotes {
    fn remove_votes(hotkey: &AccountId) -> Result<bool, sp_runtime::DispatchError> {
        Triumvirate::remove_votes(hotkey)
    }

    fn add_vote(
        hotkey: &AccountId,
        proposal: Hash,
        index: u32,
        approve: bool,
    ) -> Result<bool, sp_runtime::DispatchError> {
        Triumvirate::do_vote(hotkey.clone(), proposal, index, approve)
    }
}

type EnsureMajoritySenate =
    pallet_collective::EnsureProportionMoreThan<AccountId, TriumvirateCollective, 1, 2>;

// We call pallet_collective TriumvirateCollective
type TriumvirateCollective = pallet_collective::Instance1;
impl pallet_collective::Config<TriumvirateCollective> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = GetSenateMemberCount;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type SetMembersOrigin = EnsureNever<AccountId>;
    type CanPropose = CanProposeToTriumvirate;
    type CanVote = CanVoteToTriumvirate;
    type GetVotingMembers = GetSenateMemberCount;
}

// We call council members Triumvirate
#[allow(dead_code)]
type TriumvirateMembership = pallet_membership::Instance1;
impl pallet_membership::Config<TriumvirateMembership> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = Triumvirate;
    type MembershipChanged = Triumvirate;
    type MaxMembers = CouncilMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

// We call our top K delegates membership Senate
#[allow(dead_code)]
type SenateMembership = pallet_membership::Instance2;
impl pallet_membership::Config<SenateMembership> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
    type MaxMembers = SenateMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;

    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // According to multisig pallet, key and value size be computed as follows:
    // value size is `4 + sizeof((BlockNumber, Balance, AccountId))` bytes
    // key size is `32 + sizeof(AccountId)` bytes.
    // For our case, One storage item; key size is 32+32=64 bytes; value is size 4+4+8+32 bytes = 48 bytes.
    pub const DepositBase: Balance = deposit(1, 112);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
    type BlockNumberProvider = System;
}

// Proxy Pallet config
parameter_types! {
    // One storage item; key size sizeof(AccountId) = 32, value sizeof(Balance) = 8; 40 total
    pub const ProxyDepositBase: Balance = deposit(1, 40);
    // Adding 32 bytes + sizeof(ProxyType) = 32 + 1
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u32 = 20; // max num proxies per acct
    pub const MaxPending: u32 = 15 * 5; // max blocks pending ~15min
    // 16 bytes
    pub const AnnouncementDepositBase: Balance =  deposit(1, 16);
    // 68 bytes per announcement
    pub const AnnouncementDepositFactor: Balance = deposit(0, 68);
}

impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => !matches!(
                c,
                RuntimeCall::Balances(..)
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::schedule_swap_coldkey { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_coldkey { .. })
            ),
            ProxyType::NonFungibile => !matches!(
                c,
                RuntimeCall::Balances(..)
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_limit { .. }
                    )
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_full_limit { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::unstake_all { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::unstake_all_alpha { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::move_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::schedule_swap_coldkey { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_coldkey { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey { .. })
            ),
            ProxyType::Transfer => matches!(
                c,
                RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { .. })
                    | RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death { .. })
                    | RuntimeCall::Balances(pallet_balances::Call::transfer_all { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake { .. })
            ),
            ProxyType::SmallTransfer => match c {
                RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
                    value, ..
                }) => *value < SMALL_TRANSFER_LIMIT,
                RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                    value,
                    ..
                }) => *value < SMALL_TRANSFER_LIMIT,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake {
                    alpha_amount,
                    ..
                }) => *alpha_amount < SMALL_TRANSFER_LIMIT.into(),
                _ => false,
            },
            ProxyType::Owner => {
                matches!(
                    c,
                    RuntimeCall::AdminUtils(..)
                        | RuntimeCall::SubtensorModule(
                            pallet_subtensor::Call::set_subnet_identity { .. }
                        )
                        | RuntimeCall::SubtensorModule(
                            pallet_subtensor::Call::update_symbol { .. }
                        )
                ) && !matches!(
                    c,
                    RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_sn_owner_hotkey { .. }
                    )
                )
            }
            ProxyType::NonCritical => !matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::dissolve_network { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::Triumvirate(..)
                    | RuntimeCall::Sudo(..)
            ),
            ProxyType::Triumvirate => matches!(
                c,
                RuntimeCall::Triumvirate(..) | RuntimeCall::TriumvirateMembers(..)
            ),
            ProxyType::Senate => matches!(c, RuntimeCall::SenateMembers(..)),
            ProxyType::Governance => matches!(
                c,
                RuntimeCall::SenateMembers(..)
                    | RuntimeCall::Triumvirate(..)
                    | RuntimeCall::TriumvirateMembers(..)
            ),
            ProxyType::Staking => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::unstake_all { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::unstake_all_alpha { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::move_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_limit { .. }
                    )
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_full_limit { .. }
                    )
            ),
            ProxyType::Registration => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::register { .. })
            ),
            ProxyType::RootWeights => false, // deprecated
            ProxyType::ChildKeys => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_children { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::set_childkey_take { .. }
                    )
            ),
            ProxyType::SudoUncheckedSetCode => match c {
                RuntimeCall::Sudo(pallet_sudo::Call::sudo_unchecked_weight { call, weight: _ }) => {
                    let inner_call: RuntimeCall = *call.clone();

                    matches!(
                        inner_call,
                        RuntimeCall::System(frame_system::Call::set_code { .. })
                    )
                }
                _ => false,
            },
            ProxyType::SwapHotkey => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey { .. })
            ),
            ProxyType::SubnetLeaseBeneficiary => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::start_call { .. })
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_serving_rate_limit { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_min_difficulty { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_max_difficulty { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_weights_version_key { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_adjustment_alpha { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_max_weight_limit { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_immunity_period { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_min_allowed_weights { .. }
                    )
                    | RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_kappa { .. })
                    | RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_rho { .. })
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_activity_cutoff { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_network_registration_allowed { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_network_pow_registration_allowed { .. }
                    )
                    | RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_max_burn { .. })
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_bonds_moving_average { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_bonds_penalty { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_commit_reveal_weights_enabled { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_liquid_alpha_enabled { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_alpha_values { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_commit_reveal_weights_interval { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_toggle_transfer { .. }
                    )
            ),
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => {
                // NonTransfer is NOT a superset of Transfer or SmallTransfer
                !matches!(o, ProxyType::Transfer | ProxyType::SmallTransfer)
            }
            (ProxyType::Governance, ProxyType::Triumvirate | ProxyType::Senate) => true,
            (ProxyType::Transfer, ProxyType::SmallTransfer) => true,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
    type BlockNumberProvider = System;
}

pub struct Proxier;
impl ProxyInterface<AccountId> for Proxier {
    fn add_lease_beneficiary_proxy(lease: &AccountId, beneficiary: &AccountId) -> DispatchResult {
        pallet_proxy::Pallet::<Runtime>::add_proxy_delegate(
            lease,
            beneficiary.clone(),
            ProxyType::SubnetLeaseBeneficiary,
            0,
        )
    }

    fn remove_lease_beneficiary_proxy(
        lease: &AccountId,
        beneficiary: &AccountId,
    ) -> DispatchResult {
        pallet_proxy::Pallet::<Runtime>::remove_proxy_delegate(
            lease,
            beneficiary.clone(),
            ProxyType::SubnetLeaseBeneficiary,
            0,
        )
    }
}

pub struct CommitmentsI;
impl CommitmentsInterface for CommitmentsI {
    fn purge_netuid(netuid: NetUid) {
        pallet_commitments::Pallet::<Runtime>::purge_netuid(netuid);
    }
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}

/// Used the compare the privilege of an origin inside the scheduler.
pub struct OriginPrivilegeCmp;

impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
    fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
        if left == right {
            return Some(Ordering::Equal);
        }

        match (left, right) {
            // Root is greater than anything.
            (OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
            // Check which one has more yes votes.
            (
                OriginCaller::Triumvirate(pallet_collective::RawOrigin::Members(
                    l_yes_votes,
                    l_count,
                )),
                OriginCaller::Triumvirate(pallet_collective::RawOrigin::Members(
                    r_yes_votes,
                    r_count,
                )), // Equivalent to (l_yes_votes / l_count).cmp(&(r_yes_votes / r_count))
            ) => Some(
                l_yes_votes
                    .saturating_mul(*r_count)
                    .cmp(&r_yes_votes.saturating_mul(*l_count)),
            ),
            // For every other origin we don't care, as they are not used for `ScheduleOrigin`.
            _ => None,
        }
    }
}

impl pallet_scheduler::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = OriginPrivilegeCmp;
    type Preimages = Preimage;
    type BlockNumberProvider = System;
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = deposit(2, 64);
    pub const PreimageByteDeposit: Balance = deposit(0, 1);
    pub const PreimageHoldReason: RuntimeHoldReason =
        RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Consideration = HoldConsideration<
        AccountId,
        Balances,
        PreimageHoldReason,
        LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
    >;
}

pub struct AllowIdentityReg;

impl CanRegisterIdentity<AccountId> for AllowIdentityReg {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn can_register(address: &AccountId, identified: &AccountId) -> bool {
        if address != identified {
            SubtensorModule::coldkey_owns_hotkey(address, identified)
                && SubtensorModule::is_hotkey_registered_on_network(NetUid::ROOT, identified)
        } else {
            SubtensorModule::is_subnet_owner(address)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn can_register(_: &AccountId, _: &AccountId) -> bool {
        true
    }
}

// Configure registry pallet.
parameter_types! {
    pub const MaxAdditionalFields: u32 = 1;
    pub const InitialDeposit: Balance = 100_000_000; // 0.1 TAO
    pub const FieldDeposit: Balance = 100_000_000; // 0.1 TAO
}

impl pallet_registry::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type CanRegister = AllowIdentityReg;
    type WeightInfo = pallet_registry::weights::SubstrateWeight<Runtime>;

    type MaxAdditionalFields = MaxAdditionalFields;
    type InitialDeposit = InitialDeposit;
    type FieldDeposit = FieldDeposit;
}

parameter_types! {
    pub const MaxCommitFieldsInner: u32 = 3;
    pub const CommitmentInitialDeposit: Balance = 0; // Free
    pub const CommitmentFieldDeposit: Balance = 0; // Free
}

#[subtensor_macros::freeze_struct("7c76bd954afbb54e")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub struct MaxCommitFields;
impl Get<u32> for MaxCommitFields {
    fn get() -> u32 {
        MaxCommitFieldsInner::get()
    }
}

#[subtensor_macros::freeze_struct("c39297f5eb97ee82")]
pub struct AllowCommitments;
impl CanCommit<AccountId> for AllowCommitments {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn can_commit(netuid: NetUid, address: &AccountId) -> bool {
        SubtensorModule::is_hotkey_registered_on_network(netuid, address)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn can_commit(_: NetUid, _: &AccountId) -> bool {
        true
    }
}

pub struct ResetBondsOnCommit;
impl OnMetadataCommitment<AccountId> for ResetBondsOnCommit {
    #[cfg(not(feature = "runtime-benchmarks"))]
    fn on_metadata_commitment(netuid: NetUid, address: &AccountId) {
        // Reset bonds for each mechanism of this subnet
        let mechanism_count = SubtensorModule::get_current_mechanism_count(netuid);
        for mecid in 0..u8::from(mechanism_count) {
            let netuid_index = SubtensorModule::get_mechanism_storage_index(netuid, mecid.into());
            let _ = SubtensorModule::do_reset_bonds(netuid_index, address);
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn on_metadata_commitment(_: NetUid, _: &AccountId) {}
}

pub struct GetCommitmentsStruct;
impl GetCommitments<AccountId> for GetCommitmentsStruct {
    fn get_commitments(netuid: NetUid) -> Vec<(AccountId, Vec<u8>)> {
        pallet_commitments::Pallet::<Runtime>::get_commitments(netuid)
    }
}

impl pallet_commitments::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = pallet_commitments::weights::SubstrateWeight<Runtime>;

    type CanCommit = AllowCommitments;
    type OnMetadataCommitment = ResetBondsOnCommit;

    type MaxFields = MaxCommitFields;
    type InitialDeposit = CommitmentInitialDeposit;
    type FieldDeposit = CommitmentFieldDeposit;
    type TempoInterface = TempoInterface;
}

pub struct TempoInterface;
impl pallet_commitments::GetTempoInterface for TempoInterface {
    fn get_epoch_index(netuid: NetUid, cur_block: u64) -> u64 {
        SubtensorModule::get_epoch_index(netuid, cur_block)
    }
}

impl pallet_commitments::GetTempoInterface for Runtime {
    fn get_epoch_index(netuid: NetUid, cur_block: u64) -> u64 {
        SubtensorModule::get_epoch_index(netuid, cur_block)
    }
}

pub const INITIAL_SUBNET_TEMPO: u16 = prod_or_fast!(360, 10);

// 30 days at 12 seconds per block = 216000
pub const INITIAL_CHILDKEY_TAKE_RATELIMIT: u64 = prod_or_fast!(216000, 5);

// Configure the pallet subtensor.
parameter_types! {
    pub const SubtensorInitialRho: u16 = 10;
    pub const SubtensorInitialAlphaSigmoidSteepness: i16 = 1000;
    pub const SubtensorInitialKappa: u16 = 32_767; // 0.5 = 65535/2
    pub const SubtensorInitialMaxAllowedUids: u16 = 4096;
    pub const SubtensorInitialIssuance: u64 = 0;
    pub const SubtensorInitialMinAllowedWeights: u16 = 1024;
    pub const SubtensorInitialEmissionValue: u16 = 0;
    pub const SubtensorInitialMaxWeightsLimit: u16 = 1000; // 1000/2^16 = 0.015
    pub const SubtensorInitialValidatorPruneLen: u64 = 1;
    pub const SubtensorInitialScalingLawPower: u16 = 50; // 0.5
    pub const SubtensorInitialMaxAllowedValidators: u16 = 128;
    pub const SubtensorInitialTempo: u16 = INITIAL_SUBNET_TEMPO;
    pub const SubtensorInitialDifficulty: u64 = 10_000_000;
    pub const SubtensorInitialAdjustmentInterval: u16 = 100;
    pub const SubtensorInitialAdjustmentAlpha: u64 = 0; // no weight to previous value.
    pub const SubtensorInitialTargetRegistrationsPerInterval: u16 = 2;
    pub const SubtensorInitialImmunityPeriod: u16 = 4096;
    pub const SubtensorInitialActivityCutoff: u16 = 5000;
    pub const SubtensorInitialMaxRegistrationsPerBlock: u16 = 1;
    pub const SubtensorInitialPruningScore : u16 = u16::MAX;
    pub const SubtensorInitialBondsMovingAverage: u64 = 900_000;
    pub const SubtensorInitialBondsPenalty: u16 = u16::MAX;
    pub const SubtensorInitialBondsResetOn: bool = false;
    pub const SubtensorInitialDefaultTake: u16 = 11_796; // 18% honest number.
    pub const SubtensorInitialMinDelegateTake: u16 = 0; // Allow 0% delegate take
    pub const SubtensorInitialDefaultChildKeyTake: u16 = 0; // Allow 0% childkey take
    pub const SubtensorInitialMinChildKeyTake: u16 = 0; // 0 %
    pub const SubtensorInitialMaxChildKeyTake: u16 = 11_796; // 18 %
    pub const SubtensorInitialWeightsVersionKey: u64 = 0;
    pub const SubtensorInitialMinDifficulty: u64 = 10_000_000;
    pub const SubtensorInitialMaxDifficulty: u64 = u64::MAX / 4;
    pub const SubtensorInitialServingRateLimit: u64 = 50;
    pub const SubtensorInitialBurn: u64 = 100_000_000; // 0.1 tao
    pub const SubtensorInitialMinBurn: u64 = 500_000; // 500k RAO
    pub const SubtensorInitialMaxBurn: u64 = 100_000_000_000; // 100 tao
    pub const MinBurnUpperBound: TaoCurrency = TaoCurrency::new(1_000_000_000); // 1 TAO
    pub const MaxBurnLowerBound: TaoCurrency = TaoCurrency::new(100_000_000); // 0.1 TAO
    pub const SubtensorInitialTxRateLimit: u64 = 1000;
    pub const SubtensorInitialTxDelegateTakeRateLimit: u64 = 216000; // 30 days at 12 seconds per block
    pub const SubtensorInitialTxChildKeyTakeRateLimit: u64 = INITIAL_CHILDKEY_TAKE_RATELIMIT;
    pub const SubtensorInitialRAORecycledForRegistration: u64 = 0; // 0 rao
    pub const SubtensorInitialSenateRequiredStakePercentage: u64 = 1; // 1 percent of total stake
    pub const SubtensorInitialNetworkImmunity: u64 = 1_296_000;
    pub const SubtensorInitialMinAllowedUids: u16 = 64;
    pub const SubtensorInitialMinLockCost: u64 = 1_000_000_000_000; // 1000 TAO
    pub const SubtensorInitialSubnetOwnerCut: u16 = 11_796; // 18 percent
    // pub const SubtensorInitialSubnetLimit: u16 = 12; // (DEPRECATED)
    pub const SubtensorInitialNetworkLockReductionInterval: u64 = 14 * 7200;
    pub const SubtensorInitialNetworkRateLimit: u64 = 7200;
    pub const SubtensorInitialKeySwapCost: u64 = 100_000_000; // 0.1 TAO
    pub const InitialAlphaHigh: u16 = 58982; // Represents 0.9 as per the production default
    pub const InitialAlphaLow: u16 = 45875; // Represents 0.7 as per the production default
    pub const InitialLiquidAlphaOn: bool = false; // Default value for LiquidAlphaOn
    pub const InitialYuma3On: bool = false; // Default value for Yuma3On
    // pub const SubtensorInitialNetworkMaxStake: u64 = u64::MAX; // (DEPRECATED)
    pub const InitialColdkeySwapScheduleDuration: BlockNumber = 5 * 24 * 60 * 60 / 12; // 5 days
    pub const InitialColdkeySwapRescheduleDuration: BlockNumber = 24 * 60 * 60 / 12; // 1 day
    pub const InitialDissolveNetworkScheduleDuration: BlockNumber = 5 * 24 * 60 * 60 / 12; // 5 days
    pub const SubtensorInitialTaoWeight: u64 = 971_718_665_099_567_868; // 0.05267697438728329% tao weight.
    pub const InitialEmaPriceHalvingPeriod: u64 = 201_600_u64; // 4 weeks
    // 7 * 24 * 60 * 60 / 12 = 7 days
    pub const DurationOfStartCall: u64 = prod_or_fast!(7 * 24 * 60 * 60 / 12, 10);
    pub const SubtensorInitialKeySwapOnSubnetCost: u64 = 1_000_000; // 0.001 TAO
    pub const HotkeySwapOnSubnetInterval : BlockNumber = 5 * 24 * 60 * 60 / 12; // 5 days
    pub const LeaseDividendsDistributionInterval: BlockNumber = 100; // 100 blocks
    pub const MaxImmuneUidsPercentage: Percent = Percent::from_percent(80);
}

impl pallet_subtensor::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type SudoRuntimeCall = RuntimeCall;
    type Currency = Balances;
    type CouncilOrigin = EnsureMajoritySenate;
    type SenateMembers = ManageSenateMembers;
    type TriumvirateInterface = TriumvirateVotes;
    type Scheduler = Scheduler;
    type InitialRho = SubtensorInitialRho;
    type InitialAlphaSigmoidSteepness = SubtensorInitialAlphaSigmoidSteepness;
    type InitialKappa = SubtensorInitialKappa;
    type InitialMinAllowedUids = SubtensorInitialMinAllowedUids;
    type InitialMaxAllowedUids = SubtensorInitialMaxAllowedUids;
    type InitialBondsMovingAverage = SubtensorInitialBondsMovingAverage;
    type InitialBondsPenalty = SubtensorInitialBondsPenalty;
    type InitialBondsResetOn = SubtensorInitialBondsResetOn;
    type InitialIssuance = SubtensorInitialIssuance;
    type InitialMinAllowedWeights = SubtensorInitialMinAllowedWeights;
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
    type InitialDefaultDelegateTake = SubtensorInitialDefaultTake;
    type InitialDefaultChildKeyTake = SubtensorInitialDefaultChildKeyTake;
    type InitialMinDelegateTake = SubtensorInitialMinDelegateTake;
    type InitialMinChildKeyTake = SubtensorInitialMinChildKeyTake;
    type InitialWeightsVersionKey = SubtensorInitialWeightsVersionKey;
    type InitialMaxDifficulty = SubtensorInitialMaxDifficulty;
    type InitialMinDifficulty = SubtensorInitialMinDifficulty;
    type InitialServingRateLimit = SubtensorInitialServingRateLimit;
    type InitialBurn = SubtensorInitialBurn;
    type InitialMaxBurn = SubtensorInitialMaxBurn;
    type InitialMinBurn = SubtensorInitialMinBurn;
    type MinBurnUpperBound = MinBurnUpperBound;
    type MaxBurnLowerBound = MaxBurnLowerBound;
    type InitialTxRateLimit = SubtensorInitialTxRateLimit;
    type InitialTxDelegateTakeRateLimit = SubtensorInitialTxDelegateTakeRateLimit;
    type InitialTxChildKeyTakeRateLimit = SubtensorInitialTxChildKeyTakeRateLimit;
    type InitialMaxChildKeyTake = SubtensorInitialMaxChildKeyTake;
    type InitialRAORecycledForRegistration = SubtensorInitialRAORecycledForRegistration;
    type InitialSenateRequiredStakePercentage = SubtensorInitialSenateRequiredStakePercentage;
    type InitialNetworkImmunityPeriod = SubtensorInitialNetworkImmunity;
    type InitialNetworkMinLockCost = SubtensorInitialMinLockCost;
    type InitialNetworkLockReductionInterval = SubtensorInitialNetworkLockReductionInterval;
    type InitialSubnetOwnerCut = SubtensorInitialSubnetOwnerCut;
    type InitialNetworkRateLimit = SubtensorInitialNetworkRateLimit;
    type KeySwapCost = SubtensorInitialKeySwapCost;
    type AlphaHigh = InitialAlphaHigh;
    type AlphaLow = InitialAlphaLow;
    type LiquidAlphaOn = InitialLiquidAlphaOn;
    type Yuma3On = InitialYuma3On;
    type InitialTaoWeight = SubtensorInitialTaoWeight;
    type Preimages = Preimage;
    type InitialColdkeySwapScheduleDuration = InitialColdkeySwapScheduleDuration;
    type InitialColdkeySwapRescheduleDuration = InitialColdkeySwapRescheduleDuration;
    type InitialDissolveNetworkScheduleDuration = InitialDissolveNetworkScheduleDuration;
    type InitialEmaPriceHalvingPeriod = InitialEmaPriceHalvingPeriod;
    type DurationOfStartCall = DurationOfStartCall;
    type SwapInterface = Swap;
    type KeySwapOnSubnetCost = SubtensorInitialKeySwapOnSubnetCost;
    type HotkeySwapOnSubnetInterval = HotkeySwapOnSubnetInterval;
    type ProxyInterface = Proxier;
    type LeaseDividendsDistributionInterval = LeaseDividendsDistributionInterval;
    type GetCommitments = GetCommitmentsStruct;
    type MaxImmuneUidsPercentage = MaxImmuneUidsPercentage;
    type CommitmentsInterface = CommitmentsI;
}

parameter_types! {
    pub const SwapProtocolId: PalletId = PalletId(*b"ten/swap");
    pub const SwapMaxFeeRate: u16 = 10000; // 15.26%
    pub const SwapMaxPositions: u32 = 100;
    pub const SwapMinimumLiquidity: u64 = 1_000;
    pub const SwapMinimumReserve: NonZeroU64 = NonZeroU64::new(1_000_000)
        .expect("1_000_000 fits NonZeroU64");
}

impl pallet_subtensor_swap::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SubnetInfo = SubtensorModule;
    type BalanceOps = SubtensorModule;
    type ProtocolId = SwapProtocolId;
    type MaxFeeRate = SwapMaxFeeRate;
    type MaxPositions = SwapMaxPositions;
    type MinimumLiquidity = SwapMinimumLiquidity;
    type MinimumReserve = SwapMinimumReserve;
    // TODO: set measured weights when the pallet been benchmarked and the type is generated
    type WeightInfo = pallet_subtensor_swap::weights::DefaultWeight<Runtime>;
}

impl pallet_admin_utils::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
}

/// Define the ChainId
/// EVM Chain ID will be set by sudo transaction for each chain
///     Mainnet Finney: 0x03C4 - Unicode for lowercase tau
///     TestNet Finney: 0x03B1 - Unicode for lowercase alpha
impl pallet_evm_chain_id::Config for Runtime {}

pub struct ConfigurableChainId;

impl Get<u64> for ConfigurableChainId {
    fn get() -> u64 {
        pallet_evm_chain_id::ChainId::<Runtime>::get()
    }
}

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        if let Some(author_index) = F::find_author(digests) {
            pallet_babe::Authorities::<Runtime>::get()
                .get(author_index as usize)
                .and_then(|authority_id| {
                    let raw_vec = authority_id.0.to_raw_vec();
                    raw_vec.get(4..24).map(H160::from_slice)
                })
        } else {
            None
        }
    }
}

const BLOCK_GAS_LIMIT: u64 = 75_000_000;

/// `WeightPerGas` is an approximate ratio of the amount of Weight per Gas.
///
fn weight_per_gas() -> Weight {
    (NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT)
        .checked_div(BLOCK_GAS_LIMIT)
        .unwrap_or_default()
}

parameter_types! {
    pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
    pub const GasLimitPovSizeRatio: u64 = 0;
    pub PrecompilesValue: Precompiles<Runtime> = Precompiles::<_>::new();
    pub WeightPerGas: Weight = weight_per_gas();
}

/// The difference between EVM decimals and Substrate decimals.
/// Substrate balances has 9 decimals, while EVM has 18, so the
/// difference factor is 9 decimals, or 10^9
const EVM_TO_SUBSTRATE_DECIMALS: u64 = 1_000_000_000_u64;

pub struct SubtensorEvmBalanceConverter;

impl BalanceConverter for SubtensorEvmBalanceConverter {
    /// Convert from Substrate balance (u64) to EVM balance (U256)
    fn into_evm_balance(value: SubstrateBalance) -> Option<EvmBalance> {
        let value = value.into_u256();
        if let Some(evm_value) = value.checked_mul(U256::from(EVM_TO_SUBSTRATE_DECIMALS)) {
            // Ensure the result fits within the maximum U256 value
            if evm_value <= U256::MAX {
                Some(EvmBalance::new(evm_value))
            } else {
                // Log value too large
                log::debug!(
                    "SubtensorEvmBalanceConverter::into_evm_balance( {value:?} ) larger than U256::MAX"
                );
                None
            }
        } else {
            // Log overflow
            log::debug!("SubtensorEvmBalanceConverter::into_evm_balance( {value:?} ) overflow");
            None
        }
    }

    /// Convert from EVM balance (U256) to Substrate balance (u64)
    fn into_substrate_balance(value: EvmBalance) -> Option<SubstrateBalance> {
        let value = value.into_u256();
        if let Some(substrate_value) = value.checked_div(U256::from(EVM_TO_SUBSTRATE_DECIMALS)) {
            // Ensure the result fits within the TAO balance type (u64)
            if substrate_value <= U256::from(u64::MAX) {
                Some(SubstrateBalance::new(substrate_value))
            } else {
                // Log value too large
                log::debug!(
                    "SubtensorEvmBalanceConverter::into_substrate_balance( {value:?} ) larger than u64::MAX"
                );
                None
            }
        } else {
            // Log overflow
            log::debug!(
                "SubtensorEvmBalanceConverter::into_substrate_balance( {value:?} ) overflow"
            );
            None
        }
    }
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = BaseFee;
    type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = pallet_evm::EnsureAddressTruncated;
    type WithdrawOrigin = pallet_evm::EnsureAddressTruncated;
    type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type PrecompilesType = Precompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = ConfigurableChainId;
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type OnChargeTransaction = ();
    type OnCreate = ();
    type FindAuthor = FindAuthorTruncated<Babe>;
    // type FindAuthor = FindAuthorTruncated<Aura>;
    type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
    type Timestamp = Timestamp;
    type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
    type BalanceConverter = SubtensorEvmBalanceConverter;
    type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
    type GasLimitStorageGrowthRatio = ();
    type CreateOriginFilter = ();
    type CreateInnerOriginFilter = ();
}

parameter_types! {
    pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

// Required for the IntermediateStateRoot
impl sp_core::Get<sp_version::RuntimeVersion> for Runtime {
    fn get() -> sp_version::RuntimeVersion {
        VERSION
    }
}

impl pallet_ethereum::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
    type PostLogContent = PostBlockAndTxnHashes;
    type ExtraDataLength = ConstU32<30>;
}

parameter_types! {
    pub BoundDivision: U256 = U256::from(1024);
}

parameter_types! {
    pub DefaultBaseFeePerGas: U256 = U256::from(20_000_000_000_u128);
    pub DefaultElasticity: Permill = Permill::from_parts(125_000);
}
pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
    fn lower() -> Permill {
        Permill::zero()
    }
    fn ideal() -> Permill {
        Permill::from_parts(500_000)
    }
    fn upper() -> Permill {
        Permill::from_parts(1_000_000)
    }
}
impl pallet_base_fee::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Threshold = BaseFeeThreshold;
    type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
    type DefaultElasticity = DefaultElasticity;
}

#[derive(Clone)]
pub struct TransactionConverter<B>(PhantomData<B>);

impl<B> Default for TransactionConverter<B> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<B: BlockT> fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> for TransactionConverter<B> {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> <B as BlockT>::Extrinsic {
        let extrinsic = UncheckedExtrinsic::new_bare(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        );
        let encoded = extrinsic.encode();
        <B as BlockT>::Extrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            RuntimeCall::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            RuntimeCall::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Option<TransactionValidity> {
        match self {
            RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            RuntimeCall::Ethereum(call) => {
                call.pre_dispatch_self_contained(info, dispatch_info, len)
            }
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
                Some(call.dispatch(RuntimeOrigin::from(
                    pallet_ethereum::RawOrigin::EthereumTransaction(info),
                )))
            }
            _ => None,
        }
    }
}

// Crowdloan
parameter_types! {
    pub const CrowdloanPalletId: PalletId = PalletId(*b"bt/cloan");
    pub const MinimumDeposit: Balance = 10_000_000_000; // 10 TAO
    pub const AbsoluteMinimumContribution: Balance = 100_000_000; // 0.1 TAO
    // 7 days minimum (7 * 24 * 60 * 60 / 12)
    pub const MinimumBlockDuration: BlockNumber = prod_or_fast!(50400, 50);
    // 60 days maximum (60 * 24 * 60 * 60 / 12)
    pub const MaximumBlockDuration: BlockNumber = prod_or_fast!(432000, 20000);
    pub const RefundContributorsLimit: u32 = 50;
    pub const MaxContributors: u32 = 500;
}

impl pallet_crowdloan::Config for Runtime {
    type PalletId = CrowdloanPalletId;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type WeightInfo = pallet_crowdloan::weights::SubstrateWeight<Runtime>;
    type Preimages = Preimage;
    type MinimumDeposit = MinimumDeposit;
    type AbsoluteMinimumContribution = AbsoluteMinimumContribution;
    type MinimumBlockDuration = MinimumBlockDuration;
    type MaximumBlockDuration = MaximumBlockDuration;
    type RefundContributorsLimit = RefundContributorsLimit;
    type MaxContributors = MaxContributors;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub struct Runtime
    {
        System: frame_system = 0,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1,
        Timestamp: pallet_timestamp = 2,
        Aura: pallet_aura = 3,
        Grandpa: pallet_grandpa = 4,
        Balances: pallet_balances = 5,
        TransactionPayment: pallet_transaction_payment = 6,
        SubtensorModule: pallet_subtensor = 7,
        Triumvirate: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 8,
        TriumvirateMembers: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 9,
        SenateMembers: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 10,
        Utility: pallet_utility = 11,
        Sudo: pallet_sudo = 12,
        Multisig: pallet_multisig = 13,
        Preimage: pallet_preimage = 14,
        Scheduler: pallet_scheduler = 15,
        Proxy: pallet_proxy = 16,
        Registry: pallet_registry = 17,
        Commitments: pallet_commitments = 18,
        AdminUtils: pallet_admin_utils = 19,
        SafeMode: pallet_safe_mode = 20,

        // Frontier
        Ethereum: pallet_ethereum = 21,
        EVM: pallet_evm = 22,
        EVMChainId: pallet_evm_chain_id = 23,
        // pallet_dynamic_fee was 24
        BaseFee: pallet_base_fee = 25,

        Drand: pallet_drand = 26,
        Crowdloan: pallet_crowdloan = 27,
        Swap: pallet_subtensor_swap = 28,

        // NPoS Consensus.
        // Authorship must be before session in order to note author in the correct session and era
        // for staking.
        Authorship: pallet_authorship = 30,
        Staking: pallet_staking = 31,
        Offences: pallet_offences = 32,
        Historical: session_historical = 33,
        Session: pallet_session = 34,
        Babe: pallet_babe = 36,
        ElectionProviderMultiPhase: pallet_election_provider_multi_phase = 37,
        VoterList: pallet_bags_list::<Instance1> = 38,
        FastUnstake: pallet_fast_unstake = 39,
    }
);

// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
// The extensions to the basic transaction logic.
pub type TransactionExtensions = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    check_nonce::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    ChargeTransactionPaymentWrapper<Runtime>,
    pallet_subtensor::transaction_extension::SubtensorTransactionExtension<Runtime>,
    pallet_drand::drand_priority::DrandPriority<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);

type Migrations = (
    // Leave this migration in the runtime, so every runtime upgrade tiny rounding errors (fractions of fractions
    // of a cent) are cleaned up. These tiny rounding errors occur due to floating point coversion.
    pallet_subtensor::migrations::migrate_init_total_issuance::initialise_total_issuance::Migration<
        Runtime,
    >,
    crate::migrations::babe_npos::Migration<Runtime>,
);

// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, TransactionExtensions>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic =
    fp_self_contained::CheckedExtrinsic<AccountId, RuntimeCall, TransactionExtensions, H160>;

// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TransactionExtensions>;
// Executive: handles dispatch to the various modules.
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
        [pallet_timestamp, Timestamp]
        [pallet_sudo, Sudo]
        [pallet_registry, Registry]
        [pallet_commitments, Commitments]
        [pallet_admin_utils, AdminUtils]
        [pallet_subtensor, SubtensorModule]
        [pallet_drand, Drand]
        [pallet_crowdloan, Crowdloan]
        [pallet_subtensor_swap, Swap]
    );
}

fn generate_genesis_json() -> Vec<u8> {
    let json_str = r#"{
      "balances": {
        "balances": [
          [
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            1000000000000000
          ],
          [
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            1000000000000000
          ]
        ]
      },
      "grandpa": {
        "authorities": [
          [
            "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
            1
          ]
        ]
      },
      "sudo": {
        "key": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
      },
      "subtensorModule": {
        "balancesIssuance": 0,
        "stakes": []
      }
    }"#;

    json_str.as_bytes().to_vec()
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
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

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            get_preset::<RuntimeGenesisConfig>(id, |preset_id| {
                let benchmark_id: sp_genesis_builder::PresetId = "benchmark".into();
                if *preset_id == benchmark_id {
                    // None
                    Some(generate_genesis_json())
                } else {
                    None
                }
            })
        }

        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
            vec!["benchmark".into()]
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            use codec::DecodeLimit;
            use frame_support::pallet_prelude::{InvalidTransaction, TransactionValidityError};
            use frame_support::traits::ExtrinsicCall;
            let encoded = tx.call().encode();
            if RuntimeCall::decode_all_with_depth_limit(8, &mut encoded.as_slice()).is_err() {
                log::warn!("failed to decode with depth limit of 8");
                return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
            }
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
            pallet_aura::Authorities::<Runtime>::get().into_inner()
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

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> Vec<(GrandpaId, u64)> {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: fg_primitives::AuthorityId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
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

    impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
        fn chain_id() -> u64 {
            <Runtime as pallet_evm::Config>::ChainId::get()
        }

        fn account_basic(address: H160) -> EVMAccount {
            let (account, _) = pallet_evm::Pallet::<Runtime>::account_basic(&address);
            account
        }

        fn gas_price() -> U256 {
            let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
            gas_price
        }

        fn account_code_at(address: H160) -> Vec<u8> {
            pallet_evm::AccountCodes::<Runtime>::get(address)
        }

        fn author() -> H160 {
            <pallet_evm::Pallet<Runtime>>::find_author()
        }

        fn storage_at(address: H160, index: U256) -> H256 {
            let index_hash = H256::from_slice(&index.to_big_endian());
            pallet_evm::AccountStorages::<Runtime>::get(address, index_hash)
        }

        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
            use pallet_evm::GasWeightMapping as _;

            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

                    // Estimated encoded transaction size must be based on the heaviest transaction
                    // type (EIP1559Transaction) to be compatible with all transaction types.
                    let mut estimated_transaction_len = data.len() +
                        // pallet ethereum index: 1
                        // transact call index: 1
                        // Transaction enum variant: 1
                        // chain_id 8 bytes
                        // nonce: 32
                        // max_priority_fee_per_gas: 32
                        // max_fee_per_gas: 32
                        // gas_limit: 32
                        // action: 21 (enum varianrt + call address)
                        // value: 32
                        // access_list: 1 (empty vec size)
                        // 65 bytes signature
                        258;

                    if access_list.is_some() {
                        estimated_transaction_len += access_list.encoded_size();
                    }


                    let gas_limit = if gas_limit > U256::from(u64::MAX) {
                        u64::MAX
                    } else {
                        gas_limit.low_u64()
                    };
            let without_base_extrinsic_weight = true;

            let (weight_limit, proof_size_base_cost) =
                match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
                    gas_limit,
                    without_base_extrinsic_weight
                ) {
                    weight_limit if weight_limit.proof_size() > 0 => {
                        (Some(weight_limit), Some(estimated_transaction_len as u64))
                    }
                    _ => (None, None),
                };

            <Runtime as pallet_evm::Config>::Runner::call(
                from,
                to,
                data,
                value,
                gas_limit.unique_saturated_into(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                false,
                true,
                weight_limit,
                proof_size_base_cost,
                config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
            ).map_err(|err| err.error.into())
        }

        fn create(
            from: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
            use pallet_evm::GasWeightMapping as _;

            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };


            let mut estimated_transaction_len = data.len() +
                // from: 20
                // value: 32
                // gas_limit: 32
                // nonce: 32
                // 1 byte transaction action variant
                // chain id 8 bytes
                // 65 bytes signature
                190;

            if max_fee_per_gas.is_some() {
                estimated_transaction_len += 32;
            }
            if max_priority_fee_per_gas.is_some() {
                estimated_transaction_len += 32;
            }
            if access_list.is_some() {
                estimated_transaction_len += access_list.encoded_size();
            }


            let gas_limit = if gas_limit > U256::from(u64::MAX) {
                u64::MAX
            } else {
                gas_limit.low_u64()
            };
            let without_base_extrinsic_weight = true;

            let (weight_limit, proof_size_base_cost) =
                match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
                    gas_limit,
                    without_base_extrinsic_weight
                ) {
                    weight_limit if weight_limit.proof_size() > 0 => {
                        (Some(weight_limit), Some(estimated_transaction_len as u64))
                    }
                    _ => (None, None),
                };

            let whitelist = pallet_evm::WhitelistedCreators::<Runtime>::get();
            let whitelist_disabled = pallet_evm::DisableWhitelistCheck::<Runtime>::get();
            <Runtime as pallet_evm::Config>::Runner::create(
                from,
                data,
                value,
                gas_limit.unique_saturated_into(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                whitelist,
                whitelist_disabled,
                false,
                true,
                weight_limit,
                proof_size_base_cost,
                config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
            ).map_err(|err| err.error.into())
        }

        fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
            pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
        }

        fn current_block() -> Option<pallet_ethereum::Block> {
            pallet_ethereum::CurrentBlock::<Runtime>::get()
        }

        fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
            pallet_ethereum::CurrentReceipts::<Runtime>::get()
        }

        fn current_all() -> (
            Option<pallet_ethereum::Block>,
            Option<Vec<pallet_ethereum::Receipt>>,
            Option<Vec<TransactionStatus>>
        ) {
            (
                pallet_ethereum::CurrentBlock::<Runtime>::get(),
                pallet_ethereum::CurrentReceipts::<Runtime>::get(),
                pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
            )
        }

        fn extrinsic_filter(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> Vec<EthereumTransaction> {
            xts.into_iter().filter_map(|xt| match xt.0.function {
                RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
                _ => None
            }).collect::<Vec<EthereumTransaction>>()
        }

        fn elasticity() -> Option<Permill> {
            Some(pallet_base_fee::Elasticity::<Runtime>::get())
        }

        fn gas_limit_multiplier_support() {}

        fn pending_block(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> (Option<pallet_ethereum::Block>, Option<Vec<TransactionStatus>>) {
            for ext in xts.into_iter() {
                let _ = Executive::apply_extrinsic(ext);
            }

            Ethereum::on_finalize(System::block_number() + 1);

            (
                pallet_ethereum::CurrentBlock::<Runtime>::get(),
                pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
            )
        }

        fn initialize_pending_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header);
        }
    }

    impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
        fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
            UncheckedExtrinsic::new_bare(
                pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
            )
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, BenchmarkList};
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
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
            use frame_benchmarking::{baseline, BenchmarkBatch};
            use sp_storage::TrackedStorageKey;

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            #[allow(non_local_definitions)]
            impl frame_system_benchmarking::Config for Runtime {}

            #[allow(non_local_definitions)]
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
        #[allow(clippy::unwrap_used)]
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

    impl subtensor_custom_rpc_runtime_api::DelegateInfoRuntimeApi<Block> for Runtime {
        fn get_delegates() -> Vec<DelegateInfo<AccountId32>> {
            SubtensorModule::get_delegates()
        }

        fn get_delegate(delegate_account: AccountId32) -> Option<DelegateInfo<AccountId32>> {
            SubtensorModule::get_delegate(delegate_account)
        }

        fn get_delegated(delegatee_account: AccountId32) -> Vec<(DelegateInfo<AccountId32>, (Compact<NetUid>, Compact<AlphaCurrency>))> {
            SubtensorModule::get_delegated(delegatee_account)
        }
    }

    impl subtensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi<Block> for Runtime {
        fn get_neurons_lite(netuid: NetUid) -> Vec<NeuronInfoLite<AccountId32>> {
            SubtensorModule::get_neurons_lite(netuid)
        }

        fn get_neuron_lite(netuid: NetUid, uid: u16) -> Option<NeuronInfoLite<AccountId32>> {
            SubtensorModule::get_neuron_lite(netuid, uid)
        }

        fn get_neurons(netuid: NetUid) -> Vec<NeuronInfo<AccountId32>> {
            SubtensorModule::get_neurons(netuid)
        }

        fn get_neuron(netuid: NetUid, uid: u16) -> Option<NeuronInfo<AccountId32>> {
            SubtensorModule::get_neuron(netuid, uid)
        }
    }

    impl subtensor_custom_rpc_runtime_api::SubnetInfoRuntimeApi<Block> for Runtime {
        fn get_subnet_info(netuid: NetUid) -> Option<SubnetInfo<AccountId32>> {
            SubtensorModule::get_subnet_info(netuid)
        }

        fn get_subnets_info() -> Vec<Option<SubnetInfo<AccountId32>>> {
            SubtensorModule::get_subnets_info()
        }

        fn get_subnet_info_v2(netuid: NetUid) -> Option<SubnetInfov2<AccountId32>> {
            SubtensorModule::get_subnet_info_v2(netuid)
        }

        fn get_subnets_info_v2() -> Vec<Option<SubnetInfov2<AccountId32>>> {
            SubtensorModule::get_subnets_info_v2()
        }

        fn get_subnet_hyperparams(netuid: NetUid) -> Option<SubnetHyperparams> {
            SubtensorModule::get_subnet_hyperparams(netuid)
        }

        fn get_subnet_hyperparams_v2(netuid: NetUid) -> Option<SubnetHyperparamsV2> {
            SubtensorModule::get_subnet_hyperparams_v2(netuid)
        }

        fn get_dynamic_info(netuid: NetUid) -> Option<DynamicInfo<AccountId32>> {
            SubtensorModule::get_dynamic_info(netuid)
        }

        fn get_metagraph(netuid: NetUid) -> Option<Metagraph<AccountId32>> {
            SubtensorModule::get_metagraph(netuid)
        }

        fn get_mechagraph(netuid: NetUid, mecid: MechId) -> Option<Metagraph<AccountId32>> {
            SubtensorModule::get_mechagraph(netuid, mecid)
        }

        fn get_subnet_state(netuid: NetUid) -> Option<SubnetState<AccountId32>> {
            SubtensorModule::get_subnet_state(netuid)
        }

        fn get_all_metagraphs() -> Vec<Option<Metagraph<AccountId32>>> {
            SubtensorModule::get_all_metagraphs()
        }

        fn get_all_mechagraphs() -> Vec<Option<Metagraph<AccountId32>>> {
            SubtensorModule::get_all_mechagraphs()
        }

        fn get_all_dynamic_info() -> Vec<Option<DynamicInfo<AccountId32>>> {
            SubtensorModule::get_all_dynamic_info()
        }

        fn get_selective_metagraph(netuid: NetUid, metagraph_indexes: Vec<u16>) -> Option<SelectiveMetagraph<AccountId32>> {
            SubtensorModule::get_selective_metagraph(netuid, metagraph_indexes)
        }
        fn get_subnet_to_prune() -> Option<NetUid> {
        pallet_subtensor::Pallet::<Runtime>::get_network_to_prune()
        }

        fn get_selective_mechagraph(netuid: NetUid, mecid: MechId, metagraph_indexes: Vec<u16>) -> Option<SelectiveMetagraph<AccountId32>> {
            SubtensorModule::get_selective_mechagraph(netuid, mecid, metagraph_indexes)
        }
    }

    impl subtensor_custom_rpc_runtime_api::StakeInfoRuntimeApi<Block> for Runtime {
        fn get_stake_info_for_coldkey( coldkey_account: AccountId32 ) -> Vec<StakeInfo<AccountId32>> {
            SubtensorModule::get_stake_info_for_coldkey( coldkey_account )
        }

        fn get_stake_info_for_coldkeys( coldkey_accounts: Vec<AccountId32> ) -> Vec<(AccountId32, Vec<StakeInfo<AccountId32>>)> {
            SubtensorModule::get_stake_info_for_coldkeys( coldkey_accounts )
        }

        fn get_stake_info_for_hotkey_coldkey_netuid( hotkey_account: AccountId32, coldkey_account: AccountId32, netuid: NetUid ) -> Option<StakeInfo<AccountId32>> {
            SubtensorModule::get_stake_info_for_hotkey_coldkey_netuid( hotkey_account, coldkey_account, netuid )
        }

        fn get_stake_fee( origin: Option<(AccountId32, NetUid)>, origin_coldkey_account: AccountId32, destination: Option<(AccountId32, NetUid)>, destination_coldkey_account: AccountId32, amount: u64 ) -> u64 {
            SubtensorModule::get_stake_fee( origin, origin_coldkey_account, destination, destination_coldkey_account, amount )
        }
    }

    impl subtensor_custom_rpc_runtime_api::SubnetRegistrationRuntimeApi<Block> for Runtime {
        fn get_network_registration_cost() -> TaoCurrency {
            SubtensorModule::get_network_lock_cost()
        }
    }

    impl pallet_staking_runtime_api::StakingApi<Block, Balance, AccountId> for Runtime {
        fn nominations_quota(balance: Balance) -> u32 {
            Staking::api_nominations_quota(balance)
        }

        fn eras_stakers_page_count(era: sp_staking::EraIndex, account: AccountId) -> sp_staking::Page {
            Staking::api_eras_stakers_page_count(era, account)
        }

        fn pending_rewards(era: sp_staking::EraIndex, account: AccountId) -> bool {
            Staking::api_pending_rewards(era, account)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: epoch_config.c,
                authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: epoch_config.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl pallet_subtensor_swap_runtime_api::SwapRuntimeApi<Block> for Runtime {
        fn current_alpha_price(netuid: NetUid) -> u64 {
            use substrate_fixed::types::U96F32;

            pallet_subtensor_swap::Pallet::<Runtime>::current_price(netuid.into())
                .saturating_mul(U96F32::from_num(1_000_000_000))
                .saturating_to_num()
        }

        fn sim_swap_tao_for_alpha(netuid: NetUid, tao: TaoCurrency) -> SimSwapResult {
            pallet_subtensor_swap::Pallet::<Runtime>::sim_swap(
                netuid.into(),
                OrderType::Buy,
                tao.into(),
            )
            .map_or_else(
                |_| SimSwapResult {
                    tao_amount:   0.into(),
                    alpha_amount: 0.into(),
                    tao_fee:      0.into(),
                    alpha_fee:    0.into(),
                },
                |sr| SimSwapResult {
                    tao_amount:   sr.amount_paid_in.into(),
                    alpha_amount: sr.amount_paid_out.into(),
                    tao_fee:      sr.fee_paid.into(),
                    alpha_fee:    0.into(),
                },
            )
        }

        fn sim_swap_alpha_for_tao(netuid: NetUid, alpha: AlphaCurrency) -> SimSwapResult {
            pallet_subtensor_swap::Pallet::<Runtime>::sim_swap(
                netuid.into(),
                OrderType::Sell,
                alpha.into(),
            )
            .map_or_else(
                |_| SimSwapResult {
                    tao_amount:   0.into(),
                    alpha_amount: 0.into(),
                    tao_fee:      0.into(),
                    alpha_fee:    0.into(),
                },
                |sr| SimSwapResult {
                    tao_amount:   sr.amount_paid_out.into(),
                    alpha_amount: sr.amount_paid_in.into(),
                    tao_fee:      0.into(),
                    alpha_fee:    sr.fee_paid.into(),
                },
            )
        }
    }
}

#[test]
fn check_whitelist() {
    use crate::*;
    use frame_support::traits::WhitelistedStorageKeys;
    use sp_core::hexdisplay::HexDisplay;
    use std::collections::HashSet;
    let whitelist: HashSet<String> = AllPalletsWithSystem::whitelisted_storage_keys()
        .iter()
        .map(|e| HexDisplay::from(&e.key).to_string())
        .collect();

    // Block Number
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac"));
    // Total Issuance
    assert!(whitelist.contains("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80"));
    // Execution Phase
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a"));
    // Event Count
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850"));
    // System Events
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7"));
}

#[test]
fn test_into_substrate_balance_valid() {
    // Valid conversion within u64 range
    let evm_balance: EvmBalance = 1_000_000_000_000_000_000u128.into(); // 1 TAO in EVM
    let expected_substrate_balance: SubstrateBalance = 1_000_000_000u128.into(); // 1 TAO in Substrate

    let result = SubtensorEvmBalanceConverter::into_substrate_balance(evm_balance);
    assert_eq!(result, Some(expected_substrate_balance));
}

#[test]
fn test_into_substrate_balance_large_value() {
    // Maximum valid balance for u64
    let evm_balance = EvmBalance::new(U256::from(u64::MAX) * U256::from(EVM_TO_SUBSTRATE_DECIMALS)); // Max u64 TAO in EVM
    let expected_substrate_balance = SubstrateBalance::new(U256::from(u64::MAX));

    let result = SubtensorEvmBalanceConverter::into_substrate_balance(evm_balance);
    assert_eq!(result, Some(expected_substrate_balance));
}

#[test]
fn test_into_substrate_balance_exceeds_u64() {
    // EVM balance that exceeds u64 after conversion
    let evm_balance = EvmBalance::new(
        (U256::from(u64::MAX) + U256::from(1)) * U256::from(EVM_TO_SUBSTRATE_DECIMALS),
    );

    let result = SubtensorEvmBalanceConverter::into_substrate_balance(evm_balance);
    assert_eq!(result, None); // Exceeds u64, should return None
}

#[test]
fn test_into_substrate_balance_precision_loss() {
    // EVM balance with precision loss
    let evm_balance = EvmBalance::new(U256::from(1_000_000_000_123_456_789u128)); // 1 TAO + extra precision in EVM
    let expected_substrate_balance = SubstrateBalance::new(U256::from(1_000_000_000u128)); // Truncated to 1 TAO in Substrate

    let result = SubtensorEvmBalanceConverter::into_substrate_balance(evm_balance);
    assert_eq!(result, Some(expected_substrate_balance));
}

#[test]
fn test_into_substrate_balance_zero_value() {
    // Zero balance should convert to zero
    let evm_balance = EvmBalance::new(U256::from(0));
    let expected_substrate_balance = SubstrateBalance::new(U256::from(0));

    let result = SubtensorEvmBalanceConverter::into_substrate_balance(evm_balance);
    assert_eq!(result, Some(expected_substrate_balance));
}

#[test]
fn test_into_evm_balance_valid() {
    // Valid conversion from Substrate to EVM
    let substrate_balance: SubstrateBalance = 1_000_000_000u128.into(); // 1 TAO in Substrate
    let expected_evm_balance = EvmBalance::new(U256::from(1_000_000_000_000_000_000u128)); // 1 TAO in EVM

    let result = SubtensorEvmBalanceConverter::into_evm_balance(substrate_balance);
    assert_eq!(result, Some(expected_evm_balance));
}

#[test]
fn test_into_evm_balance_overflow() {
    // Substrate balance larger than u64::MAX but valid within U256
    let substrate_balance = SubstrateBalance::new(U256::from(u64::MAX) + U256::from(1)); // Large balance
    let expected_evm_balance =
        EvmBalance::new(substrate_balance.into_u256() * U256::from(EVM_TO_SUBSTRATE_DECIMALS));

    let result = SubtensorEvmBalanceConverter::into_evm_balance(substrate_balance);
    assert_eq!(result, Some(expected_evm_balance)); // Should return the scaled value
}
