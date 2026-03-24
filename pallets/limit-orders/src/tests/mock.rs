//! Minimal mock runtime for `pallet-limit-orders` unit tests.
//!
//! `AccountId` is `sp_runtime::AccountId32` so that `MultiSignature` works
//! out of the box; test keys come from `sp_keyring::AccountKeyring`.

use std::cell::RefCell;
use std::collections::HashMap;

use codec::Encode;
use frame_support::{
    BoundedVec, PalletId, construct_runtime, derive_impl, parameter_types,
    traits::{ConstU32, Everything},
};
use frame_system as system;
use sp_core::{H256, Pair};
use sp_keyring::Sr25519Keyring as AccountKeyring;
use sp_runtime::{
    AccountId32, BuildStorage, MultiSignature,
    traits::{BlakeTwo256, IdentityLookup},
};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};
use subtensor_swap_interface::OrderSwapInterface;

use crate as pallet_limit_orders;

// ── Runtime ──────────────────────────────────────────────────────────────────

construct_runtime!(
    pub enum Test {
        System: system = 0,
        LimitOrders: pallet_limit_orders = 1,
    }
);

pub type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = AccountId32;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type BaseCallFilter = Everything;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type PalletInfo = PalletInfo;
    type MaxConsumers = ConstU32<16>;
    type Nonce = u64;
    type Block = Block;
}

// ── MockSwap ─────────────────────────────────────────────────────────────────
//
// Records every call so tests can assert that the right transfers happened.

#[derive(Debug, Clone, PartialEq)]
pub enum SwapCall {
    BuyAlpha {
        coldkey: AccountId,
        hotkey: AccountId,
        netuid: NetUid,
        tao: u64,
    },
    SellAlpha {
        coldkey: AccountId,
        hotkey: AccountId,
        netuid: NetUid,
        alpha: u64,
    },
    TransferTao {
        from: AccountId,
        to: AccountId,
        amount: u64,
    },
    TransferStakedAlpha {
        from_coldkey: AccountId,
        from_hotkey: AccountId,
        to_coldkey: AccountId,
        to_hotkey: AccountId,
        netuid: NetUid,
        amount: u64,
    },
}

thread_local! {
    /// Log of every `OrderSwapInterface` call made during a test.
    pub static SWAP_LOG: RefCell<Vec<SwapCall>> = RefCell::new(Vec::new());
    /// Fixed price returned by `current_alpha_price` (default 1.0).
    pub static MOCK_PRICE: RefCell<U96F32> = RefCell::new(U96F32::from_num(1u32));
    /// Fixed alpha returned by `buy_alpha` (default 0 — tests override as needed).
    pub static MOCK_BUY_ALPHA_RETURN: RefCell<u64> = RefCell::new(0u64);
    /// Fixed TAO returned by `sell_alpha` (default 0 — tests override as needed).
    pub static MOCK_SELL_TAO_RETURN: RefCell<u64> = RefCell::new(0u64);
    /// In-memory staked alpha ledger: (coldkey, hotkey, netuid) → balance.
    /// `transfer_staked_alpha` debits/credits this map so tests can assert
    /// on residual balances after distribution.
    pub static ALPHA_BALANCES: RefCell<HashMap<(AccountId, AccountId, NetUid), u64>> =
        RefCell::new(HashMap::new());
    /// In-memory free TAO ledger: account → balance.
    /// `transfer_tao` debits/credits this map so tests can assert
    /// on residual balances after distribution.
    pub static TAO_BALANCES: RefCell<HashMap<AccountId, u64>> =
        RefCell::new(HashMap::new());
    /// When `true`, `buy_alpha` and `sell_alpha` return `DispatchError::Other("pool error")`.
    pub static MOCK_SWAP_FAIL: RefCell<bool> = RefCell::new(false);
}

pub struct MockSwap;

