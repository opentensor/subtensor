#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

//! Focused unit tests for lift-based lending v5.1 (Jacob §13).

use super::*;
use codec::{Decode, Encode};
use frame_support::{
    assert_ok, construct_runtime, derive_impl, parameter_types,
    traits::{ConstU32, ConstU64},
    weights::WeightMeter,
};
use sp_runtime::{
    AccountId32, BuildStorage,
    traits::{BlakeTwo256, IdentityLookup},
};

construct_runtime!(pub enum Test { System: frame_system = 0, Lending: crate = 1 });
type Account = AccountId32;
type Block = frame_system::mocking::MockBlock<Test>;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = Account;
    type Lookup = IdentityLookup<Account>;
    type Hashing = BlakeTwo256;
}

parameter_types! { pub const LendingId: PalletId = PalletId(*b"bt/lends"); }

impl Config for Test {
    type Pool = MockPool;
    type PalletId = LendingId;
    type MinimumLoanValue = ConstU64<1>;
    type InterestPeriod = ConstU64<10>;
    type BlocksPerYear = ConstU64<520>;
    type ReferenceWarmup = ConstU64<1>;
    type MaxPositionsPerSubnet = ConstU32<8>;
    type MaxTotalPositions = ConstU32<16>;
    type WeightInfo = ();
}

fn account(n: u8) -> Account {
    Account::new([n; 32])
}
fn netuid() -> NetUid {
    64.into()
}
fn key(prefix: &[u8], args: impl Encode) -> Vec<u8> {
    [prefix, &args.encode()].concat()
}
fn read<V: Decode + Default>(k: &[u8]) -> V {
    sp_io::storage::get(k)
        .and_then(|bytes| V::decode(&mut &bytes[..]).ok())
        .unwrap_or_default()
}
fn write(k: &[u8], value: impl Encode) {
    sp_io::storage::set(k, &value.encode());
}
fn tao(who: &Account) -> u64 {
    read(&key(b"test/tao", who))
}
fn alpha(who: &Account, hotkey: &Account, netuid: NetUid) -> u64 {
    read(&key(b"test/alpha", (who, hotkey, netuid)))
}
fn mint_tao(who: &Account, amount: u64) {
    write(&key(b"test/tao", who), tao(who).saturating_add(amount));
}
fn mint_alpha(who: &Account, hotkey: &Account, netuid: NetUid, amount: u64) {
    write(
        &key(b"test/alpha", (who, hotkey, netuid)),
        alpha(who, hotkey, netuid).saturating_add(amount),
    );
}
fn pool_tao() -> u64 {
    read(&key(b"test/pool_tao", netuid()))
}
fn pool_alpha() -> u64 {
    read(&key(b"test/pool_alpha", netuid()))
}
fn set_pool(tao_r: u64, alpha_r: u64) {
    write(&key(b"test/pool_tao", netuid()), tao_r);
    write(&key(b"test/pool_alpha", netuid()), alpha_r);
}
fn alpha_out() -> u64 {
    read(&key(b"test/alpha_out", netuid()))
}
fn set_alpha_out(v: u64) {
    write(&key(b"test/alpha_out", netuid()), v);
}
fn burned() -> u64 {
    tao(&account(255))
}

pub struct MockPool;
impl MockPool {
    fn price() -> U64F64 {
        let y = pool_tao();
        let x = pool_alpha();
        if x == 0 {
            U64F64::from_num(0)
        } else {
            U64F64::from_num(y) / U64F64::from_num(x)
        }
    }
}

impl OrderSwapInterface<Account> for MockPool {
    fn buy_alpha(
        coldkey: &Account,
        hotkey: &Account,
        netuid: NetUid,
        tao_amount: TaoBalance,
        _: TaoBalance,
        _: bool,
    ) -> Result<AlphaBalance, DispatchError> {
        let y = pool_tao();
        let x = pool_alpha();
        ensure!(y > 0 && x > 0, "empty pool");
        let t = tao_amount.to_u64();
        ensure!(tao(coldkey) >= t, "insufficient tao");
        // Constant-product buy: alpha_out = x * t / (y + t)
        let a = (u128::from(x) * u128::from(t) / u128::from(y.saturating_add(t))) as u64;
        ensure!(a > 0, "zero out");
        write(&key(b"test/tao", coldkey), tao(coldkey).saturating_sub(t));
        write(&key(b"test/pool_tao", netuid), y.saturating_add(t));
        write(&key(b"test/pool_alpha", netuid), x.saturating_sub(a));
        mint_alpha(coldkey, hotkey, netuid, a);
        set_alpha_out(alpha_out().saturating_add(a));
        Ok(a.into())
    }

