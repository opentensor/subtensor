use crate as pallet_shield;
use crate::MLKEM768_PK_LEN;

use frame_support::{BoundedVec, construct_runtime, derive_impl};
use sp_runtime::{BuildStorage, generic, testing::TestSignature};
use std::cell::RefCell;
use stp_shield::ShieldPublicKey;

pub type Block = frame_system::mocking::MockBlock<Test>;

pub type DecodableExtrinsic = generic::UncheckedExtrinsic<u64, RuntimeCall, TestSignature, ()>;
pub type DecodableBlock =
    generic::Block<generic::Header<u64, sp_runtime::traits::BlakeTwo256>, DecodableExtrinsic>;

construct_runtime!(
    pub enum Test {
        System: frame_system = 0,
        MevShield: pallet_shield = 1,
        Utility: pallet_subtensor_utility = 2,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

impl pallet_subtensor_utility::Config for Test {
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = ();
}

thread_local! {
    pub static CURRENT_AUTHOR: RefCell<Option<u64>> = const { RefCell::new(None) };
    pub static NEXT_AUTHOR: RefCell<Option<u64>> = const { RefCell::new(None) };
}

pub struct MockFindAuthors;

impl pallet_shield::FindAuthors<Test> for MockFindAuthors {
    fn find_current_author() -> Option<u64> {
        CURRENT_AUTHOR.with(|a| *a.borrow())
    }
    fn find_next_author() -> Option<u64> {
        NEXT_AUTHOR.with(|a| *a.borrow())
    }
}

impl pallet_shield::Config for Test {
    type AuthorityId = u64;
    type FindAuthors = MockFindAuthors;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    RuntimeGenesisConfig::default()
        .build_storage()
        .expect("valid genesis")
        .into()
}

pub fn valid_pk() -> ShieldPublicKey {
    BoundedVec::truncate_from(vec![0x42; MLKEM768_PK_LEN])
}

pub fn valid_pk_b() -> ShieldPublicKey {
    BoundedVec::truncate_from(vec![0x99; MLKEM768_PK_LEN])
}

pub fn set_authors(current: Option<u64>, next: Option<u64>) {
    CURRENT_AUTHOR.with(|a| *a.borrow_mut() = current);
    NEXT_AUTHOR.with(|a| *a.borrow_mut() = next);
}

pub fn nest_call(call: RuntimeCall, depth: usize) -> RuntimeCall {
    (0..depth).fold(call, |inner, _| {
        RuntimeCall::Utility(pallet_subtensor_utility::Call::batch { calls: vec![inner] })
    })
}

pub fn build_wire_ciphertext(
    key_hash: &[u8; 16],
    kem_ct: &[u8],
    nonce: &[u8; 24],
    aead_ct: &[u8],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(key_hash);
    buf.extend_from_slice(&(kem_ct.len() as u16).to_le_bytes());
    buf.extend_from_slice(kem_ct);
    buf.extend_from_slice(nonce);
    buf.extend_from_slice(aead_ct);
    buf
}
