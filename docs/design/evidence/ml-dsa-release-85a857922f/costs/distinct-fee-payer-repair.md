---
title: Distinct fee payer affordability repair
description: Local repair and focused regression evidence for protected pure creation and multisig approval affordability.
---

At HEAD `85a857922f70c218d499d2485b899fa9ef261a05`, a funded protected proxy owner and an unfunded delegate were checked only against `pure_proxy_reserves`. The delegate was absent from that map. Consequently its `required_free` remained unset, and Shield skipped the combined inner/carrier affordability check. A reproduced mock plan reported `ok=True` with delegate free=169 rao and total estimated fee=170 rao, while the owner independently covered its reserve.

The local repair includes the outer signer in the per-account balance checks even when it owes no reserve. Obligations are aggregated by decoded AccountId, so alternate SS58 prefixes cannot split one balance into separate checks. The owner still pays its own permanent/refundable reserves; the signer pays its own transaction fees and must retain the greater of its ED/frozen balance. Existing Shield pricing then adds its carrier requirement. Hotkey operations that charge coldkey fees retain their existing routing.

A subsequent boundary review confirmed the same aggregation defect for opening multisig approvals: deposit-plus-inner and inner-plus-carrier checks passed separately although their total exceeded the signer's balance. The repair carries concrete approval obligations through intent, proxy, adapter and batch composition. It aggregates each opening deposit against the actual approval origin, independently of any outer proxy fee payer, and quotes the carrier exactly once. Refundable multisig deposits are separate preview facts, never permanent `max_spend_tao` spend. Cancellation refunds are not available before submission.

Nested opening approvals defer their child approvals and pure creation, so those child affordability obligations are omitted until the executing approval. The eventual permanent protected-pure spend remains authorized and policy checked. Direct `group.approve(..., shielded=True)` now quotes the exact public-only wrapper and full deposit/inner/carrier/retained balance before unlocking, then reuses that wrapper for submission. Unknown quotes fail closed. Threshold-one groups correctly use `as_multi_threshold_1` without an approval deposit.

Focused regressions cover one-rao-below/exact/above boundaries with and without Shield, owner/delegate separation, announced proxy inside a batch, opening/later multisig execution and approval, direct groups and adapters, nested and batched deposit aggregation, deferred pure creation, threshold-one groups, unavailable carrier quotes, pre-unlock rejection, and equivalent SS58 encodings. The existing fee-routing and multisig safety tests also pass.

Command from `sdk/python`:

```text
PYTHONPATH=/tmp/ms-release-85a857922f/native:. .venv/bin/python -m pytest tests/unit/test_submission_costs.py tests/unit/test_multisig_safety.py tests/unit/test_fee_filters.py -q
105 passed in 2.49s
```

The full SDK draft exposed six incomplete success fixtures: missing multisig constants and unfunded success signers. The fake now supplies the current runtime's 132,000,000-rao base and 32,000,000-rao factor; only relevant success fixtures receive funding. Seven affected cases (including the plain sibling) passed in 0.97s after correction. The next full run identified three further formerly skipped prerequisites: the generic composed-call tests and the nested multisig origin in the pinned-byte recipient guard test. These now fund the applicable approval origin; all four affected cases (including threshold-one sibling) passed in 0.14s. Global default balances remain zero and no production check was weakened.

Scoped Ruff check, Ruff format check, and `git diff --check` passed. Matching native artifact was the parent-built `/tmp/ms-release-85a857922f/native/bittensor_core.so`. Independent read-only review found no additional wrong-payer, double-count, premature-refund-credit or wrapper-byte mismatch concern. Separate live coverage is owned by the parent and the new `test_mldsa_release_costs.py` and `test_mldsa_multisig_costs.py` suites. Parent is rerunning the complete SDK suite and final preflight on the frozen source.

Live follow-up: `test_mldsa_multisig_costs.py` passed both cases (fixed and rotating MS) on the final production source in 609 seconds. Each case checked insufficient-funding refusal through `MultisigExecute`, `MultisigIntentAdapter` and direct `group.approve`, with unchanged nonce, generation and pending storage; after top-up, actual Shield opening reserved exactly the deposit and cancellation refunded it exactly. Evidence is `costs/multisig-cost-pytest.log`. The final-source distinct-delegate rerun remains separate.

Native SDK provenance review: merge base with local `origin/main` is `6a972693ae9580541225bd37696ac2fee26b8ae9`. That tree already returned only the carrier receipt from `sdk/bittensor-core/src/client.rs::submit_shielded`, with the inner hash as metadata. Current lines 883–889 blame to `33fa16a7aa1` from 2026-07-12; current changes only share the signer snapshot across inner/carrier. Likewise native `Executor::submit_shielded` already enforced the policy through an inner-only plan. These are pre-existing native API limitations, not introduced by this PR. No broader native API change was made in this repair.
