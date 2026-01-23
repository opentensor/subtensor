#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::unwrap_used
)]

use core::num::NonZeroU64;

use frame_support::dispatch::DispatchResult;
use frame_support::traits::{Contains, Everything, InherentBuilder, InsideBoth};
use frame_support::weights::Weight;
use frame_support::weights::constants::RocksDbWeight;
use frame_support::{PalletId, derive_impl};
use frame_support::{assert_ok, parameter_types, traits::PrivilegeCmp};
use frame_system as system;
use frame_system::{EnsureRoot, RawOrigin, limits, offchain::CreateTransactionBase};
use pallet_contracts::HoldReason as ContractsHoldReason;
use pallet_subtensor::*;
use pallet_subtensor_proxy as pallet_proxy;
use pallet_subtensor_utility as pallet_utility;
use rate_limiting_interface::{RateLimitingInterface, TryIntoRateLimitTarget};
use sp_core::{ConstU64, H256, U256, offchain::KeyTypeId};
use sp_runtime::Perbill;
use sp_runtime::{
    BuildStorage, Percent,
    traits::{BlakeTwo256, Convert, IdentityLookup},
};
use sp_std::{cell::RefCell, cmp::Ordering, sync::OnceLock};
use subtensor_runtime_common::{
    AlphaCurrency, NetUid, TaoCurrency, rate_limiting::RateLimitUsageKey,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>} = 1,
        Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>} = 2,
        SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>} = 7,
        Utility: pallet_utility::{Pallet, Call, Storage, Event} = 8,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 9,
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 10,
        Drand: pallet_drand::{Pallet, Call, Storage, Event<T>} = 11,
        Swap: pallet_subtensor_swap::{Pallet, Call, Storage, Event<T>} = 12,
        Crowdloan: pallet_crowdloan::{Pallet, Call, Storage, Event<T>} = 13,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage} = 14,
        Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>} = 15,
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 16,
    }
);

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"test");

#[allow(dead_code)]
pub type TestRuntimeCall = frame_system::Call<Test>;

#[allow(dead_code)]
pub type AccountId = U256;

// Balance of an account.
#[allow(dead_code)]
pub type Balance = u64;

// An index to a block.
#[allow(dead_code)]
pub type BlockNumber = u64;

pub struct DummyContractsRandomness;

impl frame_support::traits::Randomness<H256, BlockNumber> for DummyContractsRandomness {
    fn random(_subject: &[u8]) -> (H256, BlockNumber) {
        (H256::zero(), 0)
    }
}

pub struct WeightToBalance;

impl Convert<Weight, Balance> for WeightToBalance {
    fn convert(weight: Weight) -> Balance {
        weight.ref_time()
    }
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxLocks = ();
    type WeightInfo = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
    type RuntimeHoldReason = ContractsHoldReason;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Test {
    type MinimumPeriod = ConstU64<1>;
}

#[derive_impl(pallet_contracts::config_preludes::TestDefaultConfig)]
impl pallet_contracts::Config for Test {
    type Time = Timestamp;
    type Randomness = DummyContractsRandomness;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = ContractsHoldReason;
    type RuntimeCall = RuntimeCall;
    type CallFilter = Everything;
    type WeightPrice = WeightToBalance;
    type WeightInfo = ();
    type ChainExtension = crate::SubtensorChainExtension<Self>;
    type Schedule = ContractsSchedule;
    type CallStack = [pallet_contracts::Frame<Self>; 5];
    type DepositPerByte = ContractsDepositPerByte;
    type DepositPerItem = ContractsDepositPerItem;
    type DefaultDepositLimit = ContractsDefaultDepositLimit;
    type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
    type UnsafeUnstableInterface = ContractsUnstableInterface;
    type UploadOrigin = frame_system::EnsureSigned<AccountId>;
    type InstantiateOrigin = frame_system::EnsureSigned<AccountId>;
    type CodeHashLockupDepositPercent = ContractsCodeHashLockupDepositPercent;
    type MaxDelegateDependencies = ContractsMaxDelegateDependencies;
    type MaxCodeLen = ContractsMaxCodeLen;
    type MaxStorageKeyLen = ContractsMaxStorageKeyLen;
    type MaxTransientStorageSize = ContractsMaxTransientStorageSize;
    type MaxDebugBufferLen = ContractsMaxDebugBufferLen;
    type Migrations = ();
    type Debug = ();
    type Environment = ();
    type ApiVersion = ();
    type Xcm = ();
}

impl frame_support::traits::InstanceFilter<RuntimeCall> for subtensor_runtime_common::ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            subtensor_runtime_common::ProxyType::Any => true,
            subtensor_runtime_common::ProxyType::Staking => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake { .. })
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
            ),
            _ => false,
        }
    }

    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (subtensor_runtime_common::ProxyType::Any, _) => true,
            _ => self == o,
        }
    }
}

