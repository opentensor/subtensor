---
title: "Fresh disk restoration and exact native Shield validation"
description: "Four protected wallet modes pass HTTP authentication; exact native Shield remains blocked by a pre-existing lifetime mismatch."
---

Candidate `85a857922f70c218d499d2485b899fa9ef261a05`, production runtime SHA-256 `edfbcf7756adf1f00953cce94c7d7741c69ccedffd55e2b2420bcf40ae36f7b6`. The normal 12-second-slot localnet endpoint was `ws://127.0.0.1:26444`, genesis `0x2a17720a249e9ba6a70eea7ea3a62547e25b30d6d48fe5c57c310f39331d9723`. Parent owns its activation/shutdown. This subtask used fresh disposable disk wallets and locked Alice sponsorship; it changed no global settings or production source. Exact source identities, the local Python overlay, and native library graph are recorded separately.

| Mode | Fresh disk HTTP authentication | Exact native Client Shield |
| --- | --- | --- |
| Hashed SR, code 4 | Passed | Pool rejected: lifetime 64 > 8 |
| Hashed ED, code 6 | Passed | Pool rejected: lifetime 64 > 8 |
| Rotating MS, code 7 | Passed | Pool rejected: lifetime 64 > 8 |
| Fixed MS, code 8 | Passed | Pool rejected: lifetime 64 > 8 |

Classical hashed wallets rejected unretired generation-zero HTTP signing and preparation, then passed after a finalized on-chain rotation and fresh disk reload. Keyfiles remained generation zero; `prepare_signer` resolved finalized authority. Both MS modes used a finalized Message capability after fresh disk restore, verified an owner-bound signed request, then revoked the capability and rejected fresh resolution. All four rejected replay, changed body, and wrong receiver, while successful offchain authentication left System account and spending authorization unchanged. These checks exercise the HTTP signing/verification helpers and finalized capability lookup, not an external HTTP server deployment or wall-clock timeout test. The nonce clock is controlled during signature verification.

The exact unmodified Rust `Client::submit_shielded` was linked from the same release dependency graph as the matching Python extension. All four submissions returned `Invalid Transaction / Transaction is outdated` before inclusion. `native-rejection-observations.json` independently checks finalized System account and registration state after the attempted calls: balance, nonce, generation, and commitment remained unchanged for every mode. No native successful carrier or decrypted-inner execution is claimed. The 64-block native lifetime versus runtime maximum 8 blocks is a confirmed existing base-branch defect; see `native-mortality-finding.md`. Eight blocks means approximately 96 seconds on these chains. Python's separate Shield pipeline selects the correct lifetime and is covered by the parent's live suite.

`results.json` preserves accepted classical HTTP evidence and exact-native failures, as well as an invalid MS fixture assertion that assumed a `WalletSigner` exposes `hashed_generation`. `ms-resume-results.json` contains the corrected MS checks on the same newly created wallets. The earlier wrong-receiver fixture remains under `rejected-wrong-receiver-fixture/`; it accidentally selected the same helper hotkey twice, and is not evidence of a production failure. `fixture-classification.json` explicitly distinguishes these fixture errors from the genuine native SDK limitation.

The direct-rustc harness is `native_shield.rs`; `native-artifacts.json` pins its native rlib, dependencies, source and executable hashes. Native Python extension SHA-256 is `eb11b5dd078cc7c9b37d9e2d1514ad6f279e617859baa9282af942b0a8b60524`. Temporary private wallet directories, executables, caches and source dependency symlinks are excluded from the portable manifest. Both live fixture processes have exited. No production edits, commits, pushes, dependency changes, or chain launches were performed by this subtask.
