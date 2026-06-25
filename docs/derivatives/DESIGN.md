# Covered continuous-unwind derivatives — subtensor design

Implementation design for the **Fixed-Liability Covered Continuous-Unwind Model v3.6.1**
(`shorting.pdf`) inside `pallet-subtensor`. This document maps the spec onto the existing
runtime, fixes the reserve-accounting model against the real AMM, and locks the storage,
extrinsic, hook, and runtime-API surface. The companion `IMPLEMENTATION_PLAN.md` has the
phased file-by-file plan and diff estimate.

Launch scope is **shorts only**. Long paths are specified for symmetry but gated behind a
disabled flag (spec §1, §9.3). Everything below is written to add the *fewest* moving parts
by reusing primitives that already exist.

---

## 1. Reality check: what the spec assumes vs. what subtensor has

The spec is written against a **pure no-fee CPMM** (`x·y=k`). Subtensor's pool is different,
and that single fact drives most of the design decisions.

| Spec assumption | Subtensor reality | Consequence |
|---|---|---|
| Pure `x·y=k` | **Balancer-weighted** pool (`pallet_subtensor_swap`), weights in `SwapBalancer`, default 0.5/0.5 (CPMM-like only at init) | Use spec closed-forms **only for quoting/sizing**; realize every pool-touching leg through the live fee+weight-aware engine (`SwapHandler::sim_swap` / `swap`). The spec explicitly allows this (§4.4, §14.6). |
| User can remove/add liquidity | User LP is **deprecated** (`add_liquidity`/`remove_liquidity` → `Error::Deprecated`) | The "remove-and-sell-back" open and the restoration/settlement zaps are realized as **protocol reserve mutations**, not user LP ops. |
| Reserves `T`, `A` | `SubnetTAO` (TAO, the quote reserve), `SubnetAlphaIn` (alpha pool reserve), `SubnetAlphaOut` (staked alpha outside the pool) | Short open/restore are mostly `SubnetTAO` mutations; close settlement touches `SubnetAlphaIn`. |
| `pEMA` price reference | **Already exists**: `SubnetMovingPrice` (per-block halving EMA, TAO/alpha) | Reuse directly as the spec's `pEMA`. No new TWAP, no new price EMA. |
| `A_EMA` reserve EMA | **Add `SubnetAlphaInMovingReserve`** | One per-block EMA of `SubnetAlphaIn`, ticked on the price EMA's smoothing. `T_EMA = pEMA·A_EMA` (short), `A_EMA` used directly (long). Lagged ⇒ not in-block manipulable (see §4). |
| Recycle floor `P`, extinguish liability | **Already exists**: `recycle_tao(coldkey, amount)`, `recycle_subnet_alpha`/`burn_subnet_alpha` | Reuse for default and terminal settlement. |
| Per-block decay/unwind step | **Net-new** | One O(1)-per-subnet call added to `block_step()`. |
| Subnet deregistration hook | **Already exists**: `do_dissolve_network` (`coinbase/root.rs`) | Insert terminal derivative settlement before `destroy_alpha_in_out_stakes`. |
| Derivative flow-neutral for emissions | **Free by construction** | We mutate reserves directly and never call `record_tao_inflow/outflow`, so TaoFlow is untouched (spec §4.5). |

**Key takeaway:** the spec's `pEMA`, recycle, and dereg primitives already exist. The genuinely
new state is (a) the position store, (b) per-side aggregate + decay accumulator, (c) a per-block
decay step, (d) ~4 extrinsics, (e) one runtime-API quote, (f) one stored reserve EMA
(`SubnetAlphaInMovingReserve`) backing both risk references (§4).

---

## 2. Notation map (spec symbol → subtensor identifier)