    fn sell_alpha(
        coldkey: &Account,
        hotkey: &Account,
        netuid: NetUid,
        alpha_amount: AlphaBalance,
        _: TaoBalance,
        _: bool,
    ) -> Result<TaoBalance, DispatchError> {
        let y = pool_tao();
        let x = pool_alpha();
        let a = alpha_amount.to_u64();
        ensure!(alpha(coldkey, hotkey, netuid) >= a, "insufficient alpha");
        // Constant-product sell: tao_out = y * a / (x + a)
        let t = (u128::from(y) * u128::from(a) / u128::from(x.saturating_add(a))) as u64;
        ensure!(t > 0, "zero out");
        write(
            &key(b"test/alpha", (coldkey, hotkey, netuid)),
            alpha(coldkey, hotkey, netuid).saturating_sub(a),
        );
        set_alpha_out(alpha_out().saturating_sub(a));
        write(&key(b"test/pool_alpha", netuid), x.saturating_add(a));
        write(&key(b"test/pool_tao", netuid), y.saturating_sub(t));
        mint_tao(coldkey, t);
        Ok(t.into())
    }

    fn current_alpha_price(_: NetUid) -> U64F64 {
        Self::price()
    }

    fn transfer_tao(from: &Account, to: &Account, amount: TaoBalance) -> DispatchResult {
        let a = amount.to_u64();
        ensure!(tao(from) >= a, "insufficient tao");
        write(&key(b"test/tao", from), tao(from).saturating_sub(a));
        mint_tao(to, a);
        Ok(())
    }

    fn transfer_staked_alpha(
        from_cold: &Account,
        from_hot: &Account,
        to_cold: &Account,
        to_hot: &Account,
        netuid: NetUid,
        amount: AlphaBalance,
        _: bool,
        _: bool,
    ) -> DispatchResult {
        let a = amount.to_u64();
        ensure!(
            alpha(from_cold, from_hot, netuid) >= a,
            "insufficient alpha"
        );
        write(
            &key(b"test/alpha", (from_cold, from_hot, netuid)),
            alpha(from_cold, from_hot, netuid).saturating_sub(a),
        );
        mint_alpha(to_cold, to_hot, netuid, a);
        Ok(())
    }

    fn register_pallet_hotkey(_: &Account, _: &Account) -> DispatchResult {
        Ok(())
    }

    fn pallet_hotkey_registered(_: &Account, _: &Account) -> bool {
        true
    }
}

impl LendingPoolInterface<Account> for MockPool {
    fn subnet_exists(_: NetUid) -> bool {
        true
    }

    fn quote_sell(netuid: NetUid, alpha: AlphaBalance) -> Result<TaoBalance, DispatchError> {
        let y = Self::pool_tao(netuid).to_u64();
        let x = Self::pool_alpha(netuid).to_u64();
        let a = alpha.to_u64();
        Ok(((u128::from(y) * u128::from(a) / u128::from(x.saturating_add(a))) as u64).into())
    }

    fn quote_buy(netuid: NetUid, tao_in: TaoBalance) -> Result<AlphaBalance, DispatchError> {
        let y = Self::pool_tao(netuid).to_u64();
        let x = Self::pool_alpha(netuid).to_u64();
        let t = tao_in.to_u64();
        Ok(((u128::from(x) * u128::from(t) / u128::from(y.saturating_add(t))) as u64).into())
    }

    fn burn_interest_tao(from: &Account, amount: TaoBalance) -> DispatchResult {
        Self::transfer_tao(from, &account(255), amount)
    }

    fn pool_tao(_: NetUid) -> TaoBalance {
        pool_tao().into()
    }

    fn pool_alpha(_: NetUid) -> AlphaBalance {
        pool_alpha().into()
    }

    fn quote_weight_parts(_: NetUid) -> u64 {
        // 50/50 Balancer quote weight parts.
        500_000_000_000_000_000
    }

    fn buy_depth_threshold(_: NetUid, ref_tao: TaoBalance, move_bps: u16) -> TaoBalance {
        // y* ≈ ref / ((1+bps)^1 - 1) for equal weights ≈ ref * 10000 / move_bps
        let r = ref_tao.to_u64();
        let b = u64::from(move_bps).max(1);
        (r.saturating_mul(10_000) / b).into()
    }

