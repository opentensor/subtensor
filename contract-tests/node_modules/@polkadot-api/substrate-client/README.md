# @polkadot-api/substrate-client

This TypeScript package provides low-level bindings to the [Substrate JSON-RPC Interface](https://paritytech.github.io/json-rpc-interface-spec/introduction.html), enabling interaction with Substrate-based blockchains.

## Usage

Start by creating a `SubstrateClient` object with the exported function `createClient`. To create one, you need a `ConnectProvider` provider defined in [@polkadot-api/json-rpc-provider](https://github.com/polkadot-api/polkadot-api/tree/main/packages/json-rpc-provider) for establishing a connection to a specific blockchain client.

For instance, you can use [@polkadot-api/sc-provider](https://github.com/polkadot-api/polkadot-api/tree/main/packages/sc-provider) to get a substrate-connect provider for connecting to the Polkadot relay chain through a light client:

```ts
import { getScProvider, WellKnownChain } from "@polkadot-api/sc-provider"
import { createClient } from "@polkadot-api/substrate-client"

const scProvider = getScProvider()
const { relayChain } = scProvider(WellKnownChain.polkadot)

const client = createClient(relayChain)
```

### Request

Invoke any method defined in the [JSON-RPC Spec](https://paritytech.github.io/json-rpc-interface-spec/introduction.html) using `client.request(method, params, abortSignal?)`. This returns a promise resolving with the response from the JSON-RPC server.

```ts
const genesisHash = await client.request("chainSpec_v1_genesisHash", [])
```

All promise-returning functions exported by this package accept an [AbortSignal](https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal) for operation cancellation.

### ChainHead

Operations within the [`chainHead` group of functions](https://paritytech.github.io/json-rpc-interface-spec/api/chainHead.html) involve subscriptions and interdependencies between methods. The client has a function that simplifies the interaction with these group.

Calling `client.chainHead(withRuntime, onFollowEvent, onFollowError)` will start a `chainHead_unstable_follow` subscription, and will return a handle to perform operations with the chainHead.

```ts
const chainHead = client.chainHead(
  true,
  (event) => {
    // ...
  },
  (error) => {
    // ...
  },
)
```

The handle provides one method per each of the functions defined inside `chainHead`: `chainHead_unstable_body`, `chainHead_unstable_call`, `chainHead_unstable_header`, `chainHead_unstable_storage`, and `chainHead_unstable_unpin`.

The JSON-RPC Spec for chainHead specifies that these functions return an `operationId`, and that the resolved response for the call will come through the `chainHead_unstable_follow` subscription, linking it through this `operationId`.

**`substrate-client`'s chainHead is an abstraction over this**: The events emitted through the `client.chainHead()` callback are only the ones initiated from the JSON-RPC Server. The promise returned by any of the `chainHead`'s handle functions will resolve with the respective event.

```ts
const chainHead = client.chainHead(
  true,
  async (event) => {
    if (event.type === "newBlock") {
      const body = await chainHead.body(event.blockHash)
      // body is a string[] containing the SCALE-encoded values within the body
      processBody(body)

      chainHead.unpin([event.blockHash])
    }
  },
  (error) => {
    // ...
  },
)
```

#### header

Calls `chainHead_unstable_call` and returns a promise that resolves with the SCALE-encoded header of the block

```ts
const header = await chainHead.header(blockHash)
```

#### body

Calls `chainHead_unstable_body` and returns a promise that will resolve with an array of strings containing the SCALE-encoded extrinsics found in the block

```ts
const body = await chainHead.body(blockHash)
```

#### call

Calls `chainHead_unstable_header` and returns a promise that resolves with the encoded output of the runtime function call

```ts
const result = await chainHead.call(blockHash, fnName, callParameters)
```

#### storage

Calls `chainHead_unstable_storage` and returns a promise that resolves with the value returned by the JSON-RPC server, which depends on the `type` parameter. See the [JSON-RPC spec for chainHead_unstable_storage](https://paritytech.github.io/json-rpc-interface-spec/api/chainHead_unstable_storage.html) for the details on the usage.

```ts
// string with the SCALE-encoded value
const value = await chainHead.storage(blockHash, "value", key, childTrie)

// string with the hash value
const hash = await chainHead.storage(blockHash, "hash", key, childTrie)

// string with the merkle value
const items = await chainHead.storage(
  blockHash,
  "closestDescendantMerkleValue",
  key,
  childTrie,
)

// array of key-value pairs
const items = await chainHead.storage(
  blockHash,
  "descendantsValues",
  key,
  childTrie,
)

// array of key-hash pairs
const hashes = await chainHead.storage(
  blockHash,
  "descendantsHashes",
  key,
  childTrie,
)
```

#### storageSubscription

While `storage` only can resolve for one specific item, the JSON-RPC specification allows to resolve multiple items within the same call. For this case, substrate-client also offers a lower-level version called `chainHead.storageSubscription(hash, inputs, childTrie, onItems, onError, onDone, onDiscardedItems)` that emits the storage items as they get resolved by the JSON-RPC server:

```ts
const abort = chainHead.storageSubscription(
  hash,
  [
    { key, type },
    /* ... each item */
  ],
  null,
  (items) => {
    // items is an array of { key, value?, hash?, closestDescendantMerkleValue? }
  },
  onError,
  onDone,
  (nDiscardedItems) => {
    // amount of discarded items, as defined by the JSON-RPC spec.
  },
)
```

`storageSubscription` returns a function to cancel the operation.

#### unpin

Calls `chainHead_unstable_unpin` and returns a promise that will resolve after the operation is done.

```ts
chainHead.unpin(blockHashes)
```

#### unfollow

To close the chainHead subscription, call `chainHead.unfollow()`.

### Transaction

[`transaction` group of functions](https://paritytech.github.io/json-rpc-interface-spec/api/transaction.html) also deals with subscriptions through `submitAndWatch`. SubstrateClient also abstracts over this:

```ts
const cancelRequest = client.transaction(
  transaction, // SCALE-encoded transaction
  (event) => {
    // ...
  },
  (error) => {
    // ...
  },
)

// call `cancelRequest()` to abort the transaction (`transaction_unstable_stop`)
```

The `event` emitted through the callback are fully typed, and can be discriminated through `event.type`

```ts
switch (event.type) {
  case "validated":
    break
  case "broadcasted":
    const { numPeers } = event
    break
  case "bestChainBlockIncluded":
  case "finalized":
    const { block } = event
    break
  case "dropped":
  case "error":
  case "invalid":
    const { error } = event
    break
}
```

### Destroy

Call `client.destroy()` to disconnect from the provider.