| Spec | Meaning | Subtensor binding |
|---|---|---|
| `T` | live TAO reserve | `SubnetTAO::<T>::get(netuid)` |
| `A` | live alpha reserve | `SubnetAlphaIn::<T>::get(netuid)` |
| `T_ref` | conservative TAO ref `min(T_live, T_EMA)` | `min(SubnetTAO, pEMA·A_EMA)` — `A_EMA` lagged (§4) |
| `pEMA` | EMA price (TAO/alpha) | `Pallet::get_moving_alpha_price(netuid)` (`SubnetMovingPrice`) |
| `P` | user position input / floor | `ShortPosition.p_floor: TaoBalance` |
| `C` | gross collateral (open-time only) | computed, **not stored** |
| `N` | retained proceeds = `R0` | computed at open → `r_stored` |
| `R(t)` | retained buffer (decays) | `ShortPosition.r_stored` × decay factor |
| `Q` | fixed alpha liability | `ShortPosition.q_liability: AlphaBalance` |
| `E(t)` | linked TAO escrow (decays) | `ShortPosition.e_stored: TaoBalance` |
| `B` | utilization footprint `λC` (TAO) | `ShortPosition.b_stored: TaoBalance` |
| `S` | aggregate active footprint | `ShortAgg.b_sigma` |
| `Ω_S` | short decay accumulator | `ShortAgg.omega: U64F64` |
| `Ω_entry` | per-position accumulator snapshot | `ShortPosition.omega_entry: U64F64` |
| `λ`, `λ_eff` | base / effective LTV | governance param `ShortBaseLtv`; `λ_eff` computed |
| `κ_S` | short footprint cap factor | governance param `ShortKappa` |
| `d_min`,`d_max` | decay bounds | `DecayMin`, `DecayMax` |
| `R_dust` | dust threshold | `ShortDust` |
| `K_D(Q)` | terminal liability value | computed at dereg: `max(K_spot,last, Q·pEMA)` |

---

## 3. Reserve-accounting model (the load-bearing part)

All pool impact is expressed as mutations to `SubnetTAO` / `SubnetAlphaIn`, executed through the
existing helpers so weights and fees stay consistent:

- `increase_provided_tao_reserve` / `decrease_provided_tao_reserve`
- `increase_provided_alpha_reserve` / `decrease_provided_alpha_reserve`
- `T::SwapInterface::sim_swap` / `swap` with `GetAlphaForTao<T>` / `GetTaoForAlpha<T>` for any
  internal swap leg (fee + weight aware).

### 3.1 Open short — net pool effect

The spec's remove-and-sell-back (§4.3) on a pure CPMM nets to: **alpha reserve unchanged, TAO
reserve drops by `N + E`**, leaving the trader owing `Q = ϕA` alpha. We realize that directly:

```
TAO removed from pool = N + E = ϕ(2-ϕ)·T      // = T - (1-ϕ)²T on pure CPMM
SubnetTAO            -= (N + E)                 // the downward price impact
held by protocol      = E (escrow) + N (becomes buffer R0)
position liability     = Q = ϕ·A (alpha debt, virtual; alpha reserve untouched at open)
```

`ϕ`, `N`, `Q`, `E` are first quoted from the spec closed-forms (Appendix A.1), then the realized
TAO leg is taken from a fee-adjusted engine quote so the booked `N`/`E` match what the pool
actually moved. The trader supplies `P = C − N` TAO, held against the floor and recycle-on-default.

### 3.2 Continuous restoration (per block) — net pool effect

For a short the decayed amount `dU = dR + dE` is TAO-side. The spec zap (swap min portion to
alpha, re-add balanced) nets, on a CPMM, to **alpha unchanged, TAO `+= dU`, price drifts up** —
exactly reversing the open impact over time:

```
restoration_zap(netuid, dU)  ≡  increase_provided_tao_reserve(netuid, dU)
```

No weight change is needed (we *want* the upward drift), so this is a single reserve increment.
This conserves TAO: the `N + E` removed at open is returned over the position's life. (If
simulation later shows the weighted pool needs the explicit min-swap, swap `z = √(T(T+U)) − T`
via the engine then add the remainder — spec §6.6 — behind the same `restoration_zap` fn.)

### 3.3 Close (partial fraction ρ, full = ρ=1) — net pool effect

Trader repays `ρQ` alpha; protocol pairs it with the escrow slice `ρE` via the settlement zap
(§8.5). Net pool effect: `SubnetAlphaIn += ρQ`, `SubnetTAO += ρ·E_remaining_share`, balanced
through an engine min-swap. Trader receives `ρ(P + R)` back. Position `P, Q, R, E, B` reduced
pro-rata; aggregates updated.

### 3.4 Default (R ≤ R_dust) and terminal dereg

- **Default:** restore residual `R + E` (restoration zap), `recycle_tao(coldkey, P)` for the floor,
  extinguish `Q` (no alpha moves — it was virtual), drop position from aggregates.
