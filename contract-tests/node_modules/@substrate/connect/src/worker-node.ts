// @ts-ignore TODO: fix types in smoldot/worker
import * as smoldot from "smoldot/worker"
// @ts-ignore TODO: fix types in smoldot/bytecode
import { compileBytecode } from "smoldot/bytecode"
import { parentPort } from "node:worker_threads"

compileBytecode().then((bytecode: any) => parentPort!.postMessage(bytecode))

parentPort!.once("message", (data) =>
  smoldot
    .run(data)
    .catch((error: any) => console.error("[smoldot-worker]", error))
    .finally(() => process.exit()),
)
