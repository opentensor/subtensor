# @polkadot-api/json-rpc-provider-proxy

This package exports `getSyncProvider`, a function to create `JsonRpcProvider`s that will act as if the connection happen synchronously.

```ts
export type AsyncJsonRpcProvider = (
  onMessage: (message: string) => void,
  onHalt: () => void,
) => JsonRpcConnection

function getSyncProvider(
  input: () => Promise<AsyncJsonRpcProvider>,
): JsonRpcProvider
```

The returned provider will buffer up every message until it can get the `JsonRpcConnection`, at which point it will send every message buffered.
