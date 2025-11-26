# @polkadot-labs/hdkd-helpers

`@polkadot-labs/hdkd-helpers` is a pure JavaScript library providing utility functions for three signature schemes: sr25519, ed25519, and ecdsa.
This library is designed to assist with Hierarchical Deterministic Key Derivation (HDKD) in the Polkadot ecosystem.
Additionally, it includes utilities for deriving HD accounts with hard and soft derivation, creating ss58 addresses, and deriving private keys through bip39.
It is built on top of `@noble/hashes`, `@noble/curves`, and `@scure/sr25519`.

## Features

- **sr25519**: Utilities for the sr25519 signature scheme.
- **ed25519**: Utilities for the ed25519 signature scheme.
- **ecdsa**: Utilities for the ecdsa signature scheme.
- **ss58**: Utilities for creating ss58 addresses.
- **Hierarchical Deterministic Accounts**: Utilities for deriving HD accounts with hard and soft derivation.
- **bip39**: Utilities for deriving private keys through bip39.

## Installation

To install the library, you can use npm or yarn or pnpm:

```sh
npm install @polkadot-labs/hdkd-helpers
```

## Usage

Here is an example of how to use the library:

```ts
import {
  sr25519,
  DEV_PHRASE,
  entropyToMiniSecret,
  mnemonicToEntropy,
} from "@polkadot-labs/hdkd-helpers"
import { secretFromSeed } from "@scure/sr25519"

const entropy = mnemonicToEntropy(DEV_PHRASE)
const miniSecret = entropyToMiniSecret(entropy)

// Example usage for generating a sr25519 keypair with hard derivation
const privateKey = secretFromSeed(miniSecret)
const publicKey = sr25519.getPublicKey(privateKey)

// Example usage for signing a message
const message = new TextEncoder().encode("Hello")
const signature = sr25519.sign(message, privateKey)

// Example usage for verifying a signature
const isValid = sr25519.verify(signature, message, publicKey)
console.log("Is valid:", isValid)
```

## License

This project is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.
