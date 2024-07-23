#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
use frame_support::derive_impl;
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::weights::constants::RocksDbWeight;
// use frame_support::weights::constants::WEIGHT_PER_SECOND;
use frame_support::weights::Weight;
use frame_support::{
    assert_ok, parameter_types,
    traits::{Everything, Hooks},
};
use frame_system as system;
use frame_system::{limits, EnsureNever, EnsureRoot, RawOrigin};
use pallet_collective::MemberCount;
use sp_core::{Get, H256, U256};
use sp_runtime::Perbill;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
        Triumvirate: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TriumvirateMembers: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
        Senate: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        SenateMembers: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>},
        SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>},
        Utility: pallet_utility::{Pallet, Call, Storage, Event},
    }
);

#[allow(dead_code)]
pub type SubtensorCall = pallet_subtensor::Call<Test>;

#[allow(dead_code)]
pub type SubtensorEvent = pallet_subtensor::Event<Test>;

#[allow(dead_code)]
pub type BalanceCall = pallet_balances::Call<Test>;

#[allow(dead_code)]
pub type TestRuntimeCall = frame_system::Call<Test>;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

#[allow(dead_code)]
pub type AccountId = U256;

// The address format for describing accounts.
#[allow(dead_code)]
pub type Address = AccountId;

// Balance of an account.
#[allow(dead_code)]
pub type Balance = u64;

// An index to a block.
#[allow(dead_code)]
pub type BlockNumber = u64;

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

    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type BaseCallFilter = Everything;
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
    pub const InitialMinAllowedWeights: u16 = 0;
    pub const InitialEmissionValue: u16 = 0;
    pub const InitialMaxWeightsLimit: u16 = u16::MAX;
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::with_sensible_defaults(
        Weight::from_parts(2_000_000_000_000, u64::MAX),
        Perbill::from_percent(75),
    );
    pub const ExistentialDeposit: Balance = 1;
    pub const TransactionByteFee: Balance = 100;
    pub const SDebug:u64 = 1;
    pub const InitialRho: u16 = 30;
    pub const InitialKappa: u16 = 32_767;
    pub const InitialTempo: u16 = 0;
    pub const SelfOwnership: u64 = 2;
    pub const InitialImmunityPeriod: u16 = 2;
    pub const InitialMaxAllowedUids: u16 = 2;
    pub const InitialBondsMovingAverage: u64 = 900_000;
    pub const InitialStakePruningMin: u16 = 0;
    pub const InitialFoundationDistribution: u64 = 0;
    pub const InitialDefaultTake: u16 = 11_796; // 18%, same as in production
    pub const InitialMinTake: u16 =5_898; // 9%;
    pub const InitialWeightsVersionKey: u16 = 0;
    pub const InitialServingRateLimit: u64 = 0; // No limit.
    pub const InitialTxRateLimit: u64 = 0; // Disable rate limit for testing
    pub const InitialTxDelegateTakeRateLimit: u64 = 1; // 1 block take rate limit for testing
    pub const InitialBurn: u64 = 0;
    pub const InitialMinBurn: u64 = 0;
    pub const InitialMaxBurn: u64 = 1_000_000_000;
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
    pub const InitialSenateRequiredStakePercentage: u64 = 2; // 2 percent of total stake
    pub const InitialNetworkImmunityPeriod: u64 = 7200 * 7;
    pub const InitialNetworkMinAllowedUids: u16 = 128;
    pub const InitialNetworkMinLockCost: u64 = 100_000_000_000;
    pub const InitialSubnetOwnerCut: u16 = 0; // 0%. 100% of rewards go to validators + miners.
    pub const InitialNetworkLockReductionInterval: u64 = 2; // 2 blocks.
    pub const InitialSubnetLimit: u16 = 10; // Max 10 subnets.
    pub const InitialNetworkRateLimit: u64 = 0;
    pub const InitialTargetStakesPerInterval: u16 = 2;
    pub const InitialKeySwapCost: u64 = 1_000_000_000;
    pub const InitialAlphaHigh: u16 = 58982; // Represents 0.9 as per the production default
    pub const InitialAlphaLow: u16 = 45875; // Represents 0.7 as per the production default
    pub const InitialLiquidAlphaOn: bool = false; // Default value for LiquidAlphaOn
    pub const InitialHotkeyEmissionTempo: u64 = 0; // Defaults to draining every block.
    pub const InitialNetworkMaxStake: u64 = 500_000_000_000_000; // 500,000 TAO

}

// Configure collective pallet for council
parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 100;
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

