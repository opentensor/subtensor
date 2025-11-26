# @polkadot-api/wasm-executor

This package has been strongly influenced by [Chopsticks](https://github.com/AcalaNetwork/chopsticks).
It can be used in both web and NodeJS environments.

At this point, it just exports a `getMetadataFromRuntime` function that runs WASM under the hood, and returns its metadata in `string`.

## Examples

### Web

```ts
// IMPORTANT to prefix it with `0x`!
const runtime = "0x" + fs.readFileSync("runtime.wasm").toString("hex");

// returns a `0x` prefixed OpaqueMetadata. It's as well prefixed by a compactInt of its length
const metadata = await import("@polkadot-api/wasm-executor/web").then(
  async ({ default: init, getMetadataFromRuntime }) => {
    await init();
    return getMetadataFromRuntime(runtime);
  },
);
```

### Node

```ts
import { getMetadataFromRuntime } from "@polkadot-api/wasm-executor/node";

// IMPORTANT to prefix it with `0x`!
const runtime = "0x" + fs.readFileSync("runtime.wasm").toString("hex");

// returns a `0x` prefixed OpaqueMetadata. It's as well prefixed by a compactInt of its length
const metadata = getMetadataFromRuntime(runtime);
```