impl pallet_proxy::Config for Test {
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = subtensor_runtime_common::ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = ();
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
    type BlockNumberProvider = System;
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

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type BaseCallFilter = InsideBoth<Everything, NoNestingCallFilter>;
    type BlockWeights = BlockWeights;
    type BlockLength = ();
    type DbWeight = RocksDbWeight;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = U256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Nonce = u64;
    type Block = Block;
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

parameter_types! {
    pub ContractsSchedule: pallet_contracts::Schedule<Test> = Default::default();
    pub const ContractsDepositPerByte: Balance = 1;
    pub const ContractsDepositPerItem: Balance = 10;
    pub const ContractsDefaultDepositLimit: Balance = 1_000_000_000;
    pub const ContractsCodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
    pub const ContractsMaxDelegateDependencies: u32 = 32;
    pub const ContractsMaxCodeLen: u32 = 120_000;
    pub const ContractsMaxStorageKeyLen: u32 = 256;
    pub const ContractsMaxTransientStorageSize: u32 = 1024 * 1024;
    pub const ContractsMaxDebugBufferLen: u32 = 2 * 1024 * 1024;
    pub const ContractsUnstableInterface: bool = true;
}

parameter_types! {
    pub const ProxyDepositBase: Balance = 1;
    pub const ProxyDepositFactor: Balance = 1;
    pub const MaxProxies: u32 = 32;
    pub const MaxPending: u32 = 32;
    pub const AnnouncementDepositBase: Balance = 1;
    pub const AnnouncementDepositFactor: Balance = 1;
}

parameter_types! {
    pub const InitialMinAllowedWeights: u16 = 0;
    pub const InitialEmissionValue: u16 = 0;
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::with_sensible_defaults(
        Weight::from_parts(2_000_000_000_000, u64::MAX),
        Perbill::from_percent(75),
    );
    pub const ExistentialDeposit: Balance = 1;
    pub const TransactionByteFee: Balance = 100;
    pub const SDebug:u64 = 1;
    pub const InitialRho: u16 = 30;
    pub const InitialAlphaSigmoidSteepness: i16 = 1000;
    pub const InitialKappa: u16 = 32_767;
    pub const InitialTempo: u16 = 360;
    pub const SelfOwnership: u64 = 2;
    pub const InitialImmunityPeriod: u16 = 2;
    pub const InitialMinAllowedUids: u16 = 2;
    pub const InitialMaxAllowedUids: u16 = 4;
    pub const InitialBondsMovingAverage: u64 = 900_000;
    pub const InitialBondsPenalty:u16 = u16::MAX;
    pub const InitialBondsResetOn: bool = false;
    pub const InitialStakePruningMin: u16 = 0;
    pub const InitialFoundationDistribution: u64 = 0;
    pub const InitialDefaultDelegateTake: u16 = 11_796; // 18%, same as in production
    pub const InitialMinDelegateTake: u16 = 5_898; // 9%;
    pub const InitialDefaultChildKeyTake: u16 = 0 ;// 0 %
    pub const InitialMinChildKeyTake: u16 = 0; // 0 %;
    pub const InitialMaxChildKeyTake: u16 = 11_796; // 18 %;
    pub const InitialWeightsVersionKey: u16 = 0;
    pub const InitialTxChildKeyTakeRateLimit: u64 = 1; // 1 block take rate limit for testing
    pub const InitialBurn: u64 = 0;
    pub const InitialMinBurn: u64 = 500_000;
    pub const InitialMaxBurn: u64 = 1_000_000_000;
    pub const MinBurnUpperBound: TaoCurrency = TaoCurrency::new(1_000_000_000); // 1 TAO
    pub const MaxBurnLowerBound: TaoCurrency = TaoCurrency::new(100_000_000); // 0.1 TAO
    pub const InitialValidatorPruneLen: u64 = 0;
    pub const InitialScalingLawPower: u16 = 50;
    pub const InitialMaxAllowedValidators: u16 = 100;
    pub const InitialIssuance: u64 = 0;
    pub const InitialDifficulty: u64 = 10000;
    pub const InitialActivityCutoff: u16 = 5000;
    pub const InitialAdjustmentInterval: u16 = 100;
    pub const InitialAdjustmentAlpha: u64 = 0; // no weight to previous value.
    pub const InitialMaxRegistrationsPerBlock: u16 = 3;
    pub const InitialTargetRegistrationsPerInterval: u16 = 2;
    pub const InitialPruningScore : u16 = u16::MAX;
    pub const InitialRegistrationRequirement: u16 = u16::MAX; // Top 100%
    pub const InitialMinDifficulty: u64 = 1;
    pub const InitialMaxDifficulty: u64 = u64::MAX;
    pub const InitialRAORecycledForRegistration: u64 = 0;
    pub const InitialNetworkImmunityPeriod: u64 = 1_296_000;
    pub const InitialNetworkMinLockCost: u64 = 100_000_000_000;
    pub const InitialSubnetOwnerCut: u16 = 0; // 0%. 100% of rewards go to validators + miners.
    pub const InitialNetworkLockReductionInterval: u64 = 2; // 2 blocks.
    pub const InitialKeySwapCost: u64 = 1_000_000_000;
    pub const InitialAlphaHigh: u16 = 58982; // Represents 0.9 as per the production default
    pub const InitialAlphaLow: u16 = 45875; // Represents 0.7 as per the production default
    pub const InitialLiquidAlphaOn: bool = false; // Default value for LiquidAlphaOn
    pub const InitialYuma3On: bool = false; // Default value for Yuma3On
    // pub const InitialNetworkMaxStake: u64 = u64::MAX; // (DEPRECATED)
    pub const InitialColdkeySwapScheduleDuration: u64 =  5 * 24 * 60 * 60 / 12; // Default as 5 days
    pub const InitialColdkeySwapRescheduleDuration: u64 = 24 * 60 * 60 / 12; // Default as 1 day
    pub const InitialDissolveNetworkScheduleDuration: u64 =  5 * 24 * 60 * 60 / 12; // Default as 5 days
    pub const InitialTaoWeight: u64 = 0; // 100% global weight.
    pub const InitialEmaPriceHalvingPeriod: u64 = 201_600_u64; // 4 weeks
    pub const InitialStartCallDelay: u64 =  7 * 24 * 60 * 60 / 12; // Default as 7 days
    pub const InitialKeySwapOnSubnetCost: u64 = 10_000_000;
    pub const HotkeySwapOnSubnetInterval: u64 = 15; // 15 block, should be bigger than subnet number, then trigger clean up for all subnets
    pub const MaxContributorsPerLeaseToRemove: u32 = 3;
    pub const LeaseDividendsDistributionInterval: u32 = 100;
    pub const MaxImmuneUidsPercentage: Percent = Percent::from_percent(80);
    pub const EvmKeyAssociateRateLimit: u64 = 10;
}

impl pallet_subtensor::Config for Test {
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type InitialIssuance = InitialIssuance;
    type SudoRuntimeCall = TestRuntimeCall;
    type Scheduler = Scheduler;
    type InitialMinAllowedWeights = InitialMinAllowedWeights;
    type InitialEmissionValue = InitialEmissionValue;
    type InitialTempo = InitialTempo;
    type InitialDifficulty = InitialDifficulty;
    type InitialAdjustmentInterval = InitialAdjustmentInterval;
    type InitialAdjustmentAlpha = InitialAdjustmentAlpha;
    type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
    type InitialRho = InitialRho;
    type InitialAlphaSigmoidSteepness = InitialAlphaSigmoidSteepness;
    type InitialKappa = InitialKappa;
    type InitialMinAllowedUids = InitialMinAllowedUids;
    type InitialMaxAllowedUids = InitialMaxAllowedUids;
    type InitialValidatorPruneLen = InitialValidatorPruneLen;
    type InitialScalingLawPower = InitialScalingLawPower;
    type InitialImmunityPeriod = InitialImmunityPeriod;
    type InitialActivityCutoff = InitialActivityCutoff;
    type InitialMaxRegistrationsPerBlock = InitialMaxRegistrationsPerBlock;
    type InitialPruningScore = InitialPruningScore;
    type InitialBondsMovingAverage = InitialBondsMovingAverage;
    type InitialBondsPenalty = InitialBondsPenalty;
    type InitialBondsResetOn = InitialBondsResetOn;
    type InitialMaxAllowedValidators = InitialMaxAllowedValidators;
    type InitialDefaultDelegateTake = InitialDefaultDelegateTake;
    type InitialMinDelegateTake = InitialMinDelegateTake;
    type InitialDefaultChildKeyTake = InitialDefaultChildKeyTake;
    type InitialMinChildKeyTake = InitialMinChildKeyTake;
    type InitialMaxChildKeyTake = InitialMaxChildKeyTake;
    type InitialTxChildKeyTakeRateLimit = InitialTxChildKeyTakeRateLimit;
    type InitialWeightsVersionKey = InitialWeightsVersionKey;
    type InitialMaxDifficulty = InitialMaxDifficulty;
    type InitialMinDifficulty = InitialMinDifficulty;
    type InitialBurn = InitialBurn;
    type InitialMaxBurn = InitialMaxBurn;
    type InitialMinBurn = InitialMinBurn;
    type MinBurnUpperBound = MinBurnUpperBound;
    type MaxBurnLowerBound = MaxBurnLowerBound;
    type InitialRAORecycledForRegistration = InitialRAORecycledForRegistration;
    type InitialNetworkImmunityPeriod = InitialNetworkImmunityPeriod;
    type InitialNetworkMinLockCost = InitialNetworkMinLockCost;
    type InitialSubnetOwnerCut = InitialSubnetOwnerCut;
    type InitialNetworkLockReductionInterval = InitialNetworkLockReductionInterval;
    type KeySwapCost = InitialKeySwapCost;
    type AlphaHigh = InitialAlphaHigh;
    type AlphaLow = InitialAlphaLow;
    type LiquidAlphaOn = InitialLiquidAlphaOn;
    type Yuma3On = InitialYuma3On;
    type Preimages = Preimage;
    type InitialColdkeySwapScheduleDuration = InitialColdkeySwapScheduleDuration;
    type InitialColdkeySwapRescheduleDuration = InitialColdkeySwapRescheduleDuration;
    type InitialDissolveNetworkScheduleDuration = InitialDissolveNetworkScheduleDuration;
    type InitialTaoWeight = InitialTaoWeight;
    type InitialEmaPriceHalvingPeriod = InitialEmaPriceHalvingPeriod;
    type InitialStartCallDelay = InitialStartCallDelay;
    type SwapInterface = pallet_subtensor_swap::Pallet<Self>;
    type KeySwapOnSubnetCost = InitialKeySwapOnSubnetCost;
    type HotkeySwapOnSubnetInterval = HotkeySwapOnSubnetInterval;
    type ProxyInterface = FakeProxier;
    type LeaseDividendsDistributionInterval = LeaseDividendsDistributionInterval;
    type GetCommitments = ();
    type MaxImmuneUidsPercentage = MaxImmuneUidsPercentage;
    type CommitmentsInterface = CommitmentsI;
    type RateLimiting = NoRateLimiting;
    type EvmKeyAssociateRateLimit = EvmKeyAssociateRateLimit;
}

// Swap-related parameter types
parameter_types! {
    pub const SwapProtocolId: PalletId = PalletId(*b"ten/swap");
    pub const SwapMaxFeeRate: u16 = 10000; // 15.26%
    pub const SwapMaxPositions: u32 = 100;
    pub const SwapMinimumLiquidity: u64 = 1_000;
    pub const SwapMinimumReserve: NonZeroU64 = NonZeroU64::new(100).unwrap();
}

impl pallet_subtensor_swap::Config for Test {
    type SubnetInfo = SubtensorModule;
    type BalanceOps = SubtensorModule;
    type ProtocolId = SwapProtocolId;
    type TaoReserve = TaoCurrencyReserve<Self>;
    type AlphaReserve = AlphaCurrencyReserve<Self>;
    type MaxFeeRate = SwapMaxFeeRate;
    type MaxPositions = SwapMaxPositions;
    type MinimumLiquidity = SwapMinimumLiquidity;
    type MinimumReserve = SwapMinimumReserve;
    type WeightInfo = ();
}

pub struct OriginPrivilegeCmp;

impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
    fn cmp_privilege(_left: &OriginCaller, _right: &OriginCaller) -> Option<Ordering> {
        Some(Ordering::Less)
    }
}

