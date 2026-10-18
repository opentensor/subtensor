---
title: "Independent direct SDK multisig final-state verification"
description: "Sixteen public fixtures checked against finalized balances, authority, deposits and canonical events."
---

All 16 current-run cases passed independent checks at finalized block 418 (`0x35b37608376e385fa73cffef44b5f9432019246dfcf9386d058c00636c2cd487`). Both receiving-address and local-SS58 forms, thresholds one and two, and hashed SR/ED signer and recipient combinations are represented exactly once.

Each group holds 9 TAO after funding with 10; its recipient holds 1 TAO. There are no pending multisigs or remaining member approval reserves. Member free balances equal their 100 TAO funding minus canonical fee events; exact nonces and hashed generations match those included transactions. Each permanent 0.2 TAO registration reserve was paid by Alice, verified through matching Registered/Reserved events and current registry presence. Threshold-two approval reserve and refund events match the runtime deposit constants.

Only public coldkeypub.txt files were read. Fixture ordering was matched to each worker's completed case IDs, then independently cross-checked against crypto codes and the unambiguously funded threshold-derived group. The verifier made only chain reads and did not change source, sign transactions, or operate chains. Full-suite completion is a separate result.
