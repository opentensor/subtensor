# Implementation plan — covered continuous-unwind shorts

Companion to `DESIGN.md`. Goal: land the spec's **shorts-first** launch with the smallest faithful
diff, reusing `SubnetMovingPrice` (pEMA), the swap engine (fees/weights), recycle, and the dereg
hook. Long paths are written symmetric but flag-gated off.

Code is **not** written here — this is the build order, the exact files to touch, and the test/gate
plan.

---

## Phase 0 — scaffolding (no behavior change)

| File | Change | ~LOC |
|---|---|---|
| `pallets/subtensor/src/derivatives/mod.rs` | **new** module tree: `pub mod short; pub mod decay; pub mod settle; pub mod types;` | 10 |
| `pallets/subtensor/src/lib.rs` | `pub mod derivatives;`; add storage items (`ShortPositions`, `ShortAggregate`, governance `StorageValue`s + `type_value` defaults) | ~70 |
| `pallets/subtensor/src/derivatives/types.rs` | `ShortPosition`, `ShortAgg` structs (+ `freeze_struct`, derives) | ~40 |

No migration (new empty maps default cleanly; confirmed against repo convention). No
`STORAGE_VERSION` bump.

---

## Phase 1 — math core (pure, unit-testable, no extrinsics)

All in `derivatives/short.rs` as `impl<T: Config> Pallet<T>` helpers. These are the spec
closed-forms (Appendix A.1) used for quoting/sizing.

| Function | Spec | Returns |
|---|---|---|
| `short_t_ref(netuid)` | §3.1, §4 | `min(SubnetTAO, pEMA·A_live)` |
| `solve_collateral(p, t_ref, lambda, s)` | §4.2 | `(C, N)` via quadratic; reject `N ≤ 0` |
| `lambda_eff(...)` | §4.1 | effective LTV; reject `≤ 0` |
| `solve_phi(n, t_live)` | §4.3 | `ϕ = (1 − √(1 − 4N/T))/2`; reject `4N > T` |
| `decay_factor_g(u)` | §6.2 | per-block `g` from `d_day(u)` |
| `materialize(pos, agg)` | §6.3 | `f = exp(-(Ω−Ω_entry))`, scale `r,e,b` |

Each gets a focused unit test asserting the spec's worked examples (§1.7–1.8: `C=100`, `N=37.5`,
`ϕ≈0.039`, `Q≈3900`, `E=39`).

---

## Phase 2 — reserve legs (the risky part, isolate + test)

`derivatives/settle.rs`: the three pool-touching primitives, each a thin wrapper over existing
reserve helpers + the swap engine.

| Function | Net reserve effect | Built from |
|---|---|---|
| `open_remove_sell_back(netuid, n, e, q)` | `SubnetTAO -= (N+E)`; book `Q` debt | `decrease_provided_tao_reserve`; engine quote to confirm realized `N` |
| `restoration_zap(netuid, dU)` | `SubnetTAO += dU` (price drifts up) | `increase_provided_tao_reserve` (escalate to min-swap form only if sim demands) |
| `settlement_zap(netuid, alpha_in, tao_in)` | balanced add of repaid `Q` + escrow | engine min-swap (§8.5) + `increase_provided_*` |

**Gate for this phase:** the §3.5 conservation test — open → N blocks of decay → close (or default)
returns exactly the TAO removed plus posted floor, minus equity. Run on a Balancer pool with
non-default weights, not just 0.5/0.5.

---

## Phase 3 — extrinsics

| File | Change | ~LOC |
|---|---|---|
| `derivatives/short.rs` | `do_open_short`, `do_top_up_short`, `do_close_short`, `do_default_short` (ensure_signed, validate, materialize, mutate, emit) | ~220 |
| `macros/dispatches.rs` | 4 thin wrappers, `call_index` 139–142, placeholder `DbWeight` weights | ~50 |
| `macros/events.rs` | 5 event variants | ~25 |
| `macros/errors.rs` | ~8 error variants | ~12 |

Validation order in `do_open_short` (spec §8.1): side flag → `SubnetMechanism==1` → solve `C,N` →
reject `N≤0` / `4N>T` / `S+B>κ_S·T_ref` → solve `ϕ,Q,E` → realize legs → store/merge → bump
aggregate. Same-block stacked opens read the progressively updated `b_sigma` (spec §5.2.1) for free,
because each open re-reads `ShortAggregate`.

---

## Phase 4 — per-block decay hook

| File | Change | ~LOC |
|---|---|---|
| `derivatives/decay.rs` | `run_derivatives_decay()` — iterate subnets with `b_sigma>0`, O(1) tick each (§6.4), call `restoration_zap` | ~70 |
| `coinbase/block_step.rs` | one call after `run_coinbase`, before `update_moving_prices` | ~2 |

---

## Phase 5 — terminal dereg settlement