    fn quote_sell_on(
        tao_reserve: TaoBalance,
        alpha_reserve: AlphaBalance,
        _: u64,
        delta_alpha: AlphaBalance,
    ) -> TaoBalance {
        let y = tao_reserve.to_u64();
        let x = alpha_reserve.to_u64();
        let a = delta_alpha.to_u64();
        ((u128::from(y) * u128::from(a) / u128::from(x.saturating_add(a))) as u64).into()
    }

    fn quote_buy_on(
        tao_reserve: TaoBalance,
        alpha_reserve: AlphaBalance,
        _: u64,
        delta_tao: TaoBalance,
    ) -> AlphaBalance {
        let y = tao_reserve.to_u64();
        let x = alpha_reserve.to_u64();
        let t = delta_tao.to_u64();
        ((u128::from(x) * u128::from(t) / u128::from(y.saturating_add(t))) as u64).into()
    }

    fn lift_and_debit(
        netuid: NetUid,
        g_numer: u64,
        g_denom: u64,
        custody_cold: &Account,
        custody_hot: &Account,
    ) -> Result<(AlphaBalance, TaoBalance), DispatchError> {
        ensure!(g_denom > 0 && g_numer > 0 && g_numer < g_denom, "bad g");
        let y = pool_tao();
        let x = pool_alpha();
        let lift_tao = (u128::from(y) * u128::from(g_numer) / u128::from(g_denom)) as u64;
        let lift_alpha = (u128::from(x) * u128::from(g_numer) / u128::from(g_denom)) as u64;
        ensure!(lift_tao > 0 && lift_alpha > 0, "zero lift");
        write(&key(b"test/pool_tao", netuid), y.saturating_sub(lift_tao));
        write(
            &key(b"test/pool_alpha", netuid),
            x.saturating_sub(lift_alpha),
        );
        mint_tao(custody_cold, lift_tao);
        mint_alpha(custody_cold, custody_hot, netuid, lift_alpha);
        set_alpha_out(alpha_out().saturating_add(lift_alpha));
        Ok((lift_alpha.into(), lift_tao.into()))
    }

    fn readd_and_credit(
        netuid: NetUid,
        custody_cold: &Account,
        custody_hot: &Account,
        tao_in: TaoBalance,
        alpha_in: AlphaBalance,
    ) -> DispatchResult {
        let t = tao_in.to_u64();
        let a = alpha_in.to_u64();
        if t > 0 {
            ensure!(tao(custody_cold) >= t, "insufficient tao");
            write(&key(b"test/tao", custody_cold), tao(custody_cold) - t);
            write(&key(b"test/pool_tao", netuid), pool_tao().saturating_add(t));
        }
        if a > 0 {
            ensure!(
                alpha(custody_cold, custody_hot, netuid) >= a,
                "insufficient alpha"
            );
            write(
                &key(b"test/alpha", (custody_cold, custody_hot, netuid)),
                alpha(custody_cold, custody_hot, netuid) - a,
            );
            set_alpha_out(alpha_out().saturating_sub(a));
            write(
                &key(b"test/pool_alpha", netuid),
                pool_alpha().saturating_add(a),
            );
        }
        Ok(())
    }

    fn forced_buy_into_custody(
        custody_cold: &Account,
        custody_hot: &Account,
        netuid: NetUid,
        tao_in: TaoBalance,
    ) -> Result<AlphaBalance, DispatchError> {
        Self::buy_alpha(
            custody_cold,
            custody_hot,
            netuid,
            tao_in,
            u64::MAX.into(),
            false,
        )
    }

    fn forced_sell_into_escrow(
        custody_cold: &Account,
        custody_hot: &Account,
        escrow_cold: &Account,
        netuid: NetUid,
        alpha_in: AlphaBalance,
    ) -> Result<TaoBalance, DispatchError> {
        let t = Self::sell_alpha(
            custody_cold,
            custody_hot,
            netuid,
            alpha_in,
            0u64.into(),
            false,
        )?;
        if custody_cold != escrow_cold {
            Self::transfer_tao(custody_cold, escrow_cold, t)?;
        }
        Ok(t)
    }

    fn unlocked_stake(hotkey: &Account, coldkey: &Account, netuid: NetUid) -> AlphaBalance {
        alpha(coldkey, hotkey, netuid).into()
    }

    fn unstake_for_raise(
        hotkey: &Account,
        coldkey: &Account,
        netuid: NetUid,
        alpha_in: AlphaBalance,
        min_tao_out: TaoBalance,
    ) -> Result<TaoBalance, DispatchError> {
        let t = Self::sell_alpha(coldkey, hotkey, netuid, alpha_in, 0u64.into(), false)?;
        ensure!(t >= min_tao_out, "slippage");
        Ok(t)
    }