use pallet_subtensor::{CollectiveInterface, MemberManagement};
pub struct ManageSenateMembers;
impl MemberManagement<AccountId> for ManageSenateMembers {
    fn add_member(account: &AccountId) -> DispatchResultWithPostInfo {
        let who = *account;
        SenateMembers::add_member(RawOrigin::Root.into(), who)
    }

    fn remove_member(account: &AccountId) -> DispatchResultWithPostInfo {
        let who = *account;
        SenateMembers::remove_member(RawOrigin::Root.into(), who)
    }

    fn swap_member(rm: &AccountId, add: &AccountId) -> DispatchResultWithPostInfo {
        let remove = *rm;
        let add = *add;

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
        Senate::members().len() as u32
    }
}
impl Get<MemberCount> for GetSenateMemberCount {
    fn get() -> MemberCount {
        SenateMaxMembers::get()
    }
}

pub struct TriumvirateVotes;
impl CollectiveInterface<AccountId, H256, u32> for TriumvirateVotes {
    fn remove_votes(hotkey: &AccountId) -> Result<bool, sp_runtime::DispatchError> {
        Triumvirate::remove_votes(hotkey)
    }

    fn add_vote(
        hotkey: &AccountId,
        proposal: H256,
        index: u32,
        approve: bool,
    ) -> Result<bool, sp_runtime::DispatchError> {
        Triumvirate::do_vote(*hotkey, proposal, index, approve)
    }
}

// We call pallet_collective TriumvirateCollective
type TriumvirateCollective = pallet_collective::Instance1;
impl pallet_collective::Config<TriumvirateCollective> for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = GetSenateMemberCount;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Test>;
    type SetMembersOrigin = EnsureNever<AccountId>;
    type CanPropose = CanProposeToTriumvirate;
    type CanVote = CanVoteToTriumvirate;
    type GetVotingMembers = GetSenateMemberCount;
}

// We call council members Triumvirate
type TriumvirateMembership = pallet_membership::Instance1;
impl pallet_membership::Config<TriumvirateMembership> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = Triumvirate;
    type MembershipChanged = Triumvirate;
    type MaxMembers = CouncilMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Test>;
}

// This is a dummy collective instance for managing senate members
// Probably not the best solution, but fastest implementation
type SenateCollective = pallet_collective::Instance2;
impl pallet_collective::Config<SenateCollective> for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = SenateMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Test>;
    type SetMembersOrigin = EnsureNever<AccountId>;
    type CanPropose = ();
    type CanVote = ();
    type GetVotingMembers = ();
}

// We call our top K delegates membership Senate
type SenateMembership = pallet_membership::Instance2;
impl pallet_membership::Config<SenateMembership> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = Senate;
    type MembershipChanged = Senate;
    type MaxMembers = SenateMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Test>;
}

impl pallet_subtensor::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InitialIssuance = InitialIssuance;
    type SudoRuntimeCall = TestRuntimeCall;
    type CouncilOrigin = frame_system::EnsureSigned<AccountId>;
    type SenateMembers = ManageSenateMembers;
    type TriumvirateInterface = TriumvirateVotes;

    type InitialMinAllowedWeights = InitialMinAllowedWeights;
    type InitialEmissionValue = InitialEmissionValue;
    type InitialMaxWeightsLimit = InitialMaxWeightsLimit;
    type InitialTempo = InitialTempo;
    type InitialDifficulty = InitialDifficulty;
    type InitialAdjustmentInterval = InitialAdjustmentInterval;
    type InitialAdjustmentAlpha = InitialAdjustmentAlpha;
    type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
    type InitialRho = InitialRho;
    type InitialKappa = InitialKappa;
    type InitialMaxAllowedUids = InitialMaxAllowedUids;
    type InitialValidatorPruneLen = InitialValidatorPruneLen;
    type InitialScalingLawPower = InitialScalingLawPower;
    type InitialImmunityPeriod = InitialImmunityPeriod;
    type InitialActivityCutoff = InitialActivityCutoff;
    type InitialMaxRegistrationsPerBlock = InitialMaxRegistrationsPerBlock;
    type InitialPruningScore = InitialPruningScore;
    type InitialBondsMovingAverage = InitialBondsMovingAverage;
    type InitialMaxAllowedValidators = InitialMaxAllowedValidators;
    type InitialDefaultTake = InitialDefaultTake;
    type InitialMinTake = InitialMinTake;
    type InitialWeightsVersionKey = InitialWeightsVersionKey;
    type InitialMaxDifficulty = InitialMaxDifficulty;
    type InitialMinDifficulty = InitialMinDifficulty;
    type InitialServingRateLimit = InitialServingRateLimit;
    type InitialTxRateLimit = InitialTxRateLimit;
    type InitialTxDelegateTakeRateLimit = InitialTxDelegateTakeRateLimit;
    type InitialBurn = InitialBurn;
    type InitialMaxBurn = InitialMaxBurn;
    type InitialMinBurn = InitialMinBurn;
    type InitialRAORecycledForRegistration = InitialRAORecycledForRegistration;
    type InitialSenateRequiredStakePercentage = InitialSenateRequiredStakePercentage;
    type InitialNetworkImmunityPeriod = InitialNetworkImmunityPeriod;
    type InitialNetworkMinAllowedUids = InitialNetworkMinAllowedUids;
    type InitialNetworkMinLockCost = InitialNetworkMinLockCost;
    type InitialSubnetOwnerCut = InitialSubnetOwnerCut;
    type InitialNetworkLockReductionInterval = InitialNetworkLockReductionInterval;
    type InitialSubnetLimit = InitialSubnetLimit;
    type InitialNetworkRateLimit = InitialNetworkRateLimit;
    type InitialTargetStakesPerInterval = InitialTargetStakesPerInterval;
    type KeySwapCost = InitialKeySwapCost;
    type AlphaHigh = InitialAlphaHigh;
    type AlphaLow = InitialAlphaLow;
    type LiquidAlphaOn = InitialLiquidAlphaOn;
    type InitialHotkeyEmissionTempo = InitialHotkeyEmissionTempo;
    type InitialNetworkMaxStake = InitialNetworkMaxStake;
}

