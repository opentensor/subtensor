/// <reference lib="WebWorker" />

// @ts-ignore TODO: fix types in smoldot/worker
import * as smoldot from "smoldot/worker"
// @ts-ignore TODO: fix types in smoldot/bytecode
import { compileBytecode } from "smoldot/bytecode"

declare var self: DedicatedWorkerGlobalScope

compileBytecode().then((bytecode: unknown) => self.postMessage(bytecode))

self.onmessage = ({ data }) =>
  smoldot
    .run(data)
    .catch((error: any) => console.error("[smoldot-worker]", error))
    .finally(self.close)