| File | Change | ~LOC |
|---|---|---|
| `derivatives/settle.rs` | `settle_shorts_on_dereg(netuid)` — for each short: materialize, `K_D=max(K_spot,last, Q·pEMA)`, pay `equity`, `recycle_tao(liability_cover)`, extinguish `Q`, clear | ~90 |
| `coinbase/root.rs` (`do_dissolve_network`) | call `settle_shorts_on_dereg(netuid)` before `destroy_alpha_in_out_stakes` | ~2 |

`K_spot,last(Q)` = `sim_swap(GetAlphaForTao, …)` cost to buy `Q` at the final executable state;
`pEMA` = `get_moving_alpha_price`. Buckets stay disjoint (liability-cover recycled outside terminal
distribution — same rule as default), so no terminal fixed-point (spec §11.3).

---

## Phase 6 — runtime API

| File | Change | ~LOC |
|---|---|---|
| `rpc_info/derivatives_info.rs` | **new** `ShortOpenQuote`, `ShortPositionInfo` DTOs + `quote_open_short`, `get_short_position` | ~110 |
| `rpc_info/mod.rs` | `pub mod derivatives_info;` | 1 |
| `runtime-api/src/lib.rs` | new trait `DerivativesRuntimeApi` (2 methods) + DTO imports | ~20 |
| `runtime/src/lib.rs` | `impl DerivativesRuntimeApi for Runtime` in `impl_runtime_apis!` | ~12 |

JSON-RPC (`pallets/subtensor/rpc`, `node/src/rpc.rs`) only if external clients need it — deferred.

---

## Phase 7 — governance wiring

| File | Change | ~LOC |
|---|---|---|
| `utils/misc.rs` | `set_*` for each param (put + event), `get_*` readers | ~60 |
| `admin-utils/src/lib.rs` | sudo/owner extrinsics: `sudo_set_shorts_enabled`, `…_short_kappa`, `…_short_base_ltv`, `…_decay_bounds`, `…_short_dust` | ~90 |

`ShortsEnabled` stays `false` until the trading-games gate passes.

---

## Phase 8 — tests & trading-games gate (spec §14.5)

`pallets/subtensor/src/tests/derivatives.rs` (+ eco-tests for adversarial sims). The spec makes these
the launch gate, not optional:

1. **Conservation** (§3.5) on weighted pools.
2. **Same-block stacked opens** cannot bypass `S+B ≤ κ_S·T_ref` (§5.2.1).
3. **Worked examples** (§1.7–1.8, §15) reproduce exactly.
4. **Dust/escrow bound** `E/R ≤ 1/(1−ϕ_cap)` holds through top-ups/partials (§7.3).
5. **Short-driven dereg**: no free terminal extraction; payout bounded by `K_D(Q)` (§10.7).
6. **Flow neutrality**: assert `SubnetTaoFlow` unchanged across every derivative leg (§4.5).
7. **Decay schedule**: 365-day remaining-fraction table (§14.3) within tolerance.

Only after 1–7 pass on a mainnet-like replica does governance flip `ShortsEnabled` and begin ramping
`κ_S` (spec §5.1, §14.6).

---

## Diff estimate

| Area | Files touched | New files | ~LOC |
|---|---|---|---|
| Storage + types | `lib.rs` | `derivatives/{mod,types}.rs` | ~120 |
| Math core | — | `derivatives/short.rs` (part) | ~120 |
| Reserve legs | — | `derivatives/settle.rs` (part) | ~140 |
| Extrinsics + FRAME surface | `dispatches.rs`, `events.rs`, `errors.rs` | — | ~90 |
| Decay hook | `coinbase/block_step.rs` | `derivatives/decay.rs` | ~72 |
| Dereg hook | `coinbase/root.rs` | — | ~92 |
| Runtime API | `runtime-api/src/lib.rs`, `runtime/src/lib.rs`, `rpc_info/mod.rs` | `rpc_info/derivatives_info.rs` | ~143 |
| Governance | `utils/misc.rs`, `admin-utils/src/lib.rs` | — | ~150 |
| **Total (excl. tests)** | **~10 edited** | **~6 new** | **~1,000** |

No on-chain migration. No `STORAGE_VERSION` bump. Reuses pEMA, swap engine, recycle, and dereg
plumbing rather than re-implementing them — which is where the line-count is kept down.

---

## Build / sanity commands

```bash
# compile the pallet only (fast loop)
cargo check -p pallet-subtensor

# pallet tests
cargo test -p pallet-subtensor derivatives

# full runtime build (after runtime-api wiring)
cargo check -p node-subtensor-runtime
```

## Open decisions for the author

1. **Position granularity**: merged-per-`(coldkey,netuid)` (chosen, minimal) vs. multi-position with
   an id index. Merge is spec-sanctioned (§8.6); revisit only if UX needs distinct lots.
2. **Restoration realization**: net `SubnetTAO +=` (chosen) vs. explicit min-swap zap. Start with the
   net form; escalate only if the conservation test on weighted pools fails.
3. **`hotkey` association**: carry it for identity/precompile parity, or drop it and key purely on
   coldkey. Carrying it is cheap and keeps consistency with staking.
