# @substrate/connect

## Using `@substrate/connect` for library authors

Provide a well-known chain name ('polkadot', 'ksmcc3', 'westend2', 'rococo_v2_2'):

```js
import { createScClient, WellKnownChain } from '@substrate/connect';

const scClient = createScClient();
const chain = await scClient.addWellKnownChain(
  WellKnownChain.westend2,
  function jsonRpcCallback(response) {
    console.log("response", response);
  }
);

chain.sendJsonRpc(
  '{"jsonrpc":"2.0","id":"1","method":"system_health","params":[]}'
);
```

...or provide your custom substrate chain's name and chainspec:

```js
import { createScClient } from '@substrate/connect';
import myJsonChainSpec from './mySubstrateChainSpec.json';

const myChainSpec = JSON.stringify(myJsonChainSpec);

const scClient = createScClient();
const chain = await scClient.addChain(
  myChainSpec,
  function jsonRpcCallback(response) {
    console.log("response", response);
  }
);

chain.sendJsonRpc(
  '{"jsonrpc":"2.0","id":"1","method":"system_health","params":[]}'
);
```

In order to connect to a parachain, you must first instantiate the relay chain
this parachain is connected to, then instantiate the parachain on the same
relay chain. The following example connects to a parachain of the Westend test
network:

```js
import { createScClient, WellKnownChain } from '@substrate/connect';
import jsonParachainSpec from './myParaChainSpec.json';

const parachainSpec = JSON.stringify(jsonParachainSpec);

const scClient = createScClient();
const relayChain = await scClient.addWellKnownChain(WellKnownChain.westend2)
const parachain = await relayChain.addChain(
  parachainSpec,
  function jsonRpcCallback(response) {
    console.log("response", response);
  }
);

parachain.sendJsonRpc(
  '{"jsonrpc":"2.0","id":"1","method":"system_health","params":[]}'
);
```

## Scripts

* `pnpm test` to run the unit tests
* `pnpm build` to build @substrate-connect
* `pnpm lint` to run linter for @substrate-connect
