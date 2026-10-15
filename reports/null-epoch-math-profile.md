---
title: "Null consensus epoch math profile"
description: "Phase breakdown of Null epoch CPU and storage at 2.5k–32k UIDs; what is O(N), what Null can skip, and whether 16k/32k fit today's 2.5k cost."
---

# Null consensus epoch math profile

**Scope.** Analysis and estimates only. `NULL_UID_BUDGET` stays **2,500**. No production code changes in this report.

**Question.** At 2.5k / 8k / 16k / 32k UIDs, where does Null `epoch_mechanism` + persist spend time and storage reads/writes? Which work is inherently O(N)? What can Null skip? Are **16k** or **32k** feasible at today's **2.5k** epoch cost, and with which changes?

**Method.** Release `pallet-subtensor` ignored test `null_epoch_phase_profile` on this branch. Fixture: Null subnet, one permit holder, one dense weight row of length N, ~64 staked hotkeys (rest zero stake), claim-based flag on (does not change epoch math). Wall times are local-machine relative; ratios matter more than absolute milliseconds. Persist timing is in-memory mock DB — on-chain RocksDB + proof-of-validity (PoV) cost is higher than the wall numbers below.

```bash
cargo test -p pallet-subtensor --release null_epoch_phase_profile \
  -- --ignored --nocapture
```

## Measured phase times (release)

| UIDs | Keys→map ms | LastUpdate ms | BlockAtReg ms | Null stake ms | Weight row ms | **Epoch total ms** | Persist ms | Vote ms | Epoch+persist ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2,500 | 2.11 | ~0 | 1.24 | **6.97** | 0.03 | **11.3** | 0.07 | ~0 | **11.4** |
| 8,000 | 7.46 | ~0 | 5.40 | **23.9** | 0.10 | **38.2** | 0.22 | ~0 | **38.4** |
| 16,000 | 15.1 | ~0 | 11.7 | **47.9** | 0.20 | **80.6** | 0.53 | ~0 | **81.1** |
| 32,000 | 30.7 | ~0 | 25.5 | **95.9** | 0.40 | **164** | 1.49 | ~0 | **165** |

**Share of epoch at 2.5k (from separately timed phases):**

| Phase | ≈ % of epoch | What it does |
| --- | ---: | --- |
| Null stake (`get_null_stake_weights_for_network`) | **~61%** | Per-UID `Keys` lookup + alpha/root stake + parent/child inheritance for **every** UID |
| Keys → `BTreeMap` of `EpochTerms` | **~19%** | Build hotkey→terms for all N UIDs |
| `BlockAtRegistration` vector | **~11%** | N× (`Keys::contains_key` + map get) |
| Remaining (masks, normalize, apportion, populate terms) | **~9%** | Fixed-point + payout math over length-N vectors |
| Winner weight row read | **≪1%** | One `Weights` row (length N in this fixture) |
| Persist (mock wall) | **≪1%** | Dense vector inserts + event (understates chain write/PoV) |
| Voting power | **~0** | Tracking off in fixture |

**Scaling.** All dominant phases grow ~linearly with N. Ratio 32k/2.5k = 12.8× UIDs; epoch wall ≈ **14.5×** (mild superlinearity from `BTreeMap` + vector fixed-point).

Companion capacity report (`null-epoch-capacity-claim-based.md`): claim-based cuts **distribute** ~3.6×; **epoch math is unchanged** by the claim toggle.

## Code path (Null branch of `epoch_mechanism`)

Already different from Yuma:

| Work | Null today | Notes |
| --- | --- | --- |
| Bonds load / EMA / persist | **Skipped** | `persist_mechanism` only writes bonds when `EpochConsensus::Yuma` |
| Rank / Trust maps | **Gone** | Deprecated getters return empty; not written in epoch |
| PruningScores | **Gone** | Deprecated; Null prune uses batch trim (`NULL_PRUNING_BATCH`), not epoch scores |
| Full weight matrix | **Skipped** | Only elected validator's row loaded into an N-row sparse shell |
| Consensus median / clip / matmul ranks | **Skipped** | Consensus + validator_trust filled with **zeros**; incentive from weight integers; dividends from raw stake bits |
| Row-normalize weights | **Skipped** | Null keeps integer ratios for `apportion_units` |

Still shared / still O(N) on Null:

1. **Keys prefix → `BTreeMap<AccountId, EpochTerms>`** — one entry per UID.
2. **LastUpdate** — one dense `Vec<u64>` read (cheap); builds inactive/active masks O(N).
3. **BlockAtRegistration** — N map reads every epoch (used to mask outdated weight columns).
4. **Null stake** — O(N) hotkeys + O(N) stake/parent/child even when most stakes are zero.
5. **Winner scan** — O(N) over stake vector (`null_validator_winner`).
6. **Weight masks** — diagonal + registration (+ commit-reveal if enabled) over the winner row; row length often N.
7. **Dividend + incentive report vectors** — normalize full length-N fixed-point vectors for **display**; comment in code: displayed proportions never feed back into Null emission amounts. Actual rao payouts come from `apportion_units` on integer shares.
8. **Populate every `EpochTerms`** — copy incentive/dividend/emission/permit/… for all N entries.
9. **Persist** — rewrite dense maps every epoch (see below).
10. **Event** — `IncentiveAlphaEmittedToMiners { emissions: Vec }` length N.