pub struct CommitmentsI;
impl CommitmentsInterface for CommitmentsI {
    fn purge_netuid(_netuid: NetUid) {}
}

pub struct NoRateLimiting;

impl RateLimitingInterface for NoRateLimiting {
    type GroupId = subtensor_runtime_common::rate_limiting::GroupId;
    type CallMetadata = RuntimeCall;
    type Limit = BlockNumber;
    type Scope = subtensor_runtime_common::NetUid;
    type UsageKey = RateLimitUsageKey<AccountId>;

    fn rate_limit<TargetArg>(_target: TargetArg, _scope: Option<Self::Scope>) -> Option<Self::Limit>
    where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>,
    {
        None
    }

    fn last_seen<TargetArg>(
        _target: TargetArg,
        _usage_key: Option<Self::UsageKey>,
    ) -> Option<Self::Limit>
    where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>,
    {
        None
    }

    fn set_last_seen<TargetArg>(
        _target: TargetArg,
        _usage_key: Option<Self::UsageKey>,
        _block: Option<Self::Limit>,
    ) where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>,
    {
    }
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Test>;
    type OriginPrivilegeCmp = OriginPrivilegeCmp;
    type Preimages = Preimage;
    type BlockNumberProvider = System;
}

impl pallet_utility::Config for Test {
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Test>;
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = 1;
    pub const PreimageByteDeposit: Balance = 1;
}

