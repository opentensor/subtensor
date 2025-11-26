# @polkadot-api/metadata-builders

This package has multiple functions that read a metadata object, denormalizes it, and builds other structures needed for different processes

## getLookupFn

```ts
function getLookupFn(
  lookupData: Metadata["lookup"],
): (id: number) => LookupEntry
```

Given the lookup property of a metadata, returns a function that will give the `LookupEntry` for an id.

The `LookupEntry` is a denormalized data structure for one entry in the metadata. It also "shortcuts" type references when those are pointers (composites or tuples of length 1). Essentially, it's a union of each of the different types that can be found in the lookup, mostly equivalent to something like:

```ts
type TerminalVar =
  | PrimitiveVar // u8, str, char, i128, etc.
  | CompactVar
  | BitSequenceVar
  | AccountId32

type ComposedVar =
  | TupleVar
  | StructVar
  | SequenceVar
  | ArrayVar
  | OptionVar
  | ResultVar
  | EnumVar

type LookupEntry = TerminalVar | ComposedVar
```

Where, for instance, a StructVar is of the shape

```ts
type StructVar = {
  type: "struct"
  value: Record<string, LookupEntry>
}
```

It's useful to get types referenced by storage calls, etc.

## getDynamicBuilder

```ts
function getDynamicBuilder(lookupData: Metadata): {
  buildDefinition: (id: number) => Codec
  buildConstant: (pallet: string, name: string) => Codec
  buildEvent: (pallet: string, name: string) => VariantEntry
  buildError: (pallet: string, name: string) => VariantEntry
  buildCall: (pallet: string, name: string) => VariantEntry
  buildStorage: (pallet: string, entry: string) => StorageEntry
  buildRuntimeCall: (api: string, method: string) => RuntimeEntry
}
```

Generates all the codecs needed to SCALE encode or decode the data for any interaction with the chain.

`buildDefinition` returns the codec for the type identified by the parameter `id`

`buildConstant` returns the codec for the requested constant (equivalent as calling `buildDefinition` with the type id of that constant)

`buildEvent`, `buildError` and `buildCall` return an object with the codec, and the indices of the pallet and entry within the metadata:

```ts
interface VariantEntry {
  location: [number, number] // [palletIdx, entryIdx],
  codec: Codec
}
```

`buildStorage` creates all the encoders/decoders needed to encode a storage call and decode its result:

```ts
interface StorageEntry {
  // Encodes the arguments of the storage call.
  enc: (...args: any[]) => string
  // Decodes the result from the storage call.
  dec: (value: string) => any
  // Decodes the arguments of the storage call
  keyDecoder: (value: string) => any[]
  // Expected number of arguments
  len: number
  // Decoded fallback value as defined in the metadata entry
  fallback: unknown
}
```

Similarly, `buildRuntimeCall` returns the codecs for both encoding the arguments of the runtime call, and the codec for decoding the result

```ts
interface RuntimeEntry {
  args: Codec<any[]>
  value: Codec<any>
}
```

## getChecksumBuilder

```ts
function getChecksumBuilder(metadata: Metadata): {
  buildDefinition: (id: number) => string | null
  buildRuntimeCall: (api: string, method: string) => string | null
  buildStorage: (pallet: string, entry: string) => string | null
  buildCall: (pallet: string, name: string) => string | null
  buildEvent: (pallet: string, name: string) => string | null
  buildError: (pallet: string, name: string) => string | null
  buildConstant: (pallet: string, constantName: string) => string | null
}
```

Generates the checksums for the different components defined in the metadata.

`buildDefinition` builds the checksum of one of the types in the lookup. The rest of the methods build the checksum for each of the interfaces of the chain.