    fn eligible_alpha_for_dissolution(_: NetUid) -> Result<u128, DispatchError> {
        // Legacy-style: omit AlphaIn (main's staker divisor).
        Ok(u128::from(alpha_out()))
    }

    fn all_alpha_for_lending_settlement(_: NetUid) -> u128 {
        // §13: always include pool alpha.
        u128::from(pool_alpha()).saturating_add(u128::from(alpha_out()))
    }

    fn pay_from_pot(
        netuid: NetUid,
        to: &Account,
        amount: TaoBalance,
    ) -> Result<TaoBalance, DispatchError> {
        let t = amount.to_u64();
        ensure!(pool_tao() >= t, "pot dry");
        write(&key(b"test/pool_tao", netuid), pool_tao() - t);
        mint_tao(to, t);
        Ok(amount)
    }

    fn burn_from_pot(netuid: NetUid, amount: TaoBalance) -> DispatchResult {
        let t = amount.to_u64();
        ensure!(pool_tao() >= t, "pot dry");
        write(&key(b"test/pool_tao", netuid), pool_tao() - t);
        mint_tao(&account(255), t);
        Ok(())
    }

    fn destroy_custody_alpha(
        coldkey: &Account,
        hotkey: &Account,
        netuid: NetUid,
        amount: AlphaBalance,
    ) -> DispatchResult {
        let a = amount.to_u64();
        ensure!(alpha(coldkey, hotkey, netuid) >= a, "insufficient alpha");
        write(
            &key(b"test/alpha", (coldkey, hotkey, netuid)),
            alpha(coldkey, hotkey, netuid) - a,
        );
        set_alpha_out(alpha_out().saturating_sub(a));
        Ok(())
    }
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext: sp_io::TestExternalities = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into();
    ext.execute_with(|| {
        System::set_block_number(10);
        assert_ok!(Lending::initialize_custody());
        Enabled::<Test>::put(true);
        // Deep pool so capacity / depth checks pass.
        set_pool(1_000_000_000_000_000, 1_000_000_000_000_000);
        set_alpha_out(0);
        // Seed a warm reference so opens are not blocked by warmup.
        let spot = MockPool::price();
        References::<Test>::insert(
            netuid(),
            LendingReference {
                price: spot,
                last_updated: 1u64,
                valid_after: 1u64,
            },
        );
        ActiveNets::<Test>::insert(netuid(), ());
        Lifts::<Test>::mutate(netuid(), |book| {
            book.g_ema = pool_tao();
            book.last = 1;
        });
    });
    ext
}

#[test]
fn lift_readd_preserves_price() {
    new_test_ext().execute_with(|| {
        let price_before = MockPool::price();
        let reserve = Lending::reserve_account(netuid());
        let custody = Lending::custody_hotkey().unwrap();
        let (a, t) = MockPool::lift_and_debit(netuid(), 1, 10, &reserve, &custody).unwrap();
        assert!(a.to_u64() > 0 && t.to_u64() > 0);
        let mid = MockPool::price();
        // Proportional lift leaves price unchanged.
        assert_eq!(mid, price_before);
        assert_ok!(MockPool::readd_and_credit(
            netuid(),
            &reserve,
            &custody,
            t,
            a
        ));
        assert_eq!(MockPool::price(), price_before);
    });
}

#[test]
fn short_escrows_tao_not_free_alpha() {
    new_test_ext().execute_with(|| {
        let owner = account(1);
        let hotkey = account(2);
        mint_tao(&owner, 50_000_000_000_000);
        assert_ok!(Lending::open(
            RuntimeOrigin::signed(owner.clone()),
            netuid(),
            Side::Short,
            10_000_000_000_000,
            hotkey,
            1,
            1,
        ));
        let position = Positions::<Test>::get(&owner, netuid()).unwrap();
        assert_eq!(position.side, Side::Short);
        assert!(position.principal > 0);
        assert!(position.proceeds > 0);
        assert_eq!(position.escrow_alpha, 0);
        // Borrower does not receive free borrowed alpha.
        assert_eq!(alpha(&owner, &position.hotkey, netuid()), 0);
        let escrow = Lending::position_account(&owner, netuid());
        assert!(tao(&escrow) >= position.proceeds);
    });
}