## Storage reads / writes per Null epoch

### Reads (typical Null, CR off)

| Item | Pattern | Count / size |
| --- | --- | --- |
| `Keys` | Double-map prefix / per-UID | **N** (terms_map) + **N** (stake hotkeys) + **N** (block-at-reg contains) ≈ **3N** key touches |
| Stake / children / parents | Per hotkey | **N** alpha + **N** root stake loads (+ parent extras if any) |
| `LastUpdate` | One map value | **1** blob ≈ 8N bytes |
| `ValidatorPermit` | One map value | **1** blob ≈ N bytes |
| `Weights` | One double-map value | **1** row ≈ 4×(nonzero targets); fixture dense ⇒ ~4N bytes |
| `BlockAtRegistration` | Per-UID map | **N** reads |

`Rank` / `Trust` / `PruningScores` / `Bonds`: **not read** on Null epoch path.

### Writes (every Null epoch today)

| Storage | Type | Approx value bytes | Null content |
| --- | --- | --- | --- |
| `Incentive` | `Vec<PerU16>` | ~2N | Normalized display incentives |
| `ConsensusByMechanism` | `Vec<PerU16>` | ~2N | **All zeros** |
| `Active` | `Vec<bool>` | ~N | Activity mask |
| `Emission` | `Vec<AlphaBalance>` | ~8N | Combined emission |
| `Consensus` | `Vec<PerU16>` | ~2N | **All zeros** |
| `Dividends` | `Vec<PerU16>` | ~2N | Stake-normalized display |
| `ValidatorTrust` | `Vec<PerU16>` | ~2N | **All zeros** |
| `ValidatorPermit` | `Vec<bool>` | ~N | Single `true` |
| `StakeWeight` | `Vec<u16>` | ~2N | Normalized stake u16s |
| Event emissions | `Vec<AlphaBalance>` | ~8N | Miner server emissions |

**Dense rewrite ≈ 21N bytes of values/epoch** (plus SCALE/hash overhead), ignoring the event:

| N | ≈ dense vector bytes/epoch |
| ---: | ---: |
| 2,500 | ~53 KB |
| 8,000 | ~168 KB |
| 16,000 | ~336 KB |
| 32,000 | ~672 KB |

Plus the length-N emission event (~8N more). Mock persist wall stays &lt;2 ms even at 32k; **chain weight / PoV / DB amplification** is the real limiter for these rewrites, not the mock CPU.

`LastUpdate` is **read** in epoch, not rewritten by persist (updated on set-weights paths).

## O(N) vs Null-avoidable

### Inherently O(N) (or O(row length)) for correct Null semantics

- Elect sole validator when stake can sit on any UID (unless an index of nonzero stake is maintained).
- Apportion miner budget across weight targets when the winner row has Θ(N) nonzero entries.
- Apportion validator budget across Θ(N) nonzero stakers (rare on Null; usually ≪N).
- Identity of earners for payout / claim accrual (at least O(number of nonzero emissions)).

### Avoidable or shrinkable on Null

| Lever | Why safe / conditional for Null | Est. epoch CPU save | Est. storage/PoV save |
| --- | --- | --- | --- |
| **A. Sparse / indexed Null stake** | Most UIDs have zero stake; only nonzero (and parents) need full inheritance math. Winner = max among indexed stakers. | **~50–60%** of epoch wall (stake is ~61% today) | Fewer stake map reads: from ~2N toward ~2S (S = stakers ≪ N) |
| **B. Sparse epoch output (no full `BTreeMap`)** | Only emit terms for nonzero server/validator emission + the winner permit UID. Downstream distribute/claim already iterate earners. | **~15–20%** (Keys→map ~19%) | Less RAM; persist can stay dense unless paired with C |
| **C. Skip writing zero / display-only vectors** | Null already zeros Consensus + ValidatorTrust; display proportions do not feed payouts. Optionally stop rewriting `Consensus`, `ConsensusByMechanism`, `ValidatorTrust`, and maybe unused metagraph vectors every tempo. | Small CPU in mock; **large PoV** | **~40%** of dense vector bytes (~8N of ~21N) if those four are dropped; more if Incentive/Dividends become lazy RPC |
| **D. Sparse persist Emission / Incentive / event** | Write only changed or nonzero entries (map or sparse vec). Biggest win when weight row is sparse. Dense “weight everyone” still Θ(N). | Small–moderate CPU; **large PoV** when sparse | From ~21N toward O(nonzero); event from 8N → 8×nonzero |
| **E. Cache `BlockAtRegistration` as one vector** | Same pattern as `LastUpdate`; avoid N map probes per epoch. | **~10%** | N map reads → 1 blob read (~8N bytes once) |
| **F. Skip outdated-column mask when no recent regs / no CR** | If no UID registered since last tempo and CR off, mask is a no-op. | Up to ~10% if E not done | Avoid BlockAtReg touches that epoch |
| **G. Policy: sparse validator weight rows** | Null already loads one row; length N is a product choice. Caps / UX that discourage full-N rows cut apportion + event size. | Cuts the ~9% “remaining” and distribute work with row length | Weight blob + event scale with nonzero targets |
| **H. Bonds / Rank / Trust / PruningScores** | **Already skipped** on Null | **0% more** | **0** |

