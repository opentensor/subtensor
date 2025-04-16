use frame_support::construct_runtime;
use frame_support::{
    PalletId, parameter_types,
    traits::{ConstU32, Everything},
};
use frame_system::{self as system, EnsureRoot};
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};
use subtensor_swap_interface::LiquidityDataProvider;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        Swap: crate::pallet,
    }
);

pub type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u32;
pub const OK_COLDKEY_ACCOUNT_ID: AccountId = 1;
pub const OK_HOTKEY_ACCOUNT_ID: AccountId = 1000;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
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
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
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
    type MaxConsumers = ConstU32<16>;
    type Nonce = u64;
    type Block = Block;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

parameter_types! {
    pub const SwapProtocolId: PalletId = PalletId(*b"ten/swap");
    pub const MaxFeeRate: u16 = 10000; // 15.26%
    pub const MaxPositions: u32 = 100;
    pub const MinimumLiquidity: u64 = 1_000;
}

// Mock implementor of LiquidityDataProvider trait
pub struct MockLiquidityProvider;

impl LiquidityDataProvider<AccountId> for MockLiquidityProvider {
    fn tao_reserve(netuid: u16) -> u64 {
        match netuid {
            123 => 1_000,
            _ => 1_000_000_000_000
        }
    }

    fn alpha_reserve(netuid: u16) -> u64 {
        match netuid {
            123 => 1,
            _ => 4_000_000_000_000
        }
    }

    fn tao_balance(account_id: &AccountId) -> u64 {
        if *account_id == OK_COLDKEY_ACCOUNT_ID {
            100_000_000_000_000
        } else {
            1_000_000_000
        }
    }

    fn alpha_balance(_: u16, coldkey_account_id: &AccountId, hotkey_account_id: &AccountId) -> u64 {
        if (*coldkey_account_id == OK_COLDKEY_ACCOUNT_ID) && (*hotkey_account_id == OK_HOTKEY_ACCOUNT_ID) {
            100_000_000_000_000
        } else {
            1_000_000_000
        }
    }
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AdminOrigin = EnsureRoot<AccountId>;
    type LiquidityDataProvider = MockLiquidityProvider;
    type ProtocolId = SwapProtocolId;
    type MaxFeeRate = MaxFeeRate;
    type MaxPositions = MaxPositions;
    type MinimumLiquidity = MinimumLiquidity;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
