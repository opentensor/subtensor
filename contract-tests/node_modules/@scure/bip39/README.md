# scure-bip39

Audited & minimal JS implementation of [BIP39 mnemonic phrases](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki).

- 🔒 [**Audited**](#security) by an independent security firm
- 🔻 Tree-shakeable: unused code is excluded from your builds
- 📦 ESM and common.js
- ➰ Only 2 audited dependencies by the same author:
  [noble-hashes](https://github.com/paulmillr/noble-hashes) and [scure-base](https://github.com/paulmillr/scure-base)
- 🪶 14KB gzipped with one wordlist, 79KB with all of them: much smaller than similar libraries

Check out [scure-bip32](https://github.com/paulmillr/scure-bip32) if you need
hierarchical deterministic wallets ("HD Wallets").

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
  [starknet](https://github.com/paulmillr/scure-starknet)

## Usage

> `npm install @scure/bip39`

> `deno add jsr:@scure/bip39`

> `deno doc jsr:@scure/bip39` # command-line documentation

We don't provide source maps.
Wordlists are large, including source maps would double package size.

```js
import * as bip39 from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english';

// Generate x random words. Uses Cryptographically-Secure Random Number Generator.
const mn = bip39.generateMnemonic(wordlist);
console.log(mn);

// Reversible: Converts mnemonic string to raw entropy in form of byte array.
const ent = bip39.mnemonicToEntropy(mn, wordlist);

// Reversible: Converts raw entropy in form of byte array to mnemonic string.
bip39.entropyToMnemonic(ent, wordlist);

// Validates mnemonic for being 12-24 words contained in `wordlist`.
bip39.validateMnemonic(mn, wordlist);

// Irreversible: Uses KDF to derive 64 bytes of key data from mnemonic + optional password.
await bip39.mnemonicToSeed(mn, 'password');
bip39.mnemonicToSeedSync(mn, 'password');
```

This submodule contains the word lists defined by BIP39 for Czech, English, French, Italian, Japanese, Korean, Portuguese, Simplified and Traditional Chinese, and Spanish. These are not imported by default, as that would increase bundle sizes too much. Instead, you should import and use them explicitly.

```typescript
function generateMnemonic(wordlist: string[], strength?: number): string;
function mnemonicToEntropy(mnemonic: string, wordlist: string[]): Uint8Array;
function entropyToMnemonic(entropy: Uint8Array, wordlist: string[]): string;
function validateMnemonic(mnemonic: string, wordlist: string[]): boolean;
function mnemonicToSeed(mnemonic: string, passphrase?: string): Promise<Uint8Array>;
function mnemonicToSeedSync(mnemonic: string, passphrase?: string): Uint8Array;
```

All wordlists:

```typescript
import { wordlist as czech } from '@scure/bip39/wordlists/czech';
import { wordlist as english } from '@scure/bip39/wordlists/english';
import { wordlist as french } from '@scure/bip39/wordlists/french';
import { wordlist as italian } from '@scure/bip39/wordlists/italian';
import { wordlist as japanese } from '@scure/bip39/wordlists/japanese';
import { wordlist as korean } from '@scure/bip39/wordlists/korean';
import { wordlist as portuguese } from '@scure/bip39/wordlists/portuguese';
import { wordlist as simplifiedChinese } from '@scure/bip39/wordlists/simplified-chinese';
import { wordlist as spanish } from '@scure/bip39/wordlists/spanish';
import { wordlist as traditionalChinese } from '@scure/bip39/wordlists/traditional-chinese';
```

## Security

To audit wordlist content, run `node scripts/fetch-wordlist.js`.

The library has been independently audited:

- at version 1.0.0, in Jan 2022, by [cure53](https://cure53.de)
  - PDFs: [online](https://cure53.de/pentest-report_hashing-libs.pdf), [offline](./audit/2022-01-05-cure53-audit-nbl2.pdf)
  - [Changes since audit](https://github.com/paulmillr/scure-bip39/compare/1.0.0..main).
  - The audit has been funded by [Ethereum Foundation](https://ethereum.org/en/) with help of [Nomic Labs](https://nomiclabs.io)

The library was initially developed for [js-ethereum-cryptography](https://github.com/ethereum/js-ethereum-cryptography).
At commit [ae00e6d7](https://github.com/ethereum/js-ethereum-cryptography/commit/ae00e6d7d24fb3c76a1c7fe10039f6ecd120b77e),
it was extracted to a separate package called `micro-bip39`.
After the audit we've decided to use `@scure` NPM namespace for security.

### Supply chain security

- **Commits** are signed with PGP keys, to prevent forgery. Make sure to verify commit signatures
- **Releases** are transparent and built on GitHub CI. Make sure to verify [provenance](https://docs.npmjs.com/generating-provenance-statements) logs
  - Use GitHub CLI to verify single-file builds:
    `gh attestation verify --owner paulmillr scure-bip39.js`
- **Rare releasing** is followed to ensure less re-audit need for end-users
- **Dependencies** are minimized and locked-down: any dependency could get hacked and users will be downloading malware with every install.
  - We make sure to use as few dependencies as possible
  - Automatic dep updates are prevented by locking-down version ranges; diffs are checked with `npm-diff`
- **Dev Dependencies** are disabled for end-users; they are only used to develop / build the source code

For this package, there are 2 dependencies; and a few dev dependencies:

- [noble-hashes](https://github.com/paulmillr/noble-hashes) provides cryptographic hashing functionality
- [scure-base](https://github.com/paulmillr/scure-base) provides low-level wordlist utilities
- micro-bmark, micro-should and jsbt are used for benchmarking / testing / build tooling and developed by the same author
- prettier, fast-check and typescript are used for code quality / test generation / ts compilation. It's hard to audit their source code thoroughly and fully because of their size

## Contributing & testing

- `npm install && npm run build && npm test` will build the code and run tests.
- `npm run lint` / `npm run format` will run linter / fix linter issues.
- `npm run build:release` will build single file

## License

[MIT License](./LICENSE)

Copyright (c) 2022 Patricio Palladino, Paul Miller (paulmillr.com)
