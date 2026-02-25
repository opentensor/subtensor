# pallet-shield

FRAME pallet for opt-in, per-block ephemeral-key encrypted transactions (MEV shielding).

## Overview

Block authors rotate ML-KEM-768 key pairs every slot via a mandatory inherent. Users encrypt their extrinsics to the next block author's public key, preventing front-running and sandwich attacks.

### Key rotation

Each block includes an `announce_next_key` inherent that:

1. Shifts `NextKey` into `CurrentKey` (so the previous key is still accepted during the transition).
2. Stores the current author's freshly generated public key in `AuthorKeys`.
3. Looks up the *next* author's key from `AuthorKeys` and exposes it as `NextKey`.

### Encrypted transaction flow

1. User reads `NextKey` from storage (ML-KEM-768 public key, 1184 bytes).
2. User encrypts a signed extrinsic with ML-KEM-768 + XChaCha20-Poly1305, producing:

   ```
   ciphertext = key_hash(16) || kem_len(2) || kem_ct || nonce(24) || aead_ct
   ```

3. User submits `submit_encrypted(ciphertext)` signed with their account.
4. The block author decrypts and includes the inner extrinsic in the same block.

### Transaction extension

`CheckShieldedTxValidity` validates shielded transactions at two levels:

- **Pool validation** — rejects malformed ciphertext (unparseable structure).
- **Block building** (`InBlock`) — additionally checks that `key_hash` matches either `CurrentKey` or `NextKey`, rejecting stale or tampered submissions.

## Storage

| Item | Description |
|------|-------------|
| `CurrentKey` | Previous block's `NextKey`, kept for one-block grace period |
| `NextKey` | Public key users should encrypt to |
| `AuthorKeys` | Per-authority latest announced public key |

## Dependencies

- [`stp-shield`](https://github.com/opentensor/polkadot-sdk) — shared types (`ShieldedTransaction`, `ShieldPublicKey`, `InherentType`)
- [`stp-io`](../../primitives/io) — host functions for ML-KEM decapsulation and AEAD decryption
