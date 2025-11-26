# scure-sr25519

Audited & minimal JS implementation of sr25519 cryptography for Polkadot.

- 🧜‍♂️ [sr25519 curve](https://wiki.polkadot.network/docs/learn-cryptography)
- Schnorr signature on Ristretto compressed Ed25519
- Hierarchical Deterministic Key Derivation (HDKD)
- Verifiable random function (VRF)
- Uses [Merlin](https://merlin.cool/index.html), which is based on [Strobe128](https://strobe.sourceforge.io).
  - **_NOTE_**: We implement only parts of these protocols which required for sr25519.
- ➰ Uses [noble-curves](https://github.com/paulmillr/noble-curves) for underlying arithmetics

### This library belongs to _scure_

> **scure** — audited micro-libraries.

- Zero or minimal dependencies
- Highly readable TypeScript / JS code
- PGP-signed releases and transparent NPM builds
- Check out [homepage](https://paulmillr.com/noble/#scure) & all libraries:
  [base](https://github.com/paulmillr/scure-base),
  [bip32](https://github.com/paulmillr/scure-bip32),
  [bip39](https://github.com/paulmillr/scure-bip39),
  [btc-signer](https://github.com/paulmillr/scure-btc-signer),
  [sr25519](https://github.com/paulmillr/scure-sr25519),
  [starknet](https://github.com/paulmillr/scure-starknet)

## Usage

> `npm install @scure/sr25519`

> `deno add jsr:@scure/sr25519`

We support all major platforms and runtimes.

### Basic

```ts
import * as sr25519 from '@scure/sr25519';
const signature = sr25519.sign(pair.secretKey, msg);
const isValid = sr25519.verify(msg, polkaSig, pair.publicKey);
const secretKey = sr25519.secretFromSeed(seed);
const publicKey = sr25519.getPublicKey(secretKey);
const sharedSecret = sr25519.getSharedSecret(secretKey, publicKey);
```

### HDKD

```ts
// hard
const secretKey = sr25519.HDKD.secretHard(pair.secretKey, cc);
const publicKey = sr25519.getPublicKey(secretKey);

// soft
const secretKey = sr25519.HDKD.secretSoft(pair.secretKey, cc);
const publicKey = sr25519.getPublicKey(secretKey);

// public
const publicKey = sr25519.HDKD.publicSoft(pubSelf, cc);
```

### VRF

```ts
const signature = sr25519.vrf.sign(msg, pair.secretKey);
const isValid = sr25519.vrf.verify(msg, sig, pair.publicKey);
```

### Migration from `@polkadot/utils-crypto`

- most derive methods in original return `{publicKey, privateKey}`, we always return only privateKey,
  you can get publicKey via `getPublicKey`
- privateKey is 64 byte (instead of 32 byte in ed25519), this is because we need nonce and privateKey can be
  derived from others (HDKD), and there would be no seed for that.

## Security

The library has been independently audited:

- at version 0.3.0, in Aug 2025, by [Oak Security](https://www.oaksecurity.io)
  - PDFs: [website](https://github.com/oak-security/audit-reports/tree/8f30dd4b59eae97194fb612aa8e773824a37bf65/Polkadot), [2025-08-22](./audit/2025-08-22-oak-security-audit.pdf), [2025-06-12](./audit/2025-06-12-oak-security-audit.pdf), [2025-06-12-fuzz](./audit/2025-06-12-oak-security-fuzzing.pdf)
  - [Changes since audit](https://github.com/paulmillr/scure-sr25519/compare/0.3.0..main)
  - Scope: everything
  - The audit has been funded by Polkadot, coordinated by [Edgeware](https://www.edgeware.io)

If you see anything unusual: investigate and report.

Low-level operations are done using noble-curves and noble-hashes.
Consult their README for more information about constant-timeness, memory dumping and supply chain security.
A few notes:

- Bigints are used, which are not const-time, but our elliptic curve cryptography
  implementation ensures algorithmic const-time for high-level items, which is more important
- Secrets are zeroized, but this is pointless, since at some point they are converted to bigints,
  and bigints cannot be zeroized in JS. Even zeroization of uint8arrays provides no guarantees.

## Speed

Benchmark results on Apple M4:

```
secretFromSeed x 493,827 ops/sec @ 2μs/op
getSharedSecret x 1,135 ops/sec @ 880μs/op
HDKD.secretHard x 54,121 ops/sec @ 18μs/op
HDKD.secretSoft x 4,108 ops/sec @ 243μs/op
HDKD.publicSoft x 4,499 ops/sec @ 222μs/op
sign x 2,475 ops/sec @ 403μs/op
verify x 955 ops/sec @ 1ms/op
vrfSign x 442 ops/sec @ 2ms/op
vrfVerify x 344 ops/sec @ 2ms/op
```

Comparison with wasm:

```
secretFromSeed wasm x 21,615 ops/sec @ 46μs/op
getSharedSecret wasm x 6,681 ops/sec @ 149μs/op
HDKD.secretHard wasm x 16,958 ops/sec @ 58μs/op
HDKD.secretSoft wasm x 16,075 ops/sec @ 62μs/op
HDKD.publicSoft wasm x 16,981 ops/sec @ 58μs/op
sign wasm x 16,559 ops/sec @ 60μs/op
verify wasm x 6,741 ops/sec @ 148μs/op
vrfSign wasm x 2,470 ops/sec @ 404μs/op
vrfVerify wasm x 2,917 ops/sec @ 342μs/op
```

## Contributing & testing

1. Clone the repository
2. `npm install` to install build dependencies like TypeScript
3. `npm run build` to compile TypeScript code
4. `npm run test` will execute all main tests

## License

The MIT License (MIT)

Copyright (c) 2024 Paul Miller [(https://paulmillr.com)](https://paulmillr.com)

See LICENSE file.