impl MockSwap {
    pub fn set_price(price: f64) {
        MOCK_PRICE.with(|p| *p.borrow_mut() = U96F32::from_num(price));
    }
    pub fn set_buy_alpha_return(alpha: u64) {
        MOCK_BUY_ALPHA_RETURN.with(|v| *v.borrow_mut() = alpha);
    }
    pub fn set_sell_tao_return(tao: u64) {
        MOCK_SELL_TAO_RETURN.with(|v| *v.borrow_mut() = tao);
    }
    pub fn set_swap_fail(fail: bool) {
        MOCK_SWAP_FAIL.with(|v| *v.borrow_mut() = fail);
    }
    pub fn clear_log() {
        SWAP_LOG.with(|l| l.borrow_mut().clear());
        ALPHA_BALANCES.with(|b| b.borrow_mut().clear());
        TAO_BALANCES.with(|b| b.borrow_mut().clear());
    }
    /// Seed a staked alpha balance for a (coldkey, hotkey, netuid) triple.
    pub fn set_alpha_balance(coldkey: AccountId, hotkey: AccountId, netuid: NetUid, amount: u64) {
        ALPHA_BALANCES.with(|b| {
            b.borrow_mut().insert((coldkey, hotkey, netuid), amount);
        });
    }
    /// Query the current staked alpha balance for a (coldkey, hotkey, netuid) triple.
    pub fn alpha_balance(coldkey: &AccountId, hotkey: &AccountId, netuid: NetUid) -> u64 {
        ALPHA_BALANCES.with(|b| {
            *b.borrow()
                .get(&(coldkey.clone(), hotkey.clone(), netuid))
                .unwrap_or(&0)
        })
    }
    /// Seed a free TAO balance for an account.
    pub fn set_tao_balance(account: AccountId, amount: u64) {
        TAO_BALANCES.with(|b| {
            b.borrow_mut().insert(account, amount);
        });
    }
    /// Query the current free TAO balance for an account.
    pub fn tao_balance(account: &AccountId) -> u64 {
        TAO_BALANCES.with(|b| *b.borrow().get(account).unwrap_or(&0))
    }
    pub fn log() -> Vec<SwapCall> {
        SWAP_LOG.with(|l| l.borrow().clone())
    }
    pub fn tao_transfers() -> Vec<(AccountId, AccountId, u64)> {
        Self::log()
            .into_iter()
            .filter_map(|c| {
                if let SwapCall::TransferTao { from, to, amount } = c {
                    Some((from, to, amount))
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn alpha_transfers() -> Vec<(AccountId, AccountId, AccountId, AccountId, NetUid, u64)> {
        Self::log()
            .into_iter()
            .filter_map(|c| {
                if let SwapCall::TransferStakedAlpha {
                    from_coldkey,
                    from_hotkey,
                    to_coldkey,
                    to_hotkey,
                    netuid,
                    amount,
                } = c
                {
                    Some((
                        from_coldkey,
                        from_hotkey,
                        to_coldkey,
                        to_hotkey,
                        netuid,
                        amount,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl OrderSwapInterface<AccountId> for MockSwap {
    fn buy_alpha(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        tao_amount: TaoBalance,
        _limit_price: TaoBalance,
    ) -> Result<AlphaBalance, frame_support::pallet_prelude::DispatchError> {
        if MOCK_SWAP_FAIL.with(|v| *v.borrow()) {
            return Err(frame_support::pallet_prelude::DispatchError::Other(
                "pool error",
            ));
        }
        let tao = tao_amount.to_u64();
        let alpha_out = MOCK_BUY_ALPHA_RETURN.with(|v| *v.borrow());
        // Debit TAO from coldkey, credit alpha to (coldkey, hotkey, netuid).
        TAO_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let bal = map.entry(coldkey.clone()).or_insert(0);
            *bal = bal.saturating_sub(tao);
        });
        ALPHA_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let bal = map
                .entry((coldkey.clone(), hotkey.clone(), netuid))
                .or_insert(0);
            *bal = bal.saturating_add(alpha_out);
        });
        SWAP_LOG.with(|l| {
            l.borrow_mut().push(SwapCall::BuyAlpha {
                coldkey: coldkey.clone(),
                hotkey: hotkey.clone(),
                netuid,
                tao,
            })
        });
        Ok(AlphaBalance::from(alpha_out))
    }

    fn sell_alpha(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha_amount: AlphaBalance,
        _limit_price: TaoBalance,
    ) -> Result<TaoBalance, frame_support::pallet_prelude::DispatchError> {
        if MOCK_SWAP_FAIL.with(|v| *v.borrow()) {
            return Err(frame_support::pallet_prelude::DispatchError::Other(
                "pool error",
            ));
        }
        let alpha = alpha_amount.to_u64();
        let tao_out = MOCK_SELL_TAO_RETURN.with(|v| *v.borrow());
        // Debit alpha from (coldkey, hotkey, netuid), credit TAO to coldkey.
        ALPHA_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let bal = map
                .entry((coldkey.clone(), hotkey.clone(), netuid))
                .or_insert(0);
            *bal = bal.saturating_sub(alpha);
        });
        TAO_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let bal = map.entry(coldkey.clone()).or_insert(0);
            *bal = bal.saturating_add(tao_out);
        });
        SWAP_LOG.with(|l| {
            l.borrow_mut().push(SwapCall::SellAlpha {
                coldkey: coldkey.clone(),
                hotkey: hotkey.clone(),
                netuid,
                alpha,
            })
        });
        Ok(TaoBalance::from(tao_out))
    }

    fn current_alpha_price(_netuid: NetUid) -> U96F32 {
        MOCK_PRICE.with(|p| *p.borrow())
    }

    fn transfer_tao(
        from: &AccountId,
        to: &AccountId,
        amount: TaoBalance,
    ) -> frame_support::pallet_prelude::DispatchResult {
        let amt = amount.to_u64();
        TAO_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let from_bal = map.entry(from.clone()).or_insert(0);
            *from_bal = from_bal.saturating_sub(amt);
            let to_bal = map.entry(to.clone()).or_insert(0);
            *to_bal = to_bal.saturating_add(amt);
        });
        SWAP_LOG.with(|l| {
            l.borrow_mut().push(SwapCall::TransferTao {
                from: from.clone(),
                to: to.clone(),
                amount: amt,
            })
        });
        Ok(())
    }

    fn transfer_staked_alpha(
        from_coldkey: &AccountId,
        from_hotkey: &AccountId,
        to_coldkey: &AccountId,
        to_hotkey: &AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
    ) -> frame_support::pallet_prelude::DispatchResult {
        let amt = amount.to_u64();
        ALPHA_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let from_bal = map
                .entry((from_coldkey.clone(), from_hotkey.clone(), netuid))
                .or_insert(0);
            *from_bal = from_bal.saturating_sub(amt);
            let to_bal = map
                .entry((to_coldkey.clone(), to_hotkey.clone(), netuid))
                .or_insert(0);
            *to_bal = to_bal.saturating_add(amt);
        });
        SWAP_LOG.with(|l| {
            l.borrow_mut().push(SwapCall::TransferStakedAlpha {
                from_coldkey: from_coldkey.clone(),
                from_hotkey: from_hotkey.clone(),
                to_coldkey: to_coldkey.clone(),
                to_hotkey: to_hotkey.clone(),
                netuid,
                amount: amt,
            })
        });
        Ok(())
    }
}

