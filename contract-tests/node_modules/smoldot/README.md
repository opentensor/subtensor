# Light client for Polkadot and Substrate-based chains

This JavaScript library provides a light client for
[the Polkadot blockchain](https://polkadot.network/) and for chains built
using [the Substrate blockchain framework](https://substrate.io/).

It is an "actual" light client, in the sense that it is byzantine-resilient.
It does not rely on the presence of an RPC server, but directly connects to
the full nodes of the network.

## Example

```
import * as smoldot from 'smoldot';

// Load a string chain specification.
const chainSpec = fs.readFileSync('./westend.json', 'utf8');

// A single client can be used to initialize multiple chains.
const client = smoldot.start();

const chain = await client.addChain({ chainSpec });

chain.sendJsonRpc('{"jsonrpc":"2.0","id":1,"method":"system_name","params":[]}');

// Wait for a JSON-RPC response to come back. This is typically done in a loop in the background.
const jsonRpcResponse = await chain.nextJsonRpcResponse();
console.log(jsonRpcResponse)

// Later:
// chain.remove();
```

## Usage

The first thing to do is to initialize the client with the `start` function.

Once initialized, the client can be used to connect to one or more chains. Use `addChain` to add
a new chain that the client must be connected to. `addChain` must be passed the specification of
the chain (commonly known as "chain spec").

The `addChain` function returns a `Promise` that yields a chain once the chain specification has
been successfully parsed and basic initialization is finished, but before Internet connections
are opened towards the chains.

In order to de-initialize a chain, call `chain.remove()`. Any function called afterwards on this
chain will throw an exception.
In order to de-initialize a client, call `client.terminate()`. Any function called afterwards on
any of the chains of the client will throw an exception.

After having obtained a chain, use `sendJsonRpc` to send a JSON-RPC request towards the node.
The function accepts as parameter a string request. See
[the specification of the JSON-RPC protocol](https://www.jsonrpc.org/specification),
and [the list of requests that smoldot is capable of serving](https://polkadot.js.org/docs/substrate/rpc/).
Smoldot also has experimental support for an extra (still experimental at the time of writing of
this comment) set of JSON-RPC functions [found here](https://github.com/paritytech/json-rpc-interface-spec/).

If the request is well formatted, the client will generate a response. This response can be pulled
using the `nextJsonRpcResponse` asynchronous function. Calling this function waits until a response
is available and returns it.

If the request is a subscription, the notifications will also be sent back using the same mechanism
and can be pulled using `nextJsonRpcResponse`.

If the chain specification passed to `addChain` is a parachain, then the list of potential relay
chains must be passed as parameter to `addChain` as well. In situations where the chain
specifications passed to `addChain` are not trusted, it is important for security reasons to not
establish a parachain-relay-chain link between two chains that aren't part of the same "trust
sandbox".

## Usage with a worker

By default, calling `start()` will run smoldot entirely in the current thread. This can cause
performance issues if other CPU-heavy operations are done in that thread.

In order to help with this, it is possible to use smoldot in conjunction with a worker.
To do so, you must first create a worker. Since creating a worker has some subtle differences
depending on the platform, this is outside of the responsibility of smoldot.

Once the worker is created, create two `MessagePort`s using `new MessageChannel`, and send one
of them to the worker. Then, pass one port to the `ClientOptions.portToWorker` field and the
other port to the `run()` function of smoldot, which can be imported with
`import { run } from 'smoldot/worker';` (on Deno, it is found in `worker-deno.ts`).

Another optimization that is orthogonal to but is related to running smoldot in a worker consists
in also loading the smoldot bytecode in that worker. The smoldot bytecode weights several
megabytes, and loading it in a worker rather than the main thread makes it possible to load the
UI while smoldot is still initializing. This is especially important when smoldot is included in
an application served over the web.

In order to load the smoldot bytecode in a worker, import `compileBytecode` with
`import { compileBytecode } from 'smoldot/bytecode';` (on Deno: `bytecode-deno.ts`), then call the
function and send the result to the main thread. From the main thread, rather than using the
`start` function imported from `smoldot`, use the `startWithBytecode` function that can be imported
using `import { startWithBytecode } from 'smoldot/no-auto-bytecode';` (on Deno:
`no-auto-bytecode-deno.ts`). The options provided to `startWithBytecode` are the same as the ones
passed to `start`, except for an additional `bytecode` field that must be set to the bytecode
created in the worker.

Here is an example of all this, assuming a browser environment:

```ts
import * as smoldot from 'smoldot/no-auto-bytecode';

const worker = new Worker(new URL('./worker.js', import.meta.url));

const bytecode = new Promise((resolve) => {
    worker.onmessage = (event) => resolve(event.data);
});

const { port1, port2 } = new MessageChannel();
worker.postMessage(port1, [port1]);

const client = smoldot.startWithBytecode({
    bytecode,
    portToWorker: port2,
});


// `worker.ts`

import * as smoldot from 'smoldot/worker';
import { compileBytecode } from 'smoldot/bytecode';

compileBytecode().then((bytecode) => postMessage(bytecode))
onmessage = (msg) => smoldot.run(msg.data);
```

Note that importing sub-paths (for example importing `smoldot/worker`) relies on a relatively
modern JavaScript feature. If you import a smoldot sub-path from a TypeScript file, you might have
to configure TypeScript to use `"moduleResolution": "node16"`. [The official TypeScript
documentation itself recommends setting this configuration option to
`node`](https://www.typescriptlang.org/docs/handbook/module-resolution.html#module-resolution-strategies),
and it is likely that `node16` becomes the go-to module resolution scheme in the future.
