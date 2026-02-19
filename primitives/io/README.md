# stp-io

`stp` = **Subtensor Primitive**.

Host functions that allow the Subtensor runtime (Wasm) to call into the node's native code for operations that cannot run inside the sandbox.

## Host functions

### `crypto::mlkem768_decapsulate`

Decapsulates an ML-KEM-768 ciphertext using the key material held in the node's `ShieldKeystore`. Writes the 32-byte shared secret into the caller-provided buffer.

### `crypto::aead_decrypt`

Decrypts an XChaCha20-Poly1305 ciphertext given a 32-byte key, 24-byte nonce, message, and optional AAD. Returns the plaintext bytes.

## How it fits in

The runtime cannot perform cryptographic operations directly because the secret keys live on the node side. `stp-io` bridges this gap:

```
Runtime (Wasm)          Host (native)
─────────────           ─────────────
pallet-shield           ShieldKeystore
  │                       │
  ├─ mlkem768_decapsulate ─►  ML-KEM decaps
  └─ aead_decrypt ─────────►  XChaCha20 decrypt
```

The host functions are registered via `SubtensorHostFunctions` and accessed through the `ShieldKeystoreExt` externalities extension from [`stp-shield`](https://github.com/opentensor/polkadot-sdk).

`no_std`-compatible.
