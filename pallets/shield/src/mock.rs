use crate as pallet_mev_shield;

use frame_support::{construct_runtime, derive_impl, parameter_types, traits::Everything};
use frame_system as system;

use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{ConstU32, H256};
use sp_runtime::traits::BadOrigin;
use sp_runtime::{
    AccountId32, BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};

// -----------------------------------------------------------------------------
// Mock runtime
// -----------------------------------------------------------------------------

pub type UncheckedExtrinsic = system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system = 0,
        Timestamp: pallet_timestamp = 1,
        Aura: pallet_aura = 2,
        MevShield: pallet_mev_shield = 3,
    }
);

// A concrete nonce type used in tests.
pub type TestNonce = u64;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    // Basic system config
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();

    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;

    type Nonce = TestNonce;
    type Hash = H256;
    type Hashing = BlakeTwo256;

    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;

    type BlockHashCount = ();
    type Version = ();
    type PalletInfo = PalletInfo;

    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();

    // Max number of consumer refs per account.
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// Aura mock configuration
parameter_types! {
    pub const MaxAuthorities: u32 = 32;
    pub const AllowMultipleBlocksPerSlot: bool = false;
    pub const SlotDuration: u64 = 6000;
}

impl pallet_aura::Config for Test {
    type AuthorityId = AuraId;
    // For tests we don't need dynamic disabling; just use unit type.
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
    type AllowMultipleBlocksPerSlot = AllowMultipleBlocksPerSlot;
    type SlotDuration = SlotDuration;
}

// -----------------------------------------------------------------------------
// Authority origin for tests – root-only
// -----------------------------------------------------------------------------

/// For tests, treat Root as the “validator set” and return a dummy AccountId.
pub struct TestAuthorityOrigin;

impl pallet_mev_shield::AuthorityOriginExt<RuntimeOrigin> for TestAuthorityOrigin {
    type AccountId = AccountId32;

    fn ensure_validator(origin: RuntimeOrigin) -> Result<Self::AccountId, BadOrigin> {
        // Must be a signed origin.
        let who: AccountId32 = frame_system::ensure_signed(origin).map_err(|_| BadOrigin)?;

        // Interpret the AccountId bytes as an AuraId, just like the real pallet.
        let aura_id =
            <AuraId as sp_core::ByteArray>::from_slice(who.as_ref()).map_err(|_| BadOrigin)?;

        // Check membership in the Aura validator set.
        let is_validator = pallet_aura::Authorities::<Test>::get()
            .into_iter()
            .any(|id| id == aura_id);

        if is_validator {
            Ok(who)
        } else {
            Err(BadOrigin)
        }
    }
}

// -----------------------------------------------------------------------------
// MevShield Config
// -----------------------------------------------------------------------------

impl pallet_mev_shield::Config for Test {
    type RuntimeCall = RuntimeCall;
    type AuthorityOrigin = TestAuthorityOrigin;
}

// -----------------------------------------------------------------------------
// new_test_ext
// -----------------------------------------------------------------------------

pub fn new_test_ext() -> sp_io::TestExternalities {
    // Use the construct_runtime!-generated genesis config.
    RuntimeGenesisConfig::default()
        .build_storage()
        .expect("RuntimeGenesisConfig builds valid default genesis storage")
        .into()
}