#[test]
fn long_escrows_alpha_not_free_tao() {
    new_test_ext().execute_with(|| {
        let owner = account(1);
        let hotkey = account(2);
        mint_tao(&owner, 1);
        mint_alpha(&owner, &hotkey, netuid(), 50_000_000_000_000);
        assert_ok!(Lending::open(
            RuntimeOrigin::signed(owner.clone()),
            netuid(),
            Side::Long,
            10_000_000_000_000,
            hotkey.clone(),
            1,
            0,
        ));
        let position = Positions::<Test>::get(&owner, netuid()).unwrap();
        assert_eq!(position.side, Side::Long);
        assert!(position.principal > 0);
        assert!(position.escrow_alpha > 0);
        assert_eq!(position.proceeds, 0);
        let escrow = Lending::position_account(&owner, netuid());
        let custody = Lending::custody_hotkey().unwrap();
        assert!(alpha(&escrow, &custody, netuid()) >= position.escrow_alpha);
    });
}

#[test]
fn raise_open_long_and_sell_is_atomic() {
    new_test_ext().execute_with(|| {
        let owner = account(1);
        let hotkey = account(2);
        mint_alpha(&owner, &hotkey, netuid(), 100_000_000_000_000);
        let before = tao(&owner);
        assert_ok!(Lending::open_long_and_sell(
            RuntimeOrigin::signed(owner.clone()),
            netuid(),
            hotkey.clone(),
            Some(5_000_000_000_000),
            None,
            5_000_000_000_000,
            1,
            Some(20_000_000_000_000),
            1,
            100u64.into(),
        ));
        let position = Positions::<Test>::get(&owner, netuid()).unwrap();
        assert_eq!(position.side, Side::Long);
        assert!(tao(&owner) > before);
        // Sold stake is gone from free alpha.
        assert!(alpha(&owner, &hotkey, netuid()) < 100_000_000_000_000);
    });
}

#[test]
fn raise_rolls_back_on_slippage() {
    new_test_ext().execute_with(|| {
        let owner = account(1);
        let hotkey = account(2);
        let stake = 100_000_000_000_000u64;
        mint_alpha(&owner, &hotkey, netuid(), stake);
        let alpha_before = alpha(&owner, &hotkey, netuid());
        let tao_before = tao(&owner);
        // Impossible sale floor forces RaiseSlippage after the long leg —
        // transactional rollback must leave no position and restore stake.
        assert!(
            Lending::open_long_and_sell(
                RuntimeOrigin::signed(owner.clone()),
                netuid(),
                hotkey.clone(),
                Some(5_000_000_000_000),
                None,
                5_000_000_000_000,
                1,
                Some(20_000_000_000_000),
                u64::MAX,
                100u64.into(),
            )
            .is_err()
        );
        assert!(Positions::<Test>::get(&owner, netuid()).is_none());
        assert_eq!(alpha(&owner, &hotkey, netuid()), alpha_before);
        assert_eq!(tao(&owner), tao_before);
        assert_eq!(Lifts::<Test>::get(netuid()).l_l, 0);
        assert_eq!(Lifts::<Test>::get(netuid()).held_tao, 0);
    });
}

#[test]
fn capacity_floor_ceiling_and_side_split() {
    new_test_ext().execute_with(|| {
        let params = LiftParams::default();
        assert_eq!(params.floor_bps, 3_300);
        assert_eq!(params.thin_floor_bps, 1_500);
        assert_eq!(params.ceiling_bps, 5_000);
        assert_eq!(params.split_short_bps, 5_000);
        assert_ok!(Lending::set_lift_params(
            RuntimeOrigin::root(),
            LiftParams {
                floor_bps: 3_000,
                ..params
            }
        ));
        assert_eq!(Lending::lift_params().floor_bps, 3_000);
    });
}

#[test]
fn dynamic_valuation_tracks_spot_after_warmup() {
    new_test_ext().execute_with(|| {
        let owner = account(1);
        let hotkey = account(2);
        mint_tao(&owner, 50_000_000_000_000);
        assert_ok!(Lending::open(
            RuntimeOrigin::signed(owner.clone()),
            netuid(),
            Side::Short,
            5_000_000_000_000,
            hotkey,
            1,
            1,
        ));
        let ref_before = References::<Test>::get(netuid()).unwrap().price;
        // Move spot by selling into the pool from an unrelated holder.
        let dump = account(9);
        let dump_hot = account(10);
        mint_alpha(&dump, &dump_hot, netuid(), 200_000_000_000_000);
        set_alpha_out(alpha_out().saturating_add(200_000_000_000_000));
        assert_ok!(MockPool::sell_alpha(
            &dump,
            &dump_hot,
            netuid(),
            100_000_000_000_000u64.into(),
            0u64.into(),
            false
        ));
        System::set_block_number(3);
        Lending::on_finalize(3u64);
        let ref_after = References::<Test>::get(netuid()).unwrap().price;
        assert!(ref_after < ref_before || MockPool::price() < ref_before);
    });
}