// ── MockTime ─────────────────────────────────────────────────────────────────

thread_local! {
    pub static MOCK_TIME_MS: RefCell<u64> = RefCell::new(1_000_000u64);
}

pub struct MockTime;

impl MockTime {
    pub fn set(ms: u64) {
        MOCK_TIME_MS.with(|t| *t.borrow_mut() = ms);
    }
}

impl frame_support::traits::UnixTime for MockTime {
    fn now() -> core::time::Duration {
        let ms = MOCK_TIME_MS.with(|t| *t.borrow());
        core::time::Duration::from_millis(ms)
    }
}

// ── Pallet config ─────────────────────────────────────────────────────────────

parameter_types! {
    pub const LimitOrdersPalletId: PalletId = PalletId(*b"lmt/ordr");
    pub const FeeCollectorAccount: AccountId = AccountId::new([0xfe; 32]);
    pub const PalletHotkeyAccount: AccountId = AccountId::new([0xaa; 32]);
}

impl pallet_limit_orders::Config for Test {
    type Signature = MultiSignature;
    type SwapInterface = MockSwap;
    type TimeProvider = MockTime;
    type FeeCollector = FeeCollectorAccount;
    type MaxOrdersPerBatch = ConstU32<64>;
    type PalletId = LimitOrdersPalletId;
    type PalletHotkey = PalletHotkeyAccount;
}

// ── Shared test helpers ───────────────────────────────────────────────────────

pub fn alice() -> AccountId {
    AccountKeyring::Alice.to_account_id()
}
pub fn bob() -> AccountId {
    AccountKeyring::Bob.to_account_id()
}
pub fn charlie() -> AccountId {
    AccountKeyring::Charlie.to_account_id()
}
pub fn dave() -> AccountId {
    AccountKeyring::Dave.to_account_id()
}
pub fn netuid() -> NetUid {
    NetUid::from(1u16)
}

pub const FAR_FUTURE: u64 = u64::MAX;

pub fn make_signed_order(
    keyring: AccountKeyring,
    hotkey: AccountId,
    netuid: NetUid,
    side: crate::OrderSide,
    amount: u64,
    limit_price: u64,
    expiry: u64,
) -> crate::SignedOrder<AccountId, MultiSignature> {
    let signer = keyring.to_account_id();
    let order = crate::Order {
        signer,
        hotkey,
        netuid,
        side,
        amount,
        limit_price,
        expiry,
    };
    let sig = keyring.pair().sign(&order.encode());
    crate::SignedOrder {
        order,
        signature: MultiSignature::Sr25519(sig),
    }
}

pub fn bounded(
    v: Vec<crate::SignedOrder<AccountId, MultiSignature>>,
) -> BoundedVec<crate::SignedOrder<AccountId, MultiSignature>, ConstU32<64>> {
    BoundedVec::try_from(v).unwrap()
}

pub fn order_id(order: &crate::Order<AccountId>) -> H256 {
    crate::pallet::Pallet::<Test>::derive_order_id(order)
}

// ── Test externalities ────────────────────────────────────────────────────────

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
        MockSwap::clear_log();
    });
    ext
}
