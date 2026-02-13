use crate as pallet_shield;

use frame_support::{construct_runtime, derive_impl};
use sp_runtime::BuildStorage;
use std::cell::RefCell;

pub type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system = 0,
        MevShield: pallet_shield = 1,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
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