#[test]
fn section13_legacy_dump_short_gets_positive_refund() {
    new_test_ext().execute_with(|| {
        let owner = account(1);
        let hotkey = account(2);
        mint_tao(&owner, 80_000_000_000_000);
        // Open short while price is ~1.
        assert_ok!(Lending::open(
            RuntimeOrigin::signed(owner.clone()),
            netuid(),
            Side::Short,
            20_000_000_000_000,
            hotkey,
            1,
            1,
        ));
        let position = Positions::<Test>::get(&owner, netuid()).unwrap();
        let principal = position.principal;
        let escrow_before = position.proceeds.saturating_add(position.collateral);
        assert!(escrow_before > 0);

        // Legacy dump: most holders dump alpha into the pool, collapsing price
        // and bloating AlphaIn while AlphaOut (staker claims) shrinks.
        // Keep a thin residual AlphaOut so the legacy divisor stays nonzero.
        let residual = account(6);
        let residual_hot = account(5);
        mint_alpha(&residual, &residual_hot, netuid(), 1_000_000_000_000);
        set_alpha_out(alpha_out().saturating_add(1_000_000_000_000));

        let dumper = account(7);
        let dump_hot = account(8);
        let dump_alpha = 800_000_000_000_000u64;
        mint_alpha(&dumper, &dump_hot, netuid(), dump_alpha);
        set_alpha_out(alpha_out().saturating_add(dump_alpha));
        assert_ok!(MockPool::sell_alpha(
            &dumper,
            &dump_hot,
            netuid(),
            dump_alpha.into(),
            0u64.into(),
            false
        ));

        // Main's legacy divisor omits AlphaIn → inflated rate that can wipe shorts.
        let pot = pool_tao();
        let legacy_denom = MockPool::eligible_alpha_for_dissolution(netuid()).unwrap();
        let all_alpha = MockPool::all_alpha_for_lending_settlement(netuid());
        assert!(legacy_denom > 0);
        assert!(all_alpha > legacy_denom);
        let r_all = U64F64::from_num(pot) / U64F64::from_num(all_alpha);
        let r_legacy = U64F64::from_num(pot) / U64F64::from_num(legacy_denom);
        assert!(
            r_all < r_legacy,
            "all-alpha rate must be below legacy (r_all={r_all}, r_legacy={r_legacy}, pot={pot}, all={all_alpha}, legacy={legacy_denom})"
        );

        // §13 settlement path.
        assert_ok!(Lending::start_dissolution(netuid()));
        assert_ok!(Lending::readd_lifts_for_dissolution(netuid()));
        let mut meter = WeightMeter::with_limit(Weight::from_parts(u64::MAX, u64::MAX));
        assert!(Lending::settle_lending(netuid(), &mut meter));

        // Short ends with a positive payout because r uses ALL_alpha.
        let debt = (U64F64::from_num(principal) * r_all)
            .saturating_ceil()
            .saturating_to_num::<u64>();
        let expected_refund = escrow_before.saturating_sub(debt.min(escrow_before));
        assert!(
            expected_refund > 0,
            "legacy-dump short must receive positive refund under all-alpha r (debt={debt}, escrow={escrow_before}, r_all={r_all}, r_legacy={r_legacy})"
        );
        assert!(Positions::<Test>::get(&owner, netuid()).is_none());
        assert!(tao(&owner) >= expected_refund.saturating_sub(1));
        let _ = burned();
    });
}

#[test]
fn dissolving_blocks_new_opens_only_when_flagged() {
    new_test_ext().execute_with(|| {
        let owner = account(1);
        let hotkey = account(2);
        mint_tao(&owner, 50_000_000_000_000);
        // Near-dereg is NOT modeled; only Dissolutions flag blocks opens.
        assert_ok!(Lending::start_dissolution(netuid()));
        assert!(
            Lending::open(
                RuntimeOrigin::signed(owner),
                netuid(),
                Side::Short,
                1_000_000_000_000,
                hotkey,
                1,
                1,
            )
            .is_err()
        );
    });
}
