#![allow(clippy::expect_used)]

use node_subtensor_runtime::{RuntimeGenesisConfig, System};
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;
use sp_runtime::traits::SaturatedConversion;
use subtensor_runtime_common::{AccountId, Balance};

pub struct ExtBuilder {
    balances: Vec<(AccountId, Balance)>,
    block_number: u64,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: Vec::new(),
            block_number: 1,
        }
    }
}

impl ExtBuilder {
    pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    #[allow(dead_code)]
    pub fn with_block_number(mut self, block_number: u64) -> Self {
        self.block_number = block_number;
        self
    }

    pub fn build(self) -> TestExternalities {
        let mut ext: TestExternalities = RuntimeGenesisConfig {
            balances: pallet_balances::GenesisConfig {
                balances: self.balances,
                dev_accounts: None,
            },
            ..Default::default()
        }
        .build_storage()
        .expect("runtime genesis config builds")
        .into();

        let block_number = self.block_number;
        ext.execute_with(|| System::set_block_number(block_number.saturated_into()));
        ext
    }
}