impl pallet_utility::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Test>;
}

#[allow(dead_code)]
// Build genesis storage according to the mock runtime.
pub fn new_test_ext(block_number: BlockNumber) -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(block_number));
    ext
}

#[allow(dead_code)]
pub fn test_ext_with_balances(balances: Vec<(U256, u128)>) -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: balances
            .iter()
            .map(|(a, b)| (*a, *b as u64))
            .collect::<Vec<(U256, u64)>>(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

#[allow(dead_code)]
pub(crate) fn step_block(n: u16) {
    for _ in 0..n {
        SubtensorModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SubtensorModule::on_initialize(System::block_number());
    }
}

#[allow(dead_code)]
pub(crate) fn run_to_block(n: u64) {
    while System::block_number() < n {
        SubtensorModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SubtensorModule::on_initialize(System::block_number());
    }
}

/// Increments current block by `1`, running all hooks associated with doing so, and asserts
/// that the block number was in fact incremented.
///
/// Returns the new block number.
#[allow(dead_code)]
#[cfg(test)]
pub(crate) fn next_block() -> u64 {
    let mut block = System::block_number();
    block += 1;
    run_to_block(block);
    assert_eq!(System::block_number(), block);
    block
}

#[allow(dead_code)]
pub fn register_ok_neuron(
    netuid: u16,
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
        "Register ok neuron: netuid: {:?}, coldkey: {:?}, hotkey: {:?}",
        netuid,
        hotkey_account_id,
        coldkey_account_id
    );
}

#[allow(dead_code)]
pub fn add_network(netuid: u16, tempo: u16, _modality: u16) {
    SubtensorModule::init_new_network(netuid, tempo);
    SubtensorModule::set_network_registration_allowed(netuid, true);
    SubtensorModule::set_network_pow_registration_allowed(netuid, true);
}

pub const NETUID1: u16 = 1;
pub const TEMPO: u64 = 10;
#[allow(dead_code)]
pub const HOTKEY_TEMPO: u64 = 1000;
pub const COLDKEY: u16 = 1;
pub const NOMINATOR1: u16 = 2;
#[allow(dead_code)]
pub const NOMINATOR2: u16 = 3;
pub const VALIDATOR1: u16 = 4;
pub const VALIDATOR2: u16 = 5;
pub const CHILDKEY: u16 = 6;
pub const STAKE: u64 = 1_000_000_000;

