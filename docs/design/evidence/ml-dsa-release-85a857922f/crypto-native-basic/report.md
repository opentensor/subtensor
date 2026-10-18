---
title: "Exact native Rust protected-wallet transfers"
description: "Eight finalized transfers verify seed-restored authority and beneficiary balances across all four protected wallet modes."
---

**PASS: eight finalized transfers, two sequential transfers in each of four protected wallet modes.** Candidate `85a857922f70c218d499d2485b899fa9ef261a05`, unchanged production runtime on the parent's 12-second-slot costs localnet. Each invocation constructs a fresh native Rust keypair from its disposable disk-wallet seed and calls the unmodified public `Client::submit` with current authority, no explicit nonce and a 64-block ordinary era. No production library was patched.

| Crypto code | Finalized transfer blocks | Reported and debit-verified fees, rao | Beneficiary net credit |
| --- | --- | --- | --- |
| 4 | 86, 89 | 288040, 288040 | 2,000,000 rao |
| 6 | 86, 89 | 288335, 288335 | 2,000,000 rao |
| 7 | 86, 89 | 332352, 332352 | 2,000,000 rao |
| 8 | 86, 89 | 332352, 332352 | 2,000,000 rao |

For each transfer the test separately found the exact encoded extrinsic in the canonical finalized block, verified its `System.ExtrinsicSuccess`, and matched the native receipt's block/index. Each beneficiary gained exactly 1,000,000 rao per transfer. Each sender lost exactly that amount plus the reported transaction fee; nonce and registered generation advanced by one. Rotating modes 4/6/7 changed commitment, fixed MS mode 8 retained commitment while incrementing replay generation. Reloaded private keyfiles remained at generation zero before the second process invocation, demonstrating recovery from chain authority rather than persisted local counters.

This confirms the ordinary native Rust submission path for these balance transfers and all four protected modes. It does not convert the separate exact-native Shield rejection into success or establish every native SDK intent. The native library and all harness dependencies match the previous Shield harness byte-for-byte; full identities are in `native-artifacts.json`, and the temporary executable source is `native_transfer.rs`. Native `bittensor_core` rlib SHA-256: `40a9313aeb6e98d9f6ff8f8f5d401fd07eddc1b5f9dcb4b2c83b9aec5ab1e966`.

`results.json` contains public account identities, exact encoded included extrinsics, events, before/after authority and account state, and recipient deltas. Disposable seeds were passed only through stdin, never recorded in evidence. Private wallet directories and compiled harness are excluded from the portable manifest. The fixture process exited successfully and no further chain actions are needed by this subtask; parent owns chain shutdown.
