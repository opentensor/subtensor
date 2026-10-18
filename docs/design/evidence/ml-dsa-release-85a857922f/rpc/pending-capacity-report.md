---
title: Pending RPC live capacity validation
description: Production-runtime replay, overload and rollback evidence from a normal-block localnet.
---

All five cases passed on the task-owned three-validator localnet, with normal 12-second blocks and `HashedAccountsEnabled = false`. No validator was paused and no flags, runtime code, weights or chain configuration were changed. This validates the whole-job admission repair already present at HEAD `85a857922f70c218d499d2485b899fa9ef261a05`; no production RPC edit was needed in this follow-up.

| Ready user transactions | First query | Second query | Outcome |
| --- | ---: | ---: | --- |
| 80 | 11.32 ms | 10.41 ms | Full expected pending balance |
| 100 | 18.58 ms | 13.58 ms | Full expected pending balance |
| 254 | 38.64 ms | 30.42 ms | Full expected pending balance; 256 total extrinsics including two inherents |
| 256 | 0.89 ms | 0.69 ms | Explicit work-limit error; inherents would exceed the snapshot cap |
| 257 | 0.19 ms | 0.19 ms | Explicit work-limit error before replay |

Each case required its complete submitted transaction set to be present in the ready pool, with no additional transactions and an unchanged best head. Both queries ran on that same head. Successful queries returned the entire value sum, accounting for the EVM mapping's existential-deposit subtraction. Over-limit queries returned `Pending execution deadline or work limit reached; retry later`, never a partial successful result. Every case checked that latest sender nonce/free balance and recipient balance were unchanged by the queries. All 947 accepted transfers subsequently finalized exactly once, as proved by full recipient balance and sender nonce progression.

The helper exited zero. Raw evidence, transaction hashes, before/after balances, finalized blocks and query timings are in `pending-capacity-results.json`; console output is in `pending-capacity.log`; the executable fixture is `pending_capacity.py`.

Runtime Wasm SHA-256: `edfbcf7756adf1f00953cce94c7d7741c69ccedffd55e2b2420bcf40ae36f7b6`.

Native artifact SHA-256: `eb11b5dd078cc7c9b37d9e2d1514ad6f279e617859baa9282af942b0a8b60524`.

Fixture SHA-256: `1cb64f41472152055be1d8ed40f7f76e8ea3b3e65fe4d77dca99ffeef42eac6a`.

This is evidence for cheap classical replay at the bounded snapshot limit, with the MS activation flag off. It does not establish unlimited pending throughput, worst-case ML-DSA workload capacity, or reference-hardware weights. The implementation retains its 15-second deadline, 256-extrinsic/8-MiB snapshot, cumulative attempt/byte budgets and explicit overload behavior. Nonempty pending state overrides remain explicitly unsupported rather than silently dropping pending state. Parent owns localnet shutdown after the other release-validation jobs finish.