impl pallet_preimage::Config for Test {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Test>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Consideration = ();
}

thread_local! {
    pub static PROXIES: RefCell<FakeProxier> = const { RefCell::new(FakeProxier(vec![])) };
}

pub struct FakeProxier(pub Vec<(U256, U256)>);

impl ProxyInterface<U256> for FakeProxier {
    fn add_lease_beneficiary_proxy(beneficiary: &AccountId, lease: &AccountId) -> DispatchResult {
        PROXIES.with_borrow_mut(|proxies| {
            proxies.0.push((*beneficiary, *lease));
        });
        Ok(())
    }

    fn remove_lease_beneficiary_proxy(
        beneficiary: &AccountId,
        lease: &AccountId,
    ) -> DispatchResult {
        PROXIES.with_borrow_mut(|proxies| {
            proxies.0.retain(|(b, l)| b != beneficiary && l != lease);
        });
        Ok(())
    }
}

parameter_types! {
    pub const CrowdloanPalletId: PalletId = PalletId(*b"bt/cloan");
    pub const MinimumDeposit: u64 = 50;
    pub const AbsoluteMinimumContribution: u64 = 10;
    pub const MinimumBlockDuration: u64 = 20;
    pub const MaximumBlockDuration: u64 = 100;
    pub const RefundContributorsLimit: u32 = 5;
    pub const MaxContributors: u32 = 10;
}

