//! Mock swap/time helpers for limit-orders precompile tests.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use frame_support::{PalletId, parameter_types};
use sp_runtime::AccountId32;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};
use subtensor_swap_interface::OrderSwapInterface;

type AccountId = AccountId32;

thread_local! {
    static MOCK_PRICE: RefCell<U64F64> = RefCell::new(U64F64::from_num(1u32));
    static MOCK_BUY_ALPHA_RETURN: RefCell<u64> = const { RefCell::new(0u64) };
    static ALPHA_BALANCES: RefCell<HashMap<(AccountId, AccountId, NetUid), u64>> =
        RefCell::new(HashMap::new());
    static TAO_BALANCES: RefCell<HashMap<AccountId, u64>> = RefCell::new(HashMap::new());
    static HOTKEY_REGISTRATIONS: RefCell<HashSet<(AccountId, AccountId)>> =
        RefCell::new(HashSet::new());
    static MOCK_TIME_MS: RefCell<u64> = const { RefCell::new(1_000_000u64) };
}

pub struct LimitOrdersMockSwap;

impl LimitOrdersMockSwap {
    pub fn clear() {
        ALPHA_BALANCES.with(|b| b.borrow_mut().clear());
        TAO_BALANCES.with(|b| b.borrow_mut().clear());
        HOTKEY_REGISTRATIONS.with(|r| r.borrow_mut().clear());
        MOCK_PRICE.with(|p| *p.borrow_mut() = U64F64::from_num(1u32));
        MOCK_BUY_ALPHA_RETURN.with(|v| *v.borrow_mut() = 0);
    }

    pub fn set_price(price: f64) {
        MOCK_PRICE.with(|p| *p.borrow_mut() = U64F64::from_num(price));
    }

    pub fn set_buy_alpha_return(alpha: u64) {
        MOCK_BUY_ALPHA_RETURN.with(|v| *v.borrow_mut() = alpha);
    }

    pub fn set_tao_balance(account: AccountId, amount: u64) {
        TAO_BALANCES.with(|b| {
            b.borrow_mut().insert(account, amount);
        });
    }

    pub fn register_hotkey(coldkey: &AccountId, hotkey: &AccountId) {
        HOTKEY_REGISTRATIONS.with(|r| {
            r.borrow_mut().insert((coldkey.clone(), hotkey.clone()));
        });
    }
}

impl OrderSwapInterface<AccountId> for LimitOrdersMockSwap {
    fn buy_alpha(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        tao_amount: TaoBalance,
        _limit_price: TaoBalance,
        _apply_limits: bool,
    ) -> Result<AlphaBalance, frame_support::pallet_prelude::DispatchError> {
        let tao = tao_amount.to_u64();
        TAO_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let bal = map.entry(coldkey.clone()).or_insert(0);
            *bal = bal.saturating_sub(tao);
        });
        let alpha_out = MOCK_BUY_ALPHA_RETURN.with(|v| *v.borrow());
        ALPHA_BALANCES.with(|b| {
            let mut map = b.borrow_mut();
            let bal = map
                .entry((coldkey.clone(), hotkey.clone(), netuid))
                .or_insert(0);
            *bal = bal.saturating_add(alpha_out);
        });
        Ok(AlphaBalance::from(alpha_out))
    }

    fn sell_alpha(
        _coldkey: &AccountId,
        _hotkey: &AccountId,
        _netuid: NetUid,
        _alpha_amount: AlphaBalance,
        _limit_price: TaoBalance,
        _apply_limits: bool,
    ) -> Result<TaoBalance, frame_support::pallet_prelude::DispatchError> {
        Ok(TaoBalance::from(0))
    }

    fn current_alpha_price(_netuid: NetUid) -> U64F64 {
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
        Ok(())
    }

    fn register_pallet_hotkey(
        coldkey: &AccountId,
        hotkey: &AccountId,
    ) -> frame_support::pallet_prelude::DispatchResult {
        LimitOrdersMockSwap::register_hotkey(coldkey, hotkey);
        Ok(())
    }

    fn pallet_hotkey_registered(coldkey: &AccountId, hotkey: &AccountId) -> bool {
        HOTKEY_REGISTRATIONS.with(|r| r.borrow().contains(&(coldkey.clone(), hotkey.clone())))
    }

    fn transfer_staked_alpha(
        from_coldkey: &AccountId,
        from_hotkey: &AccountId,
        to_coldkey: &AccountId,
        to_hotkey: &AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
        _validate_sender: bool,
        _set_receiver_limit: bool,
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
        Ok(())
    }
}

pub struct LimitOrdersMockTime;

impl LimitOrdersMockTime {
    pub fn set(ms: u64) {
        MOCK_TIME_MS.with(|t| *t.borrow_mut() = ms);
    }
}

impl frame_support::traits::UnixTime for LimitOrdersMockTime {
    fn now() -> core::time::Duration {
        core::time::Duration::from_millis(MOCK_TIME_MS.with(|t| *t.borrow()))
    }
}

parameter_types! {
    pub const LimitOrdersPalletId: PalletId = PalletId(*b"lmt/ordr");
    pub const LimitOrdersPalletHotkey: AccountId32 = AccountId32::new([0xaa; 32]);
    pub const LimitOrdersChainId: u64 = 945;
}
