# @polkadot-api/ws-provider

`JsonRpcProvider` to connect to a chain via a WebSocket.

```ts
function WebSocketProvider(
  uri: string,
  protocols?: string | string[],
): JsonRpcProvider
```

This package doesn't have any export on the root. Instead it has 2 subpaths, each for a different environment.

For web runtimes, where the browser's `WebSocket` constructor can be used, import `WebSocketProvider` from `@polkadot-api/ws-provider/web`.

For other runtimes where the `WebSocket` from the module `ws` should be used instead, import `WebSocketProvider` from `@polkadot-api/ws-provider/node`.
