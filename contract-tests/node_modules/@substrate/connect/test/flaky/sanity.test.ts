import { expect, test } from "vitest"
import { createClient } from "@polkadot-api/substrate-client"
import { getSyncProvider } from "@polkadot-api/json-rpc-provider-proxy"

import { createScClient, ScClient, WellKnownChain } from "../../src"

test.each([
  [WellKnownChain.polkadot],
  [WellKnownChain.ksmcc3],
  [WellKnownChain.westend2],
  [WellKnownChain.rococo_v2_2],
])(
  "WellKnownChain.%s should emit finalized events",
  async (chainSpec) => {
    const scClient = createScClient()
    const provider = ScProvider(scClient, chainSpec)
    const { chainHead, destroy } = createClient(provider)
    const count = await new Promise<number>((resolve, reject) => {
      let count = 0
      const chainHeadFollower = chainHead(
        true,
        (event) => {
          if (event.type === "finalized" && ++count === 2) {
            chainHeadFollower.unfollow()
            destroy()
            resolve(count)
          }
        },
        reject,
      )
    })
    expect(count).toBe(2)
  },
  { timeout: 300_000 },
)

const noop = () => {}
const ScProvider = (client: ScClient, input: WellKnownChain) =>
  getSyncProvider(async () => {
    let listener: (message: string) => void = noop
    const onMessage = (msg: string) => listener(msg)
    const chain = await client.addWellKnownChain(input, onMessage)
    return (onMessage) => {
      listener = onMessage
      return {
        send(msg: string) {
          chain.sendJsonRpc(msg)
        },
        disconnect() {
          listener = noop
          chain.remove()
        },
      }
    }
  })