#[allow(dead_code)]
pub fn setup_childkey_test(validators: u16) {
    let netuid: u16 = NETUID1;
    let tempo = TEMPO;
    let coldkey = U256::from(COLDKEY);
    let nominator = U256::from(NOMINATOR1);
    let validator1 = U256::from(VALIDATOR1);
    let validator2 = U256::from(VALIDATOR2);
    let childkey = U256::from(CHILDKEY);
    let stake: u64 = STAKE;

    // Remove limitations that are not needed for this test
    SubtensorModule::set_max_registrations_per_block(netuid, 4);
    SubtensorModule::set_max_allowed_uids(netuid, 2);
    SubtensorModule::set_target_stakes_per_interval(10);
    pallet_subtensor::WeightsSetRateLimit::<Test>::insert(netuid, 0);

    // Add balances
    SubtensorModule::add_balance_to_coldkey_account(
        &coldkey,
        100 * stake + ExistentialDeposit::get(),
    );
    SubtensorModule::add_balance_to_coldkey_account(
        &nominator,
        100 * stake + ExistentialDeposit::get(),
    );

    // Register a subnet and neurons to the new network.
    add_network(netuid, tempo as u16, 0);
    register_ok_neuron(netuid, validator1, coldkey, 124124);
    register_ok_neuron(netuid, validator2, coldkey, 456456);
    register_ok_neuron(netuid, childkey, coldkey, 987987);
    assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey, &validator1));
    assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey, &validator2));
    assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey, &childkey));

    // Add validator permits
    pallet_subtensor::MaxAllowedValidators::<Test>::insert(NETUID1, validators);
    for uid in 0..validators {
        SubtensorModule::set_validator_permit_for_uid(netuid, uid, true);
    }
}

#[allow(dead_code)]
pub fn add_stake(coldkey: u16, hotkey: u16, stake: u64) {
    assert_ok!(SubtensorModule::add_stake(
        RuntimeOrigin::signed(U256::from(coldkey)),
        U256::from(hotkey),
        stake
    ));
}

#[allow(dead_code)]
pub fn become_delegate(hotkey: u16) {
    let hotkey = U256::from(hotkey);
    assert_ok!(SubtensorModule::do_become_delegate(
        RuntimeOrigin::signed(U256::from(COLDKEY)),
        hotkey,
        SubtensorModule::get_default_take()
    ));
    assert!(SubtensorModule::hotkey_is_delegate(&hotkey));
}

#[allow(dead_code)]
pub fn set_weights(hotkey: u16, weights: Vec<u16>) {
    let hotkey = U256::from(hotkey);
    let neuron_count = weights.len();
    let uid_vec: Vec<u16> = (0..neuron_count)
        .into_iter()
        .map(|uid| uid as u16)
        .collect();
    assert_ok!(SubtensorModule::do_set_weights(
        RuntimeOrigin::signed(hotkey),
        NETUID1,
        uid_vec,
        weights,
        0
    ));
}

#[allow(dead_code)]
pub fn set_children(hotkey: u16, child_vec: Vec<(u64, u16)>) {
    let hotkey = U256::from(hotkey);
    let child_vec_u256 = child_vec
        .iter()
        .map(|(weight, child_id)| (*weight, U256::from(*child_id)))
        .collect();
    assert_ok!(SubtensorModule::do_set_children(
        RuntimeOrigin::signed(U256::from(COLDKEY)),
        hotkey,
        NETUID1,
        child_vec_u256
    ));
}

#[allow(dead_code)]
pub fn get_stake(coldkey: u16, hotkey: u16) -> u64 {
    let coldkey = U256::from(coldkey);
    let hotkey = U256::from(hotkey);
    SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey, &hotkey)
}

#[allow(dead_code)]
pub fn get_balance(coldkey: u16) -> u64 {
    let coldkey = U256::from(coldkey);
    pallet_balances::Account::<Test>::get(coldkey).free
}

#[allow(dead_code)]
pub fn helper_wait_until_end_of_epoch(netuid: u16, timeout: u64) {
    let mut block_count = 0;
    loop {
        step_block(1);

        let block = SubtensorModule::get_current_block_as_u64();
        if SubtensorModule::should_run_epoch(netuid, block) {
            break;
        }

        // Shouldn't be more than X blocks
        block_count += 1;
        assert!(block_count < timeout);
    }
}

#[allow(dead_code)]
pub fn helper_wait_x_blocks_before_hotkey_drained(
    hotkey: u16,
    tempo: u64,
    blocks_prior_to_drain: u64,
    timeout: u64,
) {
    let hotkey = U256::from(hotkey);

    // Otherwise wait until end of hotkey draining tempo
    let mut block_count = 0;
    loop {
        step_block(1);

        let block = SubtensorModule::get_current_block_as_u64() + blocks_prior_to_drain;
        if SubtensorModule::should_drain_hotkey(&hotkey, block, tempo as u64) {
            break;
        }

        // Shouldn't be more than X blocks
        block_count += 1;
        assert!(block_count < timeout);
    }
}

#[allow(dead_code)]
pub fn helper_wait_hotkey_drained(hotkey: u16, tempo: u64, timeout: u64) {
    let hotkey_u256 = U256::from(hotkey);

    // If hotkey emission is already drained, don't wait
    if pallet_subtensor::PendingdHotkeyEmission::<Test>::get(hotkey_u256) == 0 {
        return;
    }

    helper_wait_x_blocks_before_hotkey_drained(hotkey, tempo, 0, timeout);
}
