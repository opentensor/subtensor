use crate as pallet_commitments;
use frame_support::{
    derive_impl,
    pallet_prelude::{Get, TypeInfo},
    traits::{ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    testing::Header,
    traits::{BlakeTwo256, ConstU16, IdentityLookup},
};

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic =
    sp_runtime::generic::UncheckedExtrinsic<AccountId, RuntimeCall, test_crypto::Signature, ()>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system = 1,
        Balances: pallet_balances = 2,
        Commitments: pallet_commitments = 3,
        Drand: pallet_drand = 4,
    }
);

pub type AccountId = u64;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type Block = Block;
    type Nonce = u32;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TestMaxFields;
impl Get<u32> for TestMaxFields {
    fn get() -> u32 {
        16
    }
}
impl TypeInfo for TestMaxFields {
    type Identity = Self;
    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("TestMaxFields", module_path!()))
            .composite(scale_info::build::Fields::unit())
    }
}

pub struct TestCanCommit;
impl pallet_commitments::CanCommit<u64> for TestCanCommit {
    fn can_commit(_netuid: u16, _who: &u64) -> bool {
        true
    }
}

impl pallet_commitments::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
    type MaxFields = TestMaxFields;
    type CanCommit = TestCanCommit;
    type FieldDeposit = ConstU64<0>;
    type InitialDeposit = ConstU64<0>;
    type DefaultRateLimit = ConstU64<0>;
}

impl pallet_drand::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_drand::weights::SubstrateWeight<Test>;
    type AuthorityId = test_crypto::TestAuthId;
    type Verifier = pallet_drand::verifier::QuicknetVerifier;
    type UnsignedPriority = ConstU64<{ 1 << 20 }>;
    type HttpFetchTimeout = ConstU64<1_000>;
}

pub mod test_crypto {
    use sp_core::sr25519::{Public as Sr25519Public, Signature as Sr25519Signature};
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::IdentifyAccount,
    };

    pub const KEY_TYPE: sp_runtime::KeyTypeId = sp_runtime::KeyTypeId(*b"test");

    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    impl frame_system::offchain::AppCrypto<Public, Signature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = Sr25519Signature;
        type GenericPublic = Sr25519Public;
    }

    impl IdentifyAccount for Public {
        type AccountId = u64;

        fn into_account(self) -> u64 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(self.as_ref());
            u64::from_le_bytes(bytes[..8].try_into().unwrap())
        }
    }
}

impl frame_system::offchain::SigningTypes for Test {
    type Public = test_crypto::Public;
    type Signature = test_crypto::Signature;
}

impl frame_system::offchain::CreateSignedTransaction<pallet_drand::Call<Test>> for Test {
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        _public: Self::Public,
        account: Self::AccountId,
        _nonce: u32,
    ) -> Option<(
        RuntimeCall,
        <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
    )> {
        // Create a dummy sr25519 signature from a raw byte array
        let dummy_raw = [0u8; 64];
        let dummy_signature = sp_core::sr25519::Signature::from(dummy_raw);
        let signature = test_crypto::Signature::from(dummy_signature);
        Some((call, (account, signature, ())))
    }
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