- **Dereg terminal:** value liability at `K_D(Q) = max(K_spot,last(Q), Q·pEMA)`; equity =
  `max(0, (P+R) − K_D)` paid to trader; `min(P+R, K_D)` recycled via `recycle_tao` outside terminal
  distribution; `Q` extinguished. Hooked into `do_dissolve_network` before `destroy_alpha_in_out_stakes`.

### 3.5 Conservation invariant (must be a test)

Over any position lifecycle, total TAO returned to `SubnetTAO` via restoration + close-settlement +
default-restore, plus recycled floor/liability-cover, **equals** the `N + E` removed at open plus the
`P` the trader posted, minus equity paid out. This invariant is the acceptance gate for the
reserve math and is the first item in the spec's trading-games suite (§14.5).

> **Primary implementation risk:** reconciling the spec's CPMM closed-forms with the Balancer
> weights. Mitigation: quote/size from closed-forms, realize from the engine, gate launch on the
> conservation + capacity simulations the spec already mandates (§14.5). `κ_S` starts tiny.

---

## 4. Risk reference reserves from a block-lagged reserve EMA

The spec wants `T_ref = min(T_live, T_EMA)` to stop a same-block reserve pump from improving open
terms (§3.1–3.2). The reference must be built from **block-lagged** factors only: anything read live
within the extrinsic can be moved by the caller's own swap in the same block.

We maintain one stored reserve EMA — `SubnetAlphaInMovingReserve` (`A_EMA`), a per-block EMA of
`SubnetAlphaIn` — ticked on the **same smoothing** as the price EMA inside `update_moving_price`
(one extra storage write per subnet per block). Both derivative sides derive their reference from it,
combined with the existing `pEMA` (`SubnetMovingPrice`):

```
T_EMA  =  pEMA · A_EMA            (lagged price × lagged alpha reserve ⇒ lagged TAO depth)
T_ref  =  min(SubnetTAO, T_EMA)   (short side)
A_ref  =  min(SubnetAlphaIn, A_EMA)   (long side — uses A_EMA directly as alpha depth)
```

The `T_EMA` / `A_EMA` upper bound is now a pure function of lagged state, so an in-block reserve nudge
**cannot raise it**; the live `T_live` / `A_live` term only ever pulls the `min` *down* (conservative,
and self-defeating for an attacker). This closes the crossover attack on the earlier `pEMA·A_live`
reference: under the CPMM invariant `T_live·A_live = k`, `min(T_live, pEMA·A_live)` had one increasing
and one decreasing branch in `A_live` and therefore peaked at the crossover `A* = √(k/pEMA)` (the
geometric mean `√(T_live·T_EMA)`), letting a sandwich inflate `T_ref`, retained proceeds `N`, and the
footprint cap `S + B ≤ κ_S·T_ref` for one block — a breach that persisted because the cap is checked
only at open. Decay utilization uses the same lagged `T_ref` (spec §3.3), so flash trades cannot grief
carry either.

EMA manipulation is bounded exactly as the price EMA already is: biasing `A_EMA` requires holding the
reserve displaced across the once-per-block sample (post-coinbase), forfeiting atomicity and moving it
only by the small smoothing fraction per block.

A cold `A_EMA` (`0`, e.g. a freshly created subnet) makes the reference fall back to the live reserve
until it warms. On a live-chain upgrade, `migrate_seed_alpha_in_moving_reserve` seeds `A_EMA` from the
current `SubnetAlphaIn` per subnet so there is no cold-start window; the per-block tick carries it
forward from there.

---

## 5. Storage layout

New module `pallets/subtensor/src/derivatives/`. Storage declared inline in `lib.rs` (the repo's
convention — there is no storage macro file). `#[pallet::without_storage_info]` is already set, so
`MaxEncodedLen` is not required.

### 5.1 Position struct

One **merged** short position per `(coldkey, netuid)` — additional same-side opens merge after
materialization (spec §8.6), which keeps the store sparse and avoids a position-id index.

```rust
#[freeze_struct("<hash>")]
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ShortPosition {
    pub p_floor: TaoBalance,     // non-decaying floor (spec P)
    pub q_liability: AlphaBalance,// fixed alpha debt (spec Q)
    pub r_stored: TaoBalance,    // buffer at last materialization (spec R)
    pub e_stored: TaoBalance,    // escrow at last materialization (spec E)
    pub b_stored: TaoBalance,    // footprint at last materialization (spec B)
    pub omega_entry: U64F64,     // Ω_S snapshot at last materialization
    pub opened_at: u64,          // block, for UX/telemetry only
}
```

