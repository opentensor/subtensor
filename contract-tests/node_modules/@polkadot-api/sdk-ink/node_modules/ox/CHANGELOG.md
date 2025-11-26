# ox

## 0.9.6

### Patch Changes

- [`c154290`](https://github.com/wevm/ox/commit/c154290c6958702f854bece58309a15694589f22) Thanks [@jxom](https://github.com/jxom)! - Added ABI-shorthand for `AbiItem.{getSelector,getSignature,getSignatureHash}`

## 0.9.5

### Patch Changes

- [#113](https://github.com/wevm/ox/pull/113) [`e21cb3c`](https://github.com/wevm/ox/commit/e21cb3cf0b7412f9ca72824247d22ba25e8be4c9) Thanks [@jxom](https://github.com/jxom)! - Added support for specifying the ABI and signature name to:

  - `AbiFunction.{encodeData,encodeResult,decodeData,decodeResult}`
  - `AbiError.{encode,decode}`
  - `AbiEvent.{encode,decode}`

  Example:

  ```ts twoslash
  import { AbiFunction } from "ox";
  import { abi } from "./abi";

  const data = AbiFunction.encodeData(abi, "approve", [
    "0x0000000000000000000000000000000000000000",
    1n,
  ]);
  ```

## 0.9.4

### Patch Changes

- [`8aaf1a4`](https://github.com/wevm/ox/commit/8aaf1a4c4aedf654abf9319932eb57b560186d43) Thanks [@jxom](https://github.com/jxom)! - Removed proxy packages. Metro (the problematic bundler) now respects `package.json#exports`.

## 0.9.3

### Patch Changes

- [`1cd8943`](https://github.com/wevm/ox/commit/1cd894336fb0a4cef8b0879cc214a7997fea1042) Thanks [@jxom](https://github.com/jxom)! - Updated dependencies.

## 0.9.2

### Patch Changes

- [`9be7919`](https://github.com/wevm/ox/commit/9be791906d9496111a1607344ddb02077f02f6a6) Thanks [@jxom](https://github.com/jxom)! - Updated `ox/erc8010` to latest spec changes.

## 0.9.1

### Patch Changes

- [`dec161a`](https://github.com/wevm/ox/commit/dec161ac7b3089bd6a0647d91e02f174ac421d65) Thanks [@jxom](https://github.com/jxom)! - Fixed `signature` type on ERC-6492 and ERC-8010.

## 0.9.0

### Minor Changes

- [#104](https://github.com/wevm/ox/pull/104) [`4f4b635`](https://github.com/wevm/ox/commit/4f4b635dfb399ca9df07bab843857743f389639e) Thanks [@jxom](https://github.com/jxom)! - **Breaking(`ox/erc6492`:**

  - Renamed `WrappedSignature` to `SignatureErc6492`
  - Renamed `WrappedSignature.WrappedSignature` to `SignatureErc6492.Unwrapped`
  - Renamed `WrappedSignature.toHex` to `SignatureErc6492.wrap`
  - Renamed `WrappedSignature.fromHex` to `SignatureErc6492.unwrap`

- [#104](https://github.com/wevm/ox/pull/104) [`4f4b635`](https://github.com/wevm/ox/commit/4f4b635dfb399ca9df07bab843857743f389639e) Thanks [@jxom](https://github.com/jxom)! - Added `ox/erc8010` entrypoint with `SignatureErc8010` module.

## 0.8.9

### Patch Changes

- [#102](https://github.com/wevm/ox/pull/102) [`5796d6d`](https://github.com/wevm/ox/commit/5796d6dbebff719c84b4658de37e3240adbc87e1) Thanks [@dan1kov](https://github.com/dan1kov)! - Fixed signature destructuring on `Authorization.fromTuple`.

## 0.8.8

### Patch Changes

- [#98](https://github.com/wevm/ox/pull/98) [`96c2046`](https://github.com/wevm/ox/commit/96c20462420a3e6be1301cccb4b66afe1bccc3f8) Thanks [@mmv08](https://github.com/mmv08)! - Added handling for `bigint` chain IDs in `TypedData.extractEip712DomainTypes`.

## 0.8.7

### Patch Changes

- [`9a9ef21`](https://github.com/wevm/ox/commit/9a9ef21e17f982fa6f7b76d5ad615b68d200d9eb) Thanks [@jxom](https://github.com/jxom)! - Fixed zeroish conversion of `chainId` and `nonce` in `Authorization.fromTuple`.

## 0.8.6

### Patch Changes

- [#94](https://github.com/wevm/ox/pull/94) [`301c319`](https://github.com/wevm/ox/commit/301c319fafab25b1a3a85bcf6bc81c3c9dee72d9) Thanks [@jxom](https://github.com/jxom)! - **ERC-4337**: Added `UserOperation.fromPacked`.

## 0.8.5

### Patch Changes

- [#92](https://github.com/wevm/ox/pull/92) [`b6eaa05`](https://github.com/wevm/ox/commit/b6eaa055ce415cd24f802b8bfa5bdbbd53480ab8) Thanks [@jxom](https://github.com/jxom)! - Added support for EntryPoint 0.8.

## 0.8.4

### Patch Changes

- [`ce19a08`](https://github.com/wevm/ox/commit/ce19a087bffaa205067fca530532fb05cc02c792) Thanks [@jxom](https://github.com/jxom)! - Added `stack` to `Provider.InternalError`.

## 0.8.3

### Patch Changes

- [#74](https://github.com/wevm/ox/pull/74) [`72209ef`](https://github.com/wevm/ox/commit/72209efaf2bf6dd5d71274db8df7416532ebe9cb) Thanks [@danpopenko](https://github.com/danpopenko)! - Added extensions support for `WebAuthnP256.sign`.

## 0.8.2

### Patch Changes

- [`9fd0bf0`](https://github.com/wevm/ox/commit/9fd0bf0460694709566805bc29f50cad25816620) Thanks [@jxom](https://github.com/jxom)! - Added [ECDH (Elliptic Curve Diffie-Hellman)](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/deriveKey#ecdh) shared secrets to `P256`, `Secp256k1`, and `WebCryptoP256` modules. This enables secure key agreement between parties using elliptic curve cryptography for both secp256k1 and secp256r1 (P256) curves, with support for both `@noble/curves` (for `P256` and `Secp256k1`) implementation and Web Crypto APIs (`WebCryptoP256`).

  - `P256.getSharedSecret`
  - `Secp256k1.getSharedSecret`
  - `WebCryptoP256.getSharedSecret`

- [`9fd0bf0`](https://github.com/wevm/ox/commit/9fd0bf0460694709566805bc29f50cad25816620) Thanks [@jxom](https://github.com/jxom)! - Added `createKeyPair` helper functions for `Bls`, `P256`, and `Secp256k1` modules. These functions provide a convenient way to generate complete key pairs (private key + public key) in a single operation, simplifying key generation workflows and reducing the need for separate `randomPrivateKey` and `getPublicKey` calls.

- [`9fd0bf0`](https://github.com/wevm/ox/commit/9fd0bf0460694709566805bc29f50cad25816620) Thanks [@jxom](https://github.com/jxom)! - Added `Ed25519` and `X25519` modules. The `Ed25519` module provides functionality for creating key pairs, signing messages, and verifying signatures using the Ed25519 signature scheme. The `X25519` module enables Elliptic Curve Diffie-Hellman (ECDH) key agreement operations for secure shared secret derivation.

## 0.8.1

### Patch Changes

- [`74e47c5`](https://github.com/wevm/ox/commit/74e47c5df471a48f4fb389f0684ca52f841fbc11) Thanks [@jxom](https://github.com/jxom)! - Added `Keystore.toKey` and `Keystore.toKeyAsync` to derive a key from a JSON Keystore using a password.

## 0.8.0

### Minor Changes

- [`7fc1da0`](https://github.com/wevm/ox/commit/7fc1da0717a17dbac0e4effed2ea3911c7ca3236) Thanks [@jxom](https://github.com/jxom)! - **Breaking(`Keystore`):** Keystore derivation functions (e.g. `Keystore.pbkdf2`) now return a tuple of the key and derivation options,
  instead of an object with the key and options.

  ```diff
  import { Keystore } from 'ox'

  - const key = Keystore.pbkdf2({ password: 'testpassword' })
  + const [key, opts] = Keystore.pbkdf2({ password: 'testpassword' })
  ```

- [`7fc1da0`](https://github.com/wevm/ox/commit/7fc1da0717a17dbac0e4effed2ea3911c7ca3236) Thanks [@jxom](https://github.com/jxom)! - **Breaking(`Keystore`):** `Keystore.decrypt` function interface no longer requires an object as the second parameter, now it only requires the key itself.

  ```diff
  import { Keystore } from 'ox'

  const [key, opts] = Keystore.pbkdf2({ password: 'testpassword' })

  const encrypted = await Keystore.encrypt(secret, key, opts)

  + const decrypted = await Keystore.decrypt(encrypted, key)
  ```

- [`7fc1da0`](https://github.com/wevm/ox/commit/7fc1da0717a17dbac0e4effed2ea3911c7ca3236) Thanks [@jxom](https://github.com/jxom)! - **Breaking(`Keystore`):** `Keystore.encrypt` function interface has changed to require derivation options (`opts`).

  ```diff
  import { Keystore } from 'ox'

  const [key, opts] = Keystore.pbkdf2({ password: 'testpassword' })

  - const encrypted = await Keystore.encrypt(secret, key)
  + const encrypted = await Keystore.encrypt(secret, key, opts)
  ```

## 0.7.2

### Patch Changes

- [`6090531`](https://github.com/wevm/ox/commit/6090531e29be96d2bd1eda1f85f3e7322b48ff18) Thanks [@jxom](https://github.com/jxom)! - Updated dependencies.

- [`c4c7070`](https://github.com/wevm/ox/commit/c4c7070c7d50fd8d745e5f881305bdf4aa5362d0) Thanks [@jxom](https://github.com/jxom)! - Fixed parsing of zeroish nonces.

## 0.7.1

### Patch Changes

- [#75](https://github.com/wevm/ox/pull/75) [`27a1e28`](https://github.com/wevm/ox/commit/27a1e28e1f403ca18d428611fc3b88dcb5a4503e) Thanks [@jxom](https://github.com/jxom)! - Added `Keystore` module.

## 0.7.0

### Minor Changes

- [`09f72cb`](https://github.com/wevm/ox/commit/09f72cb33f076151e3437cf42b1cad775148a2bb) Thanks [@jxom](https://github.com/jxom)! - Updated EIP-5792 APIs to the latest spec on `RpcSchema`.

### Patch Changes

- [`61a9c57`](https://github.com/wevm/ox/commit/61a9c5798b8072b9c16691463742835b15c17468) Thanks [@jxom](https://github.com/jxom)! - Added EIP-5792 provider errors.

## 0.6.12

### Patch Changes

- [`5247546`](https://github.com/wevm/ox/commit/5247546f0400a3edb3c99f90be7696ab7d3fd7d9) Thanks [@jxom](https://github.com/jxom)! - Fixed `Provider.parseError` case.

## 0.6.11

### Patch Changes

- [`ba67f11`](https://github.com/wevm/ox/commit/ba67f11bb377f132583a3eb04ae761bd36a08387) Thanks [@jxom](https://github.com/jxom)! - Enhanced handling of arbitrary Provider errors.

## 0.6.10

### Patch Changes

- [#65](https://github.com/wevm/ox/pull/65) [`33712a5`](https://github.com/wevm/ox/commit/33712a5680e4b2ad6be0513e70049160628287a0) Thanks [@thomas779](https://github.com/thomas779)! - Added support for multiple `credentialId`s in `WebAuthnP256`.

- [`10e6449`](https://github.com/wevm/ox/commit/10e6449e0e5f060c5ea3db026f4fb98978f78cca) Thanks [@jxom](https://github.com/jxom)! - Added case to fall back to `cause.details` for `BaseError` details.

## 0.6.9

### Patch Changes

- [`6480607`](https://github.com/wevm/ox/commit/6480607767387a64f720e0fa3abbc26ea9409990) Thanks [@jxom](https://github.com/jxom)! - Fixed `AbiEvent.encode` for zeroish arguments.

## 0.6.8

### Patch Changes

- [#60](https://github.com/wevm/ox/pull/60) [`7ff54a2`](https://github.com/wevm/ox/commit/7ff54a2d0a77e2af5a2cc0e1095f0f8d952510c8) Thanks [@jxom](https://github.com/jxom)! - Added `BinaryStateTree` (EIP-7864) module.

## 0.6.7

### Patch Changes

- [`076c6a2`](https://github.com/wevm/ox/commit/076c6a260bfd42d6e66a7490bfb36425f91099d7) Thanks [@jxom](https://github.com/jxom)! - Removed redundant pure annotation.

## 0.6.6

### Patch Changes

- [`980f0e2`](https://github.com/wevm/ox/commit/980f0e269cca1ef3c564aba75055fef867ca3e6f) Thanks [@jxom](https://github.com/jxom)! - Fixed TSDoc.

## 0.6.5

### Patch Changes

- [`0b5182f`](https://github.com/wevm/ox/commit/0b5182f94821715c227dc8b0c891d4548b30fa0e) Thanks [@jxom](https://github.com/jxom)! - Fixed build process for typedef generation.

## 0.6.4

### Patch Changes

- [`74ceae4`](https://github.com/wevm/ox/commit/74ceae4089663ebae18690a44fd98accc28b9b5c) Thanks [@jxom](https://github.com/jxom)! - Fixed `Provider.parseError` behavior.

## 0.6.3

### Patch Changes

- [`ddaed51`](https://github.com/wevm/ox/commit/ddaed51550308eceda3c9a080503cf1fdfac6ac0) Thanks [@jxom](https://github.com/jxom)! - Fixed parsing of Provider RPC errors.

## 0.6.2

### Patch Changes

- [`e541cec`](https://github.com/wevm/ox/commit/e541ceca3c00f0d0b2fbd239696476934dc13ea3) Thanks [@jxom](https://github.com/jxom)! - Modified fallback RPC Errors to `RpcResponse.InternalError`.

## 0.6.1

### Patch Changes

- [`5d007ae`](https://github.com/wevm/ox/commit/5d007aebab4a7fe6acc8eb3cfecbce59fe79a00b) Thanks [@jxom](https://github.com/jxom)! - Added `RpcResponse.parseErrorObject` and `Provider.parseErrorObject`.

## 0.6.0

### Minor Changes

- [`94ec558`](https://github.com/wevm/ox/commit/94ec558c3f56d3254080be520a0d257e8b5d42c2) Thanks [@jxom](https://github.com/jxom)! - Added `BlockOverrides` & `StateOverrides` modules.

- [`94ec558`](https://github.com/wevm/ox/commit/94ec558c3f56d3254080be520a0d257e8b5d42c2) Thanks [@jxom](https://github.com/jxom)! - Added `eth_simulateV1` to `eth_` RPC schema.

## 0.5.0

### Minor Changes

- [`1406e22`](https://github.com/wevm/ox/commit/1406e224d0527732885fdb7737ed2f0dc41929ef) Thanks [@jxom](https://github.com/jxom)! - Added ERC-4337 utilities.

## 0.4.4

### Patch Changes

- [#45](https://github.com/wevm/ox/pull/45) [`48b896f`](https://github.com/wevm/ox/commit/48b896f3c491bcf9e0d8460857b278ede74eaf9e) Thanks [@deodad](https://github.com/deodad)! - Ensured addresses are checksummed when creating SIWE messages

## 0.4.3

### Patch Changes

- [`c09d165`](https://github.com/wevm/ox/commit/c09d1655a1fa65be33d0dfb86d14cfe0dad7bdc3) Thanks [@jxom](https://github.com/jxom)! - Added `checksumAddress` as an option to `AbiParameters.{encode|decode}`.

## 0.4.2

### Patch Changes

- [#40](https://github.com/wevm/ox/pull/40) [`47e306d`](https://github.com/wevm/ox/commit/47e306d8ab95140eb7e2589c05351d1663a507ae) Thanks [@jxom](https://github.com/jxom)! - **ox/erc6492:** Added universal signature verification exports.

## 0.4.1

### Patch Changes

- [#37](https://github.com/wevm/ox/pull/37) [`39604df`](https://github.com/wevm/ox/commit/39604df9f84b810322e12f767ef450c0c2ced308) Thanks [@jxom](https://github.com/jxom)! - Added `ox/erc6492` entrypoint.

## 0.4.0

### Minor Changes

- [#35](https://github.com/wevm/ox/pull/35) [`4680b06`](https://github.com/wevm/ox/commit/4680b06d4715b1b62d903f45490d325506a1e959) Thanks [@gregfromstl](https://github.com/gregfromstl)! - Updated `Signature.toHex` to serialize the last byte as `v` instead of `yParity` for widened compatibility.

### Patch Changes

- [`15f9863`](https://github.com/wevm/ox/commit/15f98630c46ec0c09998162a92a5e8bac709e32d) Thanks [@jxom](https://github.com/jxom)! - Added assertion for ABI-encoding integer ranges.

- [`2e0d4af`](https://github.com/wevm/ox/commit/2e0d4af5c6e26c09a9b83971be0fc06415ee4976) Thanks [@jxom](https://github.com/jxom)! - Added support for block identifiers.

## 0.3.1

### Patch Changes

- [`e4104cd`](https://github.com/wevm/ox/commit/e4104cdb217de1fa30480b40060eb0fb0f7ad8d5) Thanks [@jxom](https://github.com/jxom)! - Added `extraEntropy` option to `Secp256k1.sign` & `P256.sign`.

## 0.3.0

### Minor Changes

- [`9ad0d2c`](https://github.com/wevm/ox/commit/9ad0d2c9777b5c6a8c1cd64ad8742f9c05706606) Thanks [@jxom](https://github.com/jxom)! - Added extra entropy to signature generation.

## 0.2.2

### Patch Changes

- [`4f40358`](https://github.com/wevm/ox/commit/4f4035826313dce974b7c7fa64ba4ea20d1f7f61) Thanks [@jxom](https://github.com/jxom)! - Tweaked `RpcResponse` and `Provider` errors to have optional parameters.

## 0.2.1

### Patch Changes

- [`6e4b635`](https://github.com/wevm/ox/commit/6e4b635ee720312be6631dee4f24fdd3c066f2eb) Thanks [@jxom](https://github.com/jxom)! - Derive EIP-712 Domain type if not provided in `TypedData.serialize`.

## 0.2.0

### Minor Changes

- [`2f0fc9b`](https://github.com/wevm/ox/commit/2f0fc9b66ff70bf03a3ecf146ed1a62433f53eb8) Thanks [@jxom](https://github.com/jxom)! - **Breaking:** Removed `.parseError` property on functions. Use the `.ErrorType` property instead. [Example](https://oxlib.sh/error-handling#usage-with-neverthrow)

### Patch Changes

- [`af01579`](https://github.com/wevm/ox/commit/af01579951b898ebd659cd6b64aaa56f7733c191) Thanks [@jxom](https://github.com/jxom)! - Assert that EIP-712 domains are valid.

## 0.1.8

### Patch Changes

- [#25](https://github.com/wevm/ox/pull/25) [`5da9efb`](https://github.com/wevm/ox/commit/5da9efbfebfa738ee0f78927e90b3fab61cbb2e8) Thanks [@tmm](https://github.com/tmm)! - Shimmed `WebAuthnP256.createCredential` for 1Password Firefox Add-on.

## 0.1.7

### Patch Changes

- [`33b5123`](https://github.com/wevm/ox/commit/33b51236908f17cb8644a47e222995e1800853db) Thanks [@tmm](https://github.com/tmm)! - Updated Provider errors.

## 0.1.6

### Patch Changes

- [`4405c4b`](https://github.com/wevm/ox/commit/4405c4bd2bff3f9f222a90de7323cce77c94b5f3) Thanks [@jxom](https://github.com/jxom)! - Amended `accountsChanged` parameter to be `readonly`.

- [#22](https://github.com/wevm/ox/pull/22) [`23f2d61`](https://github.com/wevm/ox/commit/23f2d61f817c5d33f0053cb154447f0b26244cc1) Thanks [@tmm](https://github.com/tmm)! - Added EIP 1193 errors.

## 0.1.5

### Patch Changes

- [`644b96a`](https://github.com/wevm/ox/commit/644b96a169a118c6f0606eda5354785523ed099b) Thanks [@jxom](https://github.com/jxom)! - Added additional guard for `result` in `Provider.from`.

## 0.1.4

### Patch Changes

- [`777fe42`](https://github.com/wevm/ox/commit/777fe4249c5225c676ff690fda58c5fcfb35d1f0) Thanks [@jxom](https://github.com/jxom)! - Tweaked `trimLeft` to remove all leading zeros.

## 0.1.3

### Patch Changes

- [`868d431`](https://github.com/wevm/ox/commit/868d4319a8cda77345f85f9f2e88ca786f0c8cfe) Thanks [@jxom](https://github.com/jxom)! - Added handling for odd-length hex values.

## 0.1.2

### Patch Changes

- [#17](https://github.com/wevm/ox/pull/17) [`f438faf`](https://github.com/wevm/ox/commit/f438fafbd396248283876eba220f4c661c47bfd2) Thanks [@jxom](https://github.com/jxom)! - Moved modules to `core/`.

## 0.1.1

### Patch Changes

- [`b7de4f2`](https://github.com/wevm/ox/commit/b7de4f2180520fd7f2bf08955df6e676d75db93e) Thanks [@jxom](https://github.com/jxom)! - Fixed `RpcSchema` inference on `params`.

## 0.1.0

### Minor Changes

- [`4297bcf`](https://github.com/wevm/ox/commit/4297bcf0acef7f1f208ba3770d679fefa0c2cb8d) Thanks [@jxom](https://github.com/jxom)! - Initial release.
