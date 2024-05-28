use frame_support::traits::Hash;
use frame_support::{
    assert_ok, parameter_types,
    traits::{Everything, Hooks},
    weights,
};
use frame_system as system;
use frame_system::{limits, EnsureNever, EnsureRoot, RawOrigin};
use sp_core::{Get, H256, U256};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, DispatchResult,
};

use pallet_collective::MemberCount;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
        Governance: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TriumvirateMembers: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
        Senate: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        SenateMembers: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>},
        SubnetOwners: pallet_collective::<Instance3>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        SubnetOwnersMembers: pallet_membership::<Instance3>::{Pallet, Call, Storage, Event<T>, Config<T>},
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
    type MaxHolds = ();
    type MaxFreezes = ();
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
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
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::simple_max(weights::Weight::from_parts(1024, 0));
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
    pub const InitialDefaultTake: u16 = 11_796; // 18% honest number.
    pub const InitialWeightsVersionKey: u16 = 0;
    pub const InitialServingRateLimit: u64 = 0; // No limit.
    pub const InitialTxRateLimit: u64 = 0; // Disable rate limit for testing
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
    pub const InitialTargetStakesPerInterval: u16 = 1;
}

// Configure collective pallet for council
parameter_types! {
    pub const GovernanceMotionDuration: BlockNumber = 100;
    pub const GovernanceMaxProposals: u32 = 10;
    pub const GovernanceMaxMembers: u32 = 3;
}

// Configure collective pallet for Senate
parameter_types! {
    pub const SenateMaxMembers: u32 = 12;
}

// Configure collective pallet for Subnet Owners
parameter_types! { // Based on the number of u16::MAX subnets
    pub const SubnetOwnersMaxMembers: u32 = u16::MAX as u32;
}

use pallet_collective::{CanPropose, CanVote, GetVotingMembers};
pub struct CanProposeToGovernance;
impl CanPropose<AccountId> for CanProposeToGovernance {
    fn can_propose(account: &AccountId) -> bool {
        TriumvirateMembers::members().contains(account)
    }
}

pub struct CanVoteToGovernance;
impl CanVote<AccountId> for CanVoteToGovernance {
    fn can_vote(group: pallet_collective::VotingGroup) -> bool {
        T::GovernanceGroups::contains(group)
    }

    fn can_vote_for_group(account: &AccountId, group: pallet_collective::VotingGroup) -> bool {
        match group {
            pallet_collective::VotingGroup::Triumvirate => {
                TriumvirateMembers::members().contains(account)
            }
            pallet_collective::VotingGroup::Senate => Senate::members()
                .any(|member| SubtensorModule::coldkey_owns_hotkey(account, *member)),
            pallet_collective::VotingGroup::SubnetOwners => SubnetOwners::is_member(account),
            _ => false,
        }
    }

    fn get_member_account(account: &AccountId, group: pallet_collective::VotingGroup) -> AccountId {
        match group {
            pallet_collective::VotingGroup::Senate => {
                // Assume the account has a hotkey in the Senate
                SenateMembers::members()
                    .iter()
                    .find(|member| SubtensorModule::coldkey_owns_hotkey(account, *member))
                    .map_or(*account, |member| *member)
            }
            _ => *account,
        }
    }
}

use pallet_subtensor::{MemberManagement, SubnetOwner};
pub struct ManageSenateMembers;
impl MemberManagement<AccountId> for ManageSenateMembers {
    fn add_member(account: &AccountId) -> DispatchResult {
        let who = *account;
        SenateMembers::add_member(RawOrigin::Root.into(), who)
    }

    fn remove_member(account: &AccountId) -> DispatchResult {
        let who = *account;
        SenateMembers::remove_member(RawOrigin::Root.into(), who)
    }

    fn swap_member(rm: &AccountId, add: &AccountId) -> DispatchResult {
        let remove = *rm;
        let add = *add;

        SenateMembers::swap_member(RawOrigin::Root.into(), remove, add)
    }

    fn is_member(account: &AccountId) -> bool {
        Senate::is_member(account)
    }

    fn members() -> Vec<AccountId> {
        Senate::members()
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

pub struct ManageSubnetOwnersMembers;
impl MemberManagement<AccountId> for ManageSubnetOwnersMembers {
    fn add_member(account: &AccountId) -> DispatchResult {
        let who = *account;
        SubnetOwnersMembers::add_member(RawOrigin::Root.into(), who)
    }

    fn remove_member(account: &AccountId) -> DispatchResult {
        let who = *account;
        SubnetOwnersMembers::remove_member(RawOrigin::Root.into(), who)
    }

    fn swap_member(rm: &AccountId, add: &AccountId) -> DispatchResult {
        let remove = *rm;
        let add = *add;

        Governance::remove_votes(rm)?;
        SubnetOwnersMembers::swap_member(RawOrigin::Root.into(), remove, add)
    }

    fn is_member(account: &AccountId) -> bool {
        SubnetOwners::members().contains(account)
    }

    fn members() -> Vec<AccountId> {
        SubnetOwners::members().into()
    }

    fn max_members() -> u32 {
        SubnetOwnersMaxMembers::get()
    }
}

pub struct GetSubnetOwnersMemberCount;
impl GetVotingMembers<MemberCount> for GetSubnetOwnersMemberCount {
    fn get_count() -> MemberCount {
        SubnetOwners::members().len() as u32
    }
}
impl Get<MemberCount> for GetSubnetOwnersMemberCount {
    fn get() -> MemberCount {
        SubnetOwnersMaxMembers::get()
    }
}

// We call pallet_collective GovernanceCollective
type GovernanceCollective = pallet_collective::Instance1;
impl pallet_collective::Config<GovernanceCollective> for Test {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = GovernanceMotionDuration;
    type MaxProposals = GovernanceMaxProposals;
    type MaxMembers = GetSenateMemberCount;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Test>;
    type SetMembersOrigin = EnsureNever<AccountId>;
    type CanPropose = CanProposeToGovernance;
    type CanVote = CanVoteToGovernance;
    type MaxVotingGroups = MaxVotingGroups;
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
    type MembershipInitialized = ();
    type MembershipChanged = ();
    type MaxMembers = GovernanceMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Test>;
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
    type MembershipInitialized = ();
    type MembershipChanged = ();
    type MaxMembers = SenateMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Test>;
}

// We call our subnet owners membership SubnetOwnersMembership
type SubnetOwnersMembership = pallet_membership::Instance3;
impl pallet_membership::Config<SubnetOwnersMembership> for Test {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type PrimeOrigin = EnsureRoot<AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
    type MaxMembers = SubnetOwnersMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Test>;
}

impl pallet_subtensor::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InitialIssuance = InitialIssuance;
    type SudoRuntimeCall = TestRuntimeCall;
    type GovernanceOrigin = frame_system::EnsureSigned<AccountId>;
    type SenateMembers = ManageSenateMembers;
    type SubnetOwnersMembers = ManageSubnetOwnersMembers;
    type GovernanceInterface = GovernanceVotes;

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
    type InitialWeightsVersionKey = InitialWeightsVersionKey;
    type InitialMaxDifficulty = InitialMaxDifficulty;
    type InitialMinDifficulty = InitialMinDifficulty;
    type InitialServingRateLimit = InitialServingRateLimit;
    type InitialTxRateLimit = InitialTxRateLimit;
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
