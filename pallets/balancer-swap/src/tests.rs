use crate::{self as pallet_balancer_swap, *};
use frame_support::{
    assert_ok, assert_noop,
    parameter_types,
    traits::{ConstU32, ConstU64},
    PalletId,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, Perbill,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        BalancerSwap: pallet_balancer_swap,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

// Mock implementations
pub struct MockSubnetInfo;
impl SubnetInfo<u64> for MockSubnetInfo {
    fn exists(_netuid: subtensor_runtime_common::SubnetIndex) -> bool {
        true
    }
    fn is_subtoken_enabled(_netuid: subtensor_runtime_common::SubnetIndex) -> bool {
        true
    }
    fn is_owner(_account: &u64, _netuid: subtensor_runtime_common::SubnetIndex) -> bool {
        true
    }
    fn mechanism(_netuid: subtensor_runtime_common::SubnetIndex) -> u16 {
        1
    }
    fn hotkey_of_uid(
        _netuid: subtensor_runtime_common::SubnetIndex,
        _uid: u16,
    ) -> Result<u64, ()> {
        Ok(0)
    }
    fn get_validator_trust(_netuid: subtensor_runtime_common::SubnetIndex) -> sp_std::vec::Vec<u16> {
        sp_std::vec![]
    }
    fn get_validator_permit(_netuid: subtensor_runtime_common::SubnetIndex) -> sp_std::vec::Vec<bool> {
        sp_std::vec![]
    }
}

pub struct MockBalanceOps;
impl BalanceOps<u64> for MockBalanceOps {
    fn tao_balance(_account: &u64) -> TaoCurrency {
        1_000_000.into()
    }
    fn alpha_balance(
        _netuid: subtensor_runtime_common::SubnetIndex,
        _coldkey: &u64,
        _hotkey: &u64,
    ) -> AlphaCurrency {
        1_000_000.into()
    }
    fn decrease_balance(_account: &u64, _amount: TaoCurrency) -> Result<TaoCurrency, DispatchError> {
        Ok(_amount)
    }
    fn increase_balance(_account: &u64, _amount: TaoCurrency) {
        // No-op for mock
    }
    fn decrease_stake(
        _coldkey: &u64,
        _hotkey: &u64,
        _netuid: subtensor_runtime_common::SubnetIndex,
        _amount: AlphaCurrency,
    ) -> Result<AlphaCurrency, DispatchError> {
        Ok(_amount)
    }
    fn increase_stake(
        _coldkey: &u64,
        _hotkey: &u64,
        _netuid: subtensor_runtime_common::SubnetIndex,
        _amount: AlphaCurrency,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
}

pub struct MockReserve;
impl<C: Currency> CurrencyReserve<C> for MockReserve {
    fn reserve(_index: subtensor_runtime_common::SubnetIndex) -> C {
        1_000_000.into()
    }
    fn increase_provided(_index: subtensor_runtime_common::SubnetIndex, _amount: C) {
        // No-op for mock
    }
    fn decrease_provided(_index: subtensor_runtime_common::SubnetIndex, _amount: C) {
        // No-op for mock
    }
}

parameter_types! {
    pub const ProtocolId: PalletId = PalletId(*b"blnc/swp");
    pub const DefaultTaoWeight: Perbill = Perbill::from_percent(50);
    pub const DefaultAlphaWeight: Perbill = Perbill::from_percent(50);
    pub const DefaultSwapFee: Perbill = Perbill::from_rational(3u32, 1000u32);
    pub const MaxSwapFee: Perbill = Perbill::from_percent(10);
    pub const MinimumLiquidity: u64 = 1_000;
}

impl pallet_balancer_swap::Config for Test {
    type SubnetInfo = MockSubnetInfo;
    type BalanceOps = MockBalanceOps;
    type TaoReserve = MockReserve;
    type AlphaReserve = MockReserve;
    type ProtocolId = ProtocolId;
    type DefaultTaoWeight = DefaultTaoWeight;
    type DefaultAlphaWeight = DefaultAlphaWeight;
    type DefaultSwapFee = DefaultSwapFee;
    type MaxSwapFee = MaxSwapFee;
    type MinimumLiquidity = MinimumLiquidity;
    type WeightInfo = weights::DefaultWeight<Test>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}

#[test]
fn test_initialize_pool() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();
        let tao_amount = 10_000.into();
        let alpha_amount = 10_000.into();

        assert_ok!(BalancerSwap::initialize_pool(netuid, tao_amount, alpha_amount));

        let pool = BalancerSwap::pools(netuid).unwrap();
        assert_eq!(pool.tao_balance, tao_amount);
        assert_eq!(pool.alpha_balance, alpha_amount);
        assert!(pool.total_shares > 0);
    });
}