impl pallet_crowdloan::Config for Test {
    type PalletId = CrowdloanPalletId;
    type Currency = Balances;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_crowdloan::weights::SubstrateWeight<Test>;
    type Preimages = Preimage;
    type MinimumDeposit = MinimumDeposit;
    type AbsoluteMinimumContribution = AbsoluteMinimumContribution;
    type MinimumBlockDuration = MinimumBlockDuration;
    type MaximumBlockDuration = MaximumBlockDuration;
    type RefundContributorsLimit = RefundContributorsLimit;
    type MaxContributors = MaxContributors;
}

mod test_crypto {
    use super::KEY_TYPE;
    use sp_core::{
        U256,
        sr25519::{Public as Sr25519Public, Signature as Sr25519Signature},
    };
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::IdentifyAccount,
    };

    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    impl frame_system::offchain::AppCrypto<Public, Signature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = Sr25519Signature;
        type GenericPublic = Sr25519Public;
    }

    impl IdentifyAccount for Public {
        type AccountId = U256;

        fn into_account(self) -> U256 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(self.as_ref());
            U256::from_big_endian(&bytes)
        }
    }
}

pub type TestAuthId = test_crypto::TestAuthId;

impl pallet_drand::Config for Test {
    type AuthorityId = TestAuthId;
    type Verifier = pallet_drand::verifier::QuicknetVerifier;
    type UnsignedPriority = ConstU64<{ 1 << 20 }>;
    type HttpFetchTimeout = ConstU64<1_000>;
}

