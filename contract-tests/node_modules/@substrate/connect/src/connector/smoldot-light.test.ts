// eslint-disable-next-line import/no-extraneous-dependencies
import { beforeEach, beforeAll, it, describe, expect, vi } from "vitest"
import type { AddChainOptions, ClientOptions } from "smoldot"
import { WellKnownChain } from "../WellKnownChain.js"

class SdAlreadyDestroyedError extends Error {
  constructor() {
    super()
    this.name = "AlreadyDestroyedError"
  }
}

class SdCrashError extends Error {
  constructor(message: string) {
    super(message)
    this.name = "CrashError"
  }
}

class SdJsonRpcDisabledError extends Error {
  constructor() {
    super()
    this.name = "JsonRpcDisabledError"
  }
}

class SdMalformedJsonRpcError extends Error {
  constructor() {
    super()
    this.name = "MalformedJsonRpcError"
  }
}

class SdQueueFullError extends Error {
  constructor() {
    super()
    this.name = "QueueFullError"
  }
}

const mockSmoldotLightFactory = () => {
  const start = (options: ClientOptions) => {
    const addChain = (
      addChainOptions: AddChainOptions,
      isClientTerminated: () => boolean,
    ) => {
      let _remove = vi.fn()
      let _sendJsonRpc = vi.fn()
      return {
        _addChainOptions: addChainOptions,
        remove() {
          if (isClientTerminated()) throw new SdAlreadyDestroyedError()
          _remove()
        },
        _setRemove(nextRemove: typeof _remove) {
          _remove = nextRemove
        },
        _getSendRemove: () => _remove,
        sendJsonRpc(rpc: string) {
          if (isClientTerminated()) throw new SdAlreadyDestroyedError()
          _sendJsonRpc(rpc)
        },
        _setSendJsonRpc(nextSendJsonRpc: typeof _sendJsonRpc) {
          _sendJsonRpc = nextSendJsonRpc
        },
        _getSendJsonRpc: () => _sendJsonRpc,
      }
    }
    type MockChain = ReturnType<typeof addChain>
    const chains: MockChain[] = []

    const terminate = vi.fn()

    return {
      _options: options,
      _getChains: () => chains,
      _getLatestChain: () => chains?.[chains.length - 1],
      terminate,
      addChain: (addChainOptions: AddChainOptions) => {
        const chain = addChain(
          addChainOptions,
          () => terminate.mock.calls.length > 0,
        )
        chains.push(chain)
        return Promise.resolve(chain)
      },
    }
  }
  type MockClient = ReturnType<typeof start>

  let latestClient: MockClient
  const mock = {
    AlreadyDestroyedError: SdAlreadyDestroyedError,
    CrashError: SdCrashError,
    JsonRpcDisabledError: SdJsonRpcDisabledError,
    MalformedJsonRpcError: SdMalformedJsonRpcError,
    QueueFullError: SdQueueFullError,
    start: (options: ClientOptions) => {
      return (latestClient = start(options))
    },
    getLatestClient: () => latestClient,
  }
  return mock
}

vi.doMock("smoldot", mockSmoldotLightFactory)
vi.doMock("./specs/index.js", () => ({
  getSpec: (wellKnownChain: string) => `fake-${wellKnownChain}-spec`,
}))

type MockSmoldotLight = ReturnType<typeof mockSmoldotLightFactory>
let mockedSmoldotLight: MockSmoldotLight

beforeAll(async () => {
  mockedSmoldotLight = mockSmoldotLightFactory as unknown as MockSmoldotLight
})

beforeEach(() => {
  vi.resetModules()
})

describe("SmoldotConnect::smoldot", () => {
  describe("client", () => {
    it("does not eagerly instantiate the client", () => {
      import("./smoldot-light.js").then((smoldot) => {
        smoldot.createScClient()
        mockedSmoldotLight =
          mockSmoldotLightFactory as unknown as MockSmoldotLight

        expect(mockedSmoldotLight.start).toBeUndefined()
      })
    })

    it("terminates the internal client when all the chains, from all clients, have been removed", () => {
      import("./smoldot-light.js").then(async (smoldot) => {
        const { addWellKnownChain, addChain } = smoldot.createScClient()

        let chain1 = await addWellKnownChain("" as WellKnownChain)
        const client = mockedSmoldotLight?.getLatestClient()
        let chain2 = await addChain("")
        expect(client).toBe(mockedSmoldotLight.getLatestClient())

        chain1.remove()
        expect(client.terminate).not.toHaveBeenCalled()

        chain2.remove()
        expect(client.terminate).toHaveBeenCalled()
        let chain3 = await addWellKnownChain("" as WellKnownChain)
        expect(mockedSmoldotLight.getLatestClient()).not.toBe(client)
        expect(
          mockedSmoldotLight.getLatestClient().terminate,
        ).not.toHaveBeenCalled()
        chain3.remove()
        expect(
          mockedSmoldotLight.getLatestClient().terminate,
        ).toHaveBeenCalled()
      })
    })

    it("handles race conditions on the client when adding/removing chains", () => {
      import("./smoldot-light.js").then(async (smoldot) => {
        const { addChain } = smoldot.createScClient()
        const { addChain: addChain2 } = smoldot.createScClient()

        let chain1 = await addChain("")
        const client = mockedSmoldotLight.getLatestClient()
        const chain2Promise = addChain2("")
        chain1.remove()
        expect(client.terminate).not.toHaveBeenCalled()

        let chain2 = await chain2Promise
        chain2.remove()
        expect(client.terminate).toHaveBeenCalled()
      })
    })
  })

  describe("chain", () => {
    it("propagates the correct chainSpec to smoldot", () => {
      import("./smoldot-light.js").then(async (smoldot) => {
        const { addChain, addWellKnownChain } = smoldot.createScClient()
        const chainSpec = "testChainSpec"
        await addChain(chainSpec)
        let mockedChain = mockedSmoldotLight
          .getLatestClient()
          ._getLatestChain()!
        expect(mockedChain._addChainOptions.chainSpec).toEqual(chainSpec)

        await addWellKnownChain(WellKnownChain.polkadot)
        mockedChain = mockedSmoldotLight.getLatestClient()._getLatestChain()!
        expect(mockedChain._addChainOptions.chainSpec).toEqual(
          "fake-polkadot-spec",
        )

        await addWellKnownChain(WellKnownChain.ksmcc3)
        mockedChain = mockedSmoldotLight.getLatestClient()._getLatestChain()!
        expect(mockedChain._addChainOptions.chainSpec).toEqual(
          "fake-ksmcc3-spec",
        )

        await addWellKnownChain(WellKnownChain.rococo_v2_2)
        mockedChain = mockedSmoldotLight.getLatestClient()._getLatestChain()!
        expect(mockedChain._addChainOptions.chainSpec).toEqual(
          "fake-rococo_v2_2-spec",
        )
      })
    })

    it("propagates the correct potentialRelayChainIds to smoldot", () => {
      import("./smoldot-light.js").then(async (smoldot) => {
        const { addChain } = smoldot.createScClient()
        let prevChains = await Promise.all(
          Array(3)
            .fill(null)
            .map(() => addChain("")),
        )
        prevChains[0]!.remove()
        await addChain("")
        const mockedChains = mockedSmoldotLight
          .getLatestClient()
          ._getChains()
          .slice(-4)
        const lastMockedChain = mockedChains[3]
        expect(lastMockedChain!._addChainOptions.potentialRelayChains).toEqual([
          mockedChains[1],
          mockedChains[2],
        ])
      })
    })
  })
})