#[test]
fn test_add_liquidity_balanced() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();
        let coldkey = 1u64;
        let hotkey = 2u64;

        // Initialize pool
        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));

        // Add balanced liquidity
        assert_ok!(BalancerSwap::add_liquidity(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            1_000.into(),
            1_000.into(),
            0, // min_shares
        ));

        // Check shares were minted
        let shares = BalancerSwap::liquidity_shares(netuid, coldkey);
        assert!(shares > 0);
    });
}

#[test]
fn test_add_liquidity_unbalanced() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();
        let coldkey = 1u64;
        let hotkey = 2u64;

        // Initialize pool
        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));

        // Add unbalanced liquidity (more TAO than Alpha)
        assert_ok!(BalancerSwap::add_liquidity(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            2_000.into(),
            500.into(),
            0,
        ));

        let shares = BalancerSwap::liquidity_shares(netuid, coldkey);
        assert!(shares > 0);
    });
}

#[test]
fn test_remove_liquidity() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();
        let coldkey = 1u64;
        let hotkey = 2u64;

        // Initialize and add liquidity
        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));
        assert_ok!(BalancerSwap::add_liquidity(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            1_000.into(),
            1_000.into(),
            0,
        ));

        let shares = BalancerSwap::liquidity_shares(netuid, coldkey);

        // Remove liquidity
        assert_ok!(BalancerSwap::remove_liquidity(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            shares / 2,
            0.into(),
            0.into(),
        ));

        // Check shares were burned
        let remaining_shares = BalancerSwap::liquidity_shares(netuid, coldkey);
        assert_eq!(remaining_shares, shares / 2);
    });
}

#[test]
fn test_spot_price_calculation() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();

        // Initialize pool with 1:1 ratio
        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));

        let price = BalancerSwap::current_price(netuid);
        
        // For 50/50 pool with equal balances, price should be 1.0
        assert_eq!(price, U96F32::from_num(1.0));
    });
}

#[test]
fn test_swap_tao_for_alpha() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();

        // Initialize pool
        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));

        // Execute swap
        let (amount_out, fee) = BalancerSwap::do_swap(netuid, TokenType::Tao, 1_000, false).unwrap();

        assert!(amount_out > 0);
        assert!(amount_out < 1_000); // Due to price impact
        assert!(fee > 0); // Fee should be charged
    });
}

#[test]
fn test_set_pool_weights() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();

        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));

        // Set 80/20 weights
        assert_ok!(BalancerSwap::set_pool_weights(
            RuntimeOrigin::root(),
            netuid,
            Perbill::from_percent(80),
            Perbill::from_percent(20),
        ));

        let pool = BalancerSwap::pools(netuid).unwrap();
        assert_eq!(pool.tao_weight, Perbill::from_percent(80));
        assert_eq!(pool.alpha_weight, Perbill::from_percent(20));
    });
}

#[test]
fn test_invalid_weights() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();

        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));

        // Try to set weights that don't sum to 100%
        assert_noop!(
            BalancerSwap::set_pool_weights(
                RuntimeOrigin::root(),
                netuid,
                Perbill::from_percent(60),
                Perbill::from_percent(30),
            ),
            Error::<Test>::InvalidWeights
        );
    });
}

#[test]
fn test_slippage_protection() {
    new_test_ext().execute_with(|| {
        let netuid = 1.into();
        let coldkey = 1u64;
        let hotkey = 2u64;

        assert_ok!(BalancerSwap::initialize_pool(netuid, 10_000.into(), 10_000.into()));

        // Try to add liquidity with very high min_shares (should fail)
        assert_noop!(
            BalancerSwap::add_liquidity(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                1_000.into(),
                1_000.into(),
                1_000_000, // Unrealistic min_shares
            ),
            Error::<Test>::SlippageExceeded
        );
    });
}