impl frame_system::offchain::SigningTypes for Test {
    type Public = test_crypto::Public;
    type Signature = test_crypto::Signature;
}

pub type UncheckedExtrinsic = sp_runtime::testing::TestXt<RuntimeCall, ()>;

impl<LocalCall> frame_system::offchain::CreateTransactionBase<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    type Extrinsic = UncheckedExtrinsic;
    type RuntimeCall = RuntimeCall;
}

impl<LocalCall> frame_system::offchain::CreateInherent<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    fn create_bare(call: Self::RuntimeCall) -> Self::Extrinsic {
        UncheckedExtrinsic::new_inherent(call)
    }
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    fn create_signed_transaction<
        C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>,
    >(
        call: <Self as CreateTransactionBase<LocalCall>>::RuntimeCall,
        _public: Self::Public,
        _account: Self::AccountId,
        nonce: Self::Nonce,
    ) -> Option<Self::Extrinsic> {
        Some(UncheckedExtrinsic::new_signed(call, nonce.into(), (), ()))
    }
}

static TEST_LOGS_INIT: OnceLock<()> = OnceLock::new();

pub fn init_logs_for_tests() {
    if TEST_LOGS_INIT.get().is_some() {
        return;
    }
    let _ = TEST_LOGS_INIT.set(());
}

#[allow(dead_code)]
// Build genesis storage according to the mock runtime.
pub fn new_test_ext(block_number: BlockNumber) -> sp_io::TestExternalities {
    init_logs_for_tests();
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(block_number));
    ext
}

#[allow(dead_code)]
pub fn register_ok_neuron(
    netuid: NetUid,
    hotkey_account_id: U256,
    coldkey_account_id: U256,
    start_nonce: u64,
) {
    let block_number: u64 = SubtensorModule::get_current_block_as_u64();
    let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
        netuid,
        block_number,
        start_nonce,
        &hotkey_account_id,
    );
    let result = SubtensorModule::register(
        <<Test as frame_system::Config>::RuntimeOrigin>::signed(hotkey_account_id),
        netuid,
        block_number,
        nonce,
        work,
        hotkey_account_id,
        coldkey_account_id,
    );
    assert_ok!(result);
    log::info!(
        "Register ok neuron: netuid: {netuid:?}, coldkey: {hotkey_account_id:?}, hotkey: {coldkey_account_id:?}"
    );
}

#[allow(dead_code)]
pub fn add_dynamic_network(hotkey: &U256, coldkey: &U256) -> NetUid {
    let netuid = SubtensorModule::get_next_netuid();
    let lock_cost = SubtensorModule::get_network_lock_cost();
    SubtensorModule::add_balance_to_coldkey_account(coldkey, lock_cost.into());

    assert_ok!(SubtensorModule::register_network(
        RawOrigin::Signed(*coldkey).into(),
        *hotkey
    ));
    NetworkRegistrationAllowed::<Test>::insert(netuid, true);
    NetworkPowRegistrationAllowed::<Test>::insert(netuid, true);
    FirstEmissionBlockNumber::<Test>::insert(netuid, 0);
    SubtokenEnabled::<Test>::insert(netuid, true);
    netuid
}

#[allow(dead_code)]
pub(crate) fn setup_reserves(netuid: NetUid, tao: TaoCurrency, alpha: AlphaCurrency) {
    SubnetTAO::<Test>::set(netuid, tao);
    SubnetAlphaIn::<Test>::set(netuid, alpha);
}