Fixed-point normalize of full display vectors (incentive/dividend reports) is **avoidable CPU** (~part of the ~9%) once payouts stay on `apportion_units` only.

## Feasibility: 16k / 32k at today's 2.5k epoch cost?

**Baseline (this harness):** 2.5k epoch ≈ **11.3 ms**.
**16k** ≈ **80.6 ms** ≈ **7.1×** today's epoch.
**32k** ≈ **164 ms** ≈ **14.5×** today's epoch.

### Without further epoch changes

**No.** Neither 16k nor 32k matches today's 2.5k epoch wall. Claim-based emissions help **distribute**, not this math (see capacity report: ~2× UID headroom on epoch+distribute combined ≈ **~5k**, not 16k).

### With concrete Null-oriented changes (estimates)

Stack toward today's 11.3 ms epoch budget:

| Step | 16k epoch after step (est.) | 32k epoch after step (est.) |
| --- | ---: | ---: |
| Today (no change) | ~81 ms | ~164 ms |
| + **A** sparse stake (~55% off) | ~36 ms | ~74 ms |
| + **B** sparse terms map (~18% off remaining) | ~30 ms | ~61 ms |
| + **E/F** BlockAtReg cache/skip (~10% off original ≈ 8 ms / 16 ms) | ~22 ms | ~45 ms |
| + **C/D/G** skip zero vectors + sparse persist/event + sparse weights | CPU → ~15–25 ms if row still dense; **PoV** can fall toward O(nonzero). If weight row and stakers stay sparse (e.g. hundreds of targets, tens of stakers), CPU can approach **≲2–3×** a optimized 2.5k | Still ~2× the 16k result |

**Verdict:**

- **16k at today's 2.5k epoch CPU:** **Not with storage-layout tweaks alone.** Needs **A + B + E** at minimum, and still lands ~**2×** today's 11 ms unless weight rows and stake are sparse (**G** + indexed stake). **Feasible as an engineering target** if you also accept either (1) a higher epoch budget than today's 2.5k, or (2) sparse weights + sparse stake index so remaining work is O(S + W) not O(N).
- **32k at today's 2.5k epoch CPU:** **Not realistic** while any per-epoch work remains Θ(N) over all UIDs (dense vectors, dense weight row, full Keys scan). Would need the epoch to become **O(stakers + weight nonzeros)** end-to-end (no dense N vectors in the hot path, no full Keys rebuild, no length-N event). That is a larger Null-specific epoch redesign, not a small skip list.
- **Storage/PoV at 16k/32k:** Even if CPU were fixed, rewriting ~336 KB / ~672 KB dense vectors + length-N events every tempo is a separate blocker; **C + D** are mandatory for chain weight long before 32k.

### Rough savings checklist (at 2.5k cost units)

| Change | Est. save vs current Null epoch | Unlocks |
| --- | --- | --- |
| A – sparse/indexed Null stake | ~0.55 × epoch | Required for any ≫5k story |
| B – sparse `EpochTerms` / skip zero UIDs | ~0.15–0.20 × epoch | Helps RAM + distribute handoff |
| E – vectorized BlockAtReg | ~0.10 × epoch | Cheap win |
| C – don't persist all-zero Consensus/VTrust | ~0.4 × dense write bytes | PoV / DB |
| D – sparse Emission/Incentive/event | ~0.5–0.9 × write bytes if sparse weights | PoV; weak if everyone is weighted |
| G – sparse weight rows | Scales remaining math with W not N | Needed for 16k≈2.5k CPU |
| Bonds / Rank / Trust / PruningScores | Already 0 | — |

**Combined optimistic CPU (A+B+E, dense weights retained):** ~**0.35–0.40×** current epoch → 16k still ~**2–2.5×** today's 2.5k epoch.
**Combined with sparse weights + sparse stake (A+B+E+G+D):** 16k **can** approach today's 2.5k epoch **if** nonzero targets and stakers stay far below N. 32k still needs “no Θ(N) hot path” as a hard design rule.

## Reproduce

```bash
cargo test -p pallet-subtensor --release null_epoch_phase_profile \
  -- --ignored --nocapture
```

Related: `reports/null-epoch-capacity-claim-based.md` (epoch + distribute + claim vs immediate).
