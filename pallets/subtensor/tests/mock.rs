use frame_support::{assert_ok, parameter_types, traits::{Everything, Hooks}, weights};
use frame_system::{limits};
use frame_support::traits::{StorageMapShim};
use frame_system as system;
use frame_system::Config;
use sp_core::{H256, U256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
		SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Storage, Event},
	}
);

#[allow(dead_code)]
pub type SubtensorCall = pallet_subtensor::Call<Test>;

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
	type ExistentialDeposit = ();
	type AccountStore = StorageMapShim<
		pallet_balances::Account<Test>,
		frame_system::Provider<Test>,
		AccountId,
		pallet_balances::AccountData<Balance>,
	>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = U256;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const InitialMinAllowedWeights: u16 = 0;
	pub const InitialEmissionValue: u16 = 0;
	pub const InitialMaxWeightsLimit: u16 = u16::MAX;
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::simple_max(weights::Weight::from_ref_time(1024));
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
	pub const InitialTxRateLimit: u64 = 2; // 2 blocks per stake/unstake/delegate

	pub const InitialBurn: u64 = 0; 
	pub const InitialMinBurn: u64 = 0; 
	pub const InitialMaxBurn: u64 = 1_000_000_000;

	pub const InitialValidatorBatchSize: u16 = 10;
	pub const InitialValidatorSequenceLen: u16 = 10;
	pub const InitialValidatorPruneLen: u64 = 0;
	pub const InitialValidatorEpochLen: u16 = 10;
	pub const InitialValidatorEpochsPerReset: u16 = 10;
	pub const InitialValidatorExcludeQuantile: u16 = 10;
	pub const InitialValidatorLogitsDivergence: u16 = 0;
	pub const InitialScalingLawPower: u16 = 50;
	pub const InitialSynergyScalingLawPower: u16 = 50;
	pub const InitialMaxAllowedValidators: u16 = 100;

	pub const InitialIssuance: u64 = 548833985028256;
	pub const InitialDifficulty: u64 = 10000;
	pub const InitialActivityCutoff: u16 = 5000;
	pub const InitialAdjustmentInterval: u16 = 100;
	pub const InitialMaxRegistrationsPerBlock: u16 = 3;
	pub const InitialTargetRegistrationsPerInterval: u16 = 2;
	pub const InitialPruningScore : u16 = u16::MAX;
	pub const InitialRegistrationRequirement: u16 = u16::MAX; // Top 100%
	pub const InitialMinDifficulty: u64 = 1;
	pub const InitialMaxDifficulty: u64 = u64::MAX;
	pub const InitialRAORecycledForRegistration: u64 = 0;

}
impl pallet_subtensor::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type InitialIssuance = InitialIssuance;

	type InitialMinAllowedWeights = InitialMinAllowedWeights;
	type InitialEmissionValue  = InitialEmissionValue;
	type InitialMaxWeightsLimit = InitialMaxWeightsLimit;
	type InitialTempo = InitialTempo;
	type InitialDifficulty = InitialDifficulty;
	type InitialAdjustmentInterval = InitialAdjustmentInterval;
	type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
	type InitialRho = InitialRho;
	type InitialKappa = InitialKappa;
	type InitialMaxAllowedUids = InitialMaxAllowedUids;
	type InitialValidatorBatchSize = InitialValidatorBatchSize;
	type InitialValidatorSequenceLen = InitialValidatorSequenceLen;
	type InitialValidatorPruneLen = InitialValidatorPruneLen;
	type InitialValidatorEpochLen = InitialValidatorEpochLen;
	type InitialValidatorEpochsPerReset = InitialValidatorEpochsPerReset;
	type InitialValidatorExcludeQuantile = InitialValidatorExcludeQuantile;
	type InitialValidatorLogitsDivergence = InitialValidatorLogitsDivergence;
	type InitialScalingLawPower = InitialScalingLawPower;
	type InitialSynergyScalingLawPower = InitialSynergyScalingLawPower;
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
}

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Test>;
}

// Build genesis storage according to the mock runtime.
//pub fn new_test_ext() -> sp_io::TestExternalities {
//	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
//}

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

#[allow(dead_code)]
pub fn test_ext_with_balances(balances : Vec<(U256, u128)>) -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	pallet_balances::GenesisConfig::<Test> { balances: balances.iter().map(|(a, b)| (*a, *b as u64)).collect::<Vec<(U256, u64)>>()  }
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

#[allow(dead_code)]
pub fn register_ok_neuron( netuid: u16, hotkey_account_id: U256, coldkey_account_id: U256, start_nonce: u64) {
	let block_number: u64 = SubtensorModule::get_current_block_as_u64();
	let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, block_number, start_nonce, &hotkey_account_id);
	let result = SubtensorModule::register( <<Test as frame_system::Config>::RuntimeOrigin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id );
	assert_ok!(result);
	log::info!("Register ok neuron: netuid: {:?}, coldkey: {:?}, hotkey: {:?}", netuid, hotkey_account_id, coldkey_account_id );
}

#[allow(dead_code)]
pub fn add_network(netuid: u16, tempo: u16, modality: u16){
	let result = SubtensorModule::do_add_network(<<Test as Config>::RuntimeOrigin>::root(), netuid, tempo, modality);
	SubtensorModule::set_network_registration_allowed( netuid, true );
	assert_ok!(result);
}

