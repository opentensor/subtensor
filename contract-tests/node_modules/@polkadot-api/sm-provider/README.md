# @polkadot-api/sm-provider

`JsonRpcProvider` to connect to a chain via Smoldot.

```ts
function getSmProvider(
  chain: smoldot.Chain | Promise<smoldot.Chain>,
): JsonRpcProvider
```

The parameter it takes is a `Chain` from smoldot, the return value of calling `smoldot.addChain(..)`