```rust
// --- DMAP (netuid, coldkey) -> ShortPosition
#[pallet::storage]
pub type ShortPositions<T: Config> = StorageDoubleMap<
    _, Identity, NetUid, Blake2_128Concat, T::AccountId, ShortPosition, OptionQuery>;
```

### 5.2 Per-subnet aggregate + decay accumulator

```rust
#[freeze_struct("<hash>")]
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, Clone, PartialEq, Eq, Debug, Default)]
pub struct ShortAgg {
    pub r_sigma: TaoBalance,   // Σ current R
    pub e_sigma: TaoBalance,   // Σ current E
    pub b_sigma: TaoBalance,   // Σ current B  == active footprint S
    pub q_sigma: AlphaBalance, // Σ fixed liability (open interest)
    pub omega: U64F64,         // Ω_S cumulative decay accumulator
}

#[pallet::storage]
pub type ShortAggregate<T: Config> =
    StorageMap<_, Identity, NetUid, ShortAgg, ValueQuery, DefaultShortAgg<T>>;
```

Materialization (spec §6.3): `f = exp(-(Ω - Ω_entry))`, multiply `r,e,b` by `f`, snapshot `Ω_entry = Ω`.
Aggregate tick is O(1) per subnet: `R,E,B *= g`, `Ω += -ln g` (spec §6.4).

### 5.3 Governance parameters (global defaults; per-subnet override optional later)

Stored as `StorageValue` with `#[pallet::type_value]` defaults; setters in `utils/misc.rs`; exposed
via `pallet-admin-utils` sudo/owner extrinsics (existing pattern).

| Storage | Type | Default | Spec |
|---|---|---|---|
| `ShortsEnabled` | `bool` | `false` (flip on after games) | §14.1 |
| `LongsEnabled` | `bool` | `false` | §9.3 |
| `ShortBaseLtv` | `U64F64` | `0.50` | §14.1 |
| `ShortKappa` | `U64F64` | small, conservative | §5.1 |
| `DecayMin` | `U64F64` | `0.001`/day | §6.2 |
| `DecayMax` | `U64F64` | `0.015`/day | §6.2 |
| `ShortDust` | `TaoBalance` | `1 TAO` | §7.2 |

No migration is required: new maps default cleanly; only `ShortsEnabled` flips via governance.

---

## 6. Per-block decay step

