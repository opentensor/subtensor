# @polkadot-api/merkleize-metadata

This TS package provides utils for the merkleization of [`frame_metadata`](https://docs.rs/frame-metadata/latest/frame_metadata/) as described in
[RFC78](https://polkadot-fellows.github.io/RFCs/approved/0078-merkleized-metadata.html).

## Usage

```ts
import { merkleizeMetadata } from "@polkadot-api/merkleized-metadata"

const ksmMetadata = new Uint8Array(await readFile("ksm.bin"))
const merkleizedMetadata = merkleizeMetadata(ksmMetadata, {
  decimals: 12,
  tokenSymbol: "KSM",
})

// it returns the digest value of the metadata (aka its merkleized root-hash)
const rootHash = merkleizedMetadata.digest()

// given an extrinsic, it returns an encoded `Proof`
const proof1: Uint8Array = merkleizedMetadata.getProofForExtrinsic(
  // Hex for the transaction bytes
  "c10184008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4801127d333c8f60c0d81dd0a6e2e20ea477a06f96aaca1811872c54c244f0935c60b1f8a38aabef3d3a4ef4050d8d078e35b57b3cf4f9545f8145ce98afb8755384550000000000001448656c6c6f",
  // Optionally, we can pass the tx additional signed data
  "386d0f001a000000143c3561eefac7bc66facd4f0a7ec31d33b64f1827932fb3fda0ce361def535f143c3561eefac7bc66facd4f0a7ec31d33b64f1827932fb3fda0ce361def535f00",
)

// given the extrinsic "parts", it returns an encoded `Proof`
const proof2: Uint8Array = merkleizedMetadata.getProofForExtrinsicParts(
  // Call data
  "0x040300648ad065ea416ca1725c29979cd41e288180f3e8aefde705cd3e0bab6cd212010bcb04fb711f01",
  // Signed Extension data included in the extrinsic
  "0x2503000000",
  // Signed Extension data included in the signature
  "0x164a0f001a000000b0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe878a023bcb37967b6ba0685d002bb74e6cf3b4fc4ae37eb85f756bd9b026bede00",
)

// The type `Proof` definition is as follows:
// interface Proof {
//   leaves: Array<LookupEntry>,
//   leafIdxs: Array<number>,
//   proofs: Array<Uint8Array>,
//   extrinsic: ExtrinsicMetadata,
//   info: ExtraInfo
// }
```
