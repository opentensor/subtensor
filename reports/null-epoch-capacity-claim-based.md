---
title: "Null epoch capacity with claim-based miner emissions"
description: "Measured Null epoch cost at larger UID counts; how many UIDs match today's chain cost."
---

# Null epoch capacity with claim-based miner emissions

**UID cap unchanged.** `NULL_UID_BUDGET` remains **2,500**. This report only measures what larger caps would cost.

**Method.** Release `pallet-subtensor` test `null_epoch_capacity_report` (ignored; run with `--ignored --nocapture`). For each UID count it builds a Null subnet with one validator weight row of length N, then measures:

1. `epoch_mechanism` wall time (Null consensus math + winner row)
2. `distribute_dividends_and_incentives` wall time for N miner incentives
3. Cold storage key delta on first distribute (new map keys)

**Main vs this branch.** Main has no claim path: every miner incentive is share-pool staked every epoch. On this branch, `ClaimBasedMinerEmissions=false` is that same immediate-stake path; `true` is claim-based accrual. Epoch math is unchanged by the toggle.

**Environment.** Local release test binary on PR head `b2d6c388f` (CI green on that SHA; not reference hardware). Absolute milliseconds are machine-relative; ratios and the capacity conclusion are what matter.

## Results

| UIDs | Epoch ms (imm ≈ claim) | Distribute ms (immediate) | Distribute ms (claim) | Cold keys (immediate) | Cold keys (claim) | Total ms imm | Total ms claim |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2,500 (today's max) | 10.9 | 28.7 | 7.8 | 14,995 | 2,501 | **39.6** | 18.7 |
| 4,000 | 18.1 | 47.2 | 13.0 | 23,995 | 4,001 | 65.3 | 31.1 |
| 8,000 | 37.9 | 100.0 | 27.3 | 47,995 | 8,001 | 137.9 | 65.2 |
| 16,000 | 77.7 | 213.0 | 58.2 | 95,995 | 16,001 | 290.7 | 135.9 |
| 32,000 | 160.5 | 454.1 | 124.7 | 191,995 | 32,001 | 614.6 | 285.1 |

Steady-state distribute updates existing keys (key delta ≈ 0 after warmup). Cold keys show first-deposit storage creation: ~6 keys/UID for immediate stake vs ~1 pending row/UID (+ one subnet total) for claim-based.

Distribute speedup (claim vs immediate) is about **3.6–3.7×** at every size. Epoch time is **unchanged** by claim mode and scales roughly linearly with N.

## How many UIDs match today's epoch chain cost?

Today's Null max with immediate stake costs about **39.6 ms** total (epoch + distribute) in this harness.

With claim-based emissions, the same total is reached near **~5,000 UIDs** (interpolation between 4k = 31.1 ms and 8k = 65.2 ms).

So: **claim-based miner emissions support roughly ~2× today's Null UID count (~5k) at the same wall-clock epoch+payout cost as today's 2,500 with immediate stake.**

If you only match the **distribute** slice (28.7 ms today), claim-based stays under that budget past **8k** (27.3 ms) and crosses it before **16k** (58.2 ms) — about **~3.5×** on the payout path alone. Epoch math still grows with N, so the combined budget is the tighter limit.

## Next bottlenecks (after claim-based)

Claim-based removes the per-miner share-pool write storm. What still scales with N:

1. **Null epoch vectors** — `Active`, `Emission`, `Consensus`, `Dividends`, `LastUpdate`, `ValidatorPermit`, stake weight vectors: O(N) reads/writes every epoch.
2. **Winner weight row** — Null loads one validator row, but that row is length N (and CRV3 ciphertext grows with it).
3. **Per-UID identity maps** — `Keys`, `Uids`, `Owner`, `IsNetworkMember`, `BlockAtRegistration` (registration / trim / metagraph, not every epoch write, but state size and proof size).
4. **`PendingMinerAlpha` growth** — one row per earner who has not claimed; dissolve/trim must force-pay these.
5. **Claim extrinsic load** — if every miner claims every tempo, you recreate payout writes off the epoch path (fee ∝ weight is meant to discourage that).
6. **Validator dividends** — still share-pool staked every epoch (few validators on Null; usually not the limiter).

## What raising the UID cap would require

Without changing the cap in this PR, a future raise of `NULL_UID_BUDGET` (and related admission checks) would need:

1. **`NULL_UID_BUDGET` / `TooManyUIDsPerMechanism` / `MaxAllowedUids` policy** updated on purpose.
2. **Re-measured `block_step` / Null trim / registration weights** on reference hardware at the new N (do not invent weights).
3. **Commit/reveal bounds** — `MAX_CRV3_COMMIT_SIZE_BYTES`, Null commit queue byte/count budgets sized for a length-N weight row.
4. **Proof-size / memory** review for epoch vectors and metagraph RPCs at the new N.
5. **Operational** — claim UX and pending inventory monitoring so dissolve/trim force-payout batches stay within block weight.
6. Optionally further epoch optimizations (chunked vector IO, sparse emission persistence) if targeting ≫5k under today's block budget.

## Reproduce

```bash
cargo test -p pallet-subtensor --release null_epoch_capacity_report \
  -- --ignored --nocapture
```