Add `Self::run_derivatives_decay()` to `block_step()` **after** `run_coinbase(...)` and **before**
`update_moving_prices()` (so decay sees post-emission reserves but feeds the same block's price EMA).
For each subnet with `ShortAggregate.b_sigma > 0`:

```
u      = min(1, b_sigma / (ShortKappa · T_ref))           // EMA-smoothed via T_ref
d_day  = DecayMin + (DecayMax - DecayMin)·u²
g      = (1 - d_day)^(1 block / blocks_per_day)           // const per-block factor
dR = r_sigma·(1-g);  dE = e_sigma·(1-g);  dB = b_sigma·(1-g)
r_sigma,e_sigma,b_sigma *= g;  omega += -ln g
restoration_zap(netuid, dR + dE)                          // SubnetTAO += dR+dE
```

O(1) per active subnet, no per-position iteration. `(1-d_day)^(1/blocks_per_day)` is computed with
the existing `substrate_fixed` helpers; `blocks_per_day ≈ 7200`.

**Defaults are lazy.** Because the tick never visits individual positions, a position that has decayed
below `R_dust` is settled (a) on its owner's next interaction (materialize → if dust, default), or
(b) by a permissionless `default_short(coldkey, netuid)` poke. This keeps the block hook O(1) and
matches the spec's MEV-insensitive, time-based default (§7.1, §7.4).

---

## 7. Extrinsics (shorts launch)

Thin dispatch wrappers in `macros/dispatches.rs` → `do_*` in `derivatives/`. Next free
`call_index` is **139**.

| call_index | Extrinsic | Delegates to | Notes |
|---|---|---|---|
| 139 | `open_short(netuid, hotkey, position_input: TaoBalance, price_limit: TaoBalance)` | `do_open_short` | gated by `ShortsEnabled`; solves `C,N,ϕ,Q,E`; capacity + domain checks; merges into existing position |
| 140 | `top_up_short(netuid, amount: TaoBalance)` | `do_top_up_short` | adds to `R` only (spec §8.2); fresh decaying capital |
| 141 | `close_short(netuid, fraction: U64F64, price_limit: TaoBalance)` | `do_close_short` | partial (`ρ<1`) and full (`ρ=1`); repays `ρQ`, returns `ρ(P+R)` |
| 142 | `default_short(coldkey, netuid)` | `do_default_short` | permissionless; only valid when materialized `R ≤ R_dust` |

`hotkey` is carried so the position is associated with a `(hotkey, coldkey, netuid)` identity
consistent with the rest of staking, even though the merged position is keyed `(netuid, coldkey)`.
Long extrinsics are **not** added at launch (gated by spec §9; adding them later is symmetric).

Weights: start with inline `DbWeight::get().reads_writes(r, w)` placeholders (an accepted in-repo
pattern), benchmark before mainnet.

---

## 8. Events & errors

**Events** (`macros/events.rs`): `ShortOpened { netuid, coldkey, p, n, q, e, phi }`,
`ShortToppedUp`, `ShortClosed { netuid, coldkey, fraction, repaid_q, returned }`,
`ShortDefaulted`, `ShortTerminalSettled { netuid, coldkey, equity, liability_cover }`.

**Errors** (`macros/errors.rs`): `ShortsDisabled`, `ShortPositionNotFound`,
`EffectiveLtvNonPositive` (`λ_eff ≤ 0`), `RetainedProceedsNonPositive` (`N ≤ 0`),
`ShortCapacityExceeded` (`S + B > κ_S·T_ref`), `ReserveDomainExceeded` (`4N > T_live`),
`PositionNotDefaultEligible`, `SubnetNotDynamic` (mechanism ≠ 1 / root).

---

## 9. Runtime API (read-only quote)

Extend `runtime-api/src/lib.rs` + `rpc_info/` + `impl_runtime_apis!` (runtime/src/lib.rs).

```rust
fn quote_open_short(netuid: NetUid, position_input: TaoBalance) -> ShortOpenQuote;
fn get_short_position(coldkey: AccountId32, netuid: NetUid) -> Option<ShortPositionInfo>;
```

`ShortOpenQuote` carries the spec's pre-open trader view (§1.2): `c, n, q, e, phi, lambda_eff,
daily_decay, min/max_time_to_dust, est_close_cost (via sim_swap GetAlphaForTao for Q),
breakeven_close_price`. Pure reads + `sim_swap`; no state change. JSON-RPC wrapper is optional.

---

## 10. Invariants enforced (spec §17)

1. Shorts-first: `open_short` rejects unless `ShortsEnabled`; longs gated.
2. Covered: `P + N = C` at open.
3. No liquid proceeds: `N` is never paid out; it becomes `R0`.
4. Fixed liability: `Q` changes only on close / default / dereg.
5. Continuous unwind: `R,E,B` decay with one `g`; restored via `restoration_zap`.
6. No price-based liquidation: default iff `R ≤ R_dust`.
7. Limited recourse: residual `Q` extinguished at default/dereg.
8. Footprint cap: `S + B ≤ κ_S·T_ref` (also bounds same-block stacked opens via progressive `S`).
9. Flow neutrality: no `record_tao_*` calls on any derivative leg.
10. Dereg awareness: terminal alpha base read from subnet mode (legacy vs new, per `destroy_alpha_in_out_stakes` rules).
11. Terminal short settlement: `K_D(Q) = max(K_spot,last, Q·pEMA)`.
12. Escrow bound: `E/R = 1/(1−ϕ)` stays bounded by `κ_S`-implied `ϕ_cap`, so dust default is MEV-trivial.

---

## 11. Explicit deferrals (faithful to spec)

- **Longs**: code-symmetric but flag-gated off (`LongsEnabled=false`). Long open mirrors with
  alpha/TAO swapped, `D=ϕT`, ADR-adjusted LTV (§9.2). Not in the launch diff.
- **Derivative TaoFlow** (`χ_S`): off; flow-neutral (§4.5). Not wired.
- **Reserve EMA**: one stored EMA (`SubnetAlphaInMovingReserve`, `A_EMA`) backs both references
  (§4); `T_EMA = pEMA·A_EMA`. A separate stored `T_EMA` / TWAP is not needed and remains an optional
  later guard only (§3.4, §11.4).
- **Per-open `ϕ_max`**: not a control; only the `4N ≤ T_live` domain bound is enforced (§5.2).
- **Per-subnet param overrides**: launch uses globals; per-netuid maps can be added later without
  touching call sites.
