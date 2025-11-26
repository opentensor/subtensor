# @polkadot-labs/hdkd

`@polkadot-labs/hdkd` is a Hierarchical Deterministic (HD) account derivation library compatible with the Polkadot and Substrate ecosystems.
It supports the sr25519, ed25519, and ecdsa signature schemes, providing a comprehensive solution for HD account derivation.
It is built on top of `@polkadot-labs/hdkd-helpers`.

## Features

- **HD Derivation**: Utilities for deriving HD accounts with hard and soft derivation.
- **sr25519**: Support for the sr25519 signature scheme.
- **ed25519**: Support for the ed25519 signature scheme.
- **ecdsa**: Support for the ecdsa signature scheme.

## Installation

To install the library, you can use npm or yarn or pnpm:

```sh
npm install @polkadot-labs/hdkd
```

## Usage

Here is an example of how to use the library:

```ts
import { sr25519CreateDerive } from "@polkadot-labs/hdkd"
import {
  sr25519,
  DEV_PHRASE,
  entropyToMiniSecret,
  mnemonicToEntropy,
} from "@polkadot-labs/hdkd-helpers"

const entropy = mnemonicToEntropy(DEV_PHRASE)
const miniSecret = entropyToMiniSecret(entropy)
const derive = sr25519CreateDerive(miniSecret)

// Example usage for generating a sr25519 keypair with hard derivation
const keypair = derive("//Alice")

// Example usage for signing a message
const message = new TextEncoder().encode("Hello")
const signature = keypair.sign(message)

// Example usage for verifying a signature
const isValid = sr25519.verify(signature, message, keypair.publicKey)
console.log("Is valid:", isValid)
```

## License

This project is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.
