#![allow(clippy::unwrap_used)]

use core::num::NonZeroU64;

use frame_support::construct_runtime;
use frame_support::pallet_prelude::*;
use frame_support::{
    PalletId, parameter_types,
    traits::{ConstU32, Everything},
};
use frame_system::{self as system};
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};
use subtensor_runtime_common::{BalanceOps, NetUid, SubnetInfo};

use crate::pallet::EnabledUserLiquidity;

construct_runtime!(
    pub enum Test {
        System: frame_system = 0,
        Swap: crate::pallet = 1,
    }
);

pub type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u32;
pub const OK_COLDKEY_ACCOUNT_ID: AccountId = 1;
pub const OK_HOTKEY_ACCOUNT_ID: AccountId = 1000;
pub const NOT_SUBNET_OWNER: AccountId = 666;
pub const NON_EXISTENT_NETUID: u16 = 999;

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
    pub const MinimumReserves: NonZeroU64 = NonZeroU64::new(1).unwrap();
}

// Mock implementor of SubnetInfo trait
pub struct MockLiquidityProvider;

impl SubnetInfo<AccountId> for MockLiquidityProvider {
    fn tao_reserve(netuid: NetUid) -> u64 {
        match netuid.into() {
            123u16 => 10_000,
            _ => 1_000_000_000_000,
        }
    }

    fn alpha_reserve(netuid: NetUid) -> u64 {
        match netuid.into() {
            123u16 => 10_000,
            _ => 4_000_000_000_000,
        }
    }

    fn exists(netuid: NetUid) -> bool {
        netuid != NON_EXISTENT_NETUID.into()
    }

    fn mechanism(netuid: NetUid) -> u16 {
        if netuid == NetUid::from(0) { 0 } else { 1 }
    }

    fn is_owner(account_id: &AccountId, _netuid: NetUid) -> bool {
        *account_id != NOT_SUBNET_OWNER
    }
}

pub struct MockBalanceOps;

impl BalanceOps<AccountId> for MockBalanceOps {
    fn tao_balance(account_id: &AccountId) -> u64 {
        if *account_id == OK_COLDKEY_ACCOUNT_ID {
            100_000_000_000_000
        } else {
            1_000_000_000
        }
    }

    fn alpha_balance(
        _: NetUid,
        coldkey_account_id: &AccountId,
        hotkey_account_id: &AccountId,
    ) -> u64 {
        if (*coldkey_account_id == OK_COLDKEY_ACCOUNT_ID)
            && (*hotkey_account_id == OK_HOTKEY_ACCOUNT_ID)
        {
            100_000_000_000_000
        } else {
            1_000_000_000
        }
    }

    fn increase_balance(_coldkey: &AccountId, _tao: u64) {}

    fn decrease_balance(_coldkey: &AccountId, tao: u64) -> Result<u64, DispatchError> {
        Ok(tao)
    }

    fn increase_stake(
        _coldkey: &AccountId,
        _hotkey: &AccountId,
        _netuid: NetUid,
        _alpha: u64,
    ) -> Result<(), DispatchError> {
        Ok(())
    }

    fn decrease_stake(
        _coldkey: &AccountId,
        _hotkey: &AccountId,
        _netuid: NetUid,
        alpha: u64,
    ) -> Result<u64, DispatchError> {
        Ok(alpha)
    }
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SubnetInfo = MockLiquidityProvider;
    type BalanceOps = MockBalanceOps;
    type ProtocolId = SwapProtocolId;
    type MaxFeeRate = MaxFeeRate;
    type MaxPositions = MaxPositions;
    type MinimumLiquidity = MinimumLiquidity;
    type MinimumReserve = MinimumReserves;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);

        for netuid in 0u16..=100 {
            // enable V3 for this range of netuids
            EnabledUserLiquidity::<Test>::set(NetUid::from(netuid), true);
        }
    });
    ext
}
