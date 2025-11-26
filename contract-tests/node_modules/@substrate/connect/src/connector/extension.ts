import type { Chain, JsonRpcCallback, ScClient } from "./types.js"
import type {
  RawChain,
  LightClientProvider,
} from "@substrate/light-client-extension-helpers/web-page"
import { WellKnownChain } from "../WellKnownChain.js"

const wellKnownChainGenesisHashes: Record<string, string> = {
  polkadot:
    "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3",
  ksmcc3: "0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe",
  westend2:
    "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e",
  rococo_v2_2:
    "0x6408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e",
}

/**
 * Returns a {@link ScClient} that connects to chains by asking the substrate-connect extension
 * to do so.
 *
 * This function assumes that the extension is installed and available. It is out of scope of this
 * function to detect whether this is the case.
 * If you try to add a chain without the extension installed, nothing will happen and the
 * `Promise`s will never resolve.
 */
export const createScClient = (
  lightClientProviderPromise: Promise<LightClientProvider>,
): ScClient => {
  const internalAddChain = async (
    isWellKnown: boolean,
    chainSpecOrWellKnownName: string,
    jsonRpcCallback: JsonRpcCallback = () => {},
    relayChainGenesisHash?: string,
  ): Promise<Chain> => {
    const lightClientProvider = await lightClientProviderPromise

    let chain: RawChain
    if (isWellKnown) {
      const foundChain = Object.values(lightClientProvider.getChains()).find(
        ({ genesisHash }) =>
          genesisHash === wellKnownChainGenesisHashes[chainSpecOrWellKnownName],
      )
      if (!foundChain) throw new Error("Unknown well-known chain")
      chain = foundChain
    } else {
      chain = await lightClientProvider.getChain(
        chainSpecOrWellKnownName,
        relayChainGenesisHash,
      )
    }

    const jsonRpcProvider = chain.connect(jsonRpcCallback)

    return {
      sendJsonRpc(rpc: string): void {
        jsonRpcProvider.send(rpc)
      },
      remove() {
        jsonRpcProvider.disconnect()
      },
      addChain: function (
        chainSpec: string,
        jsonRpcCallback?: JsonRpcCallback | undefined,
      ): Promise<Chain> {
        return internalAddChain(
          false,
          chainSpec,
          jsonRpcCallback,
          chain.genesisHash,
        )
      },
    }
  }

  return {
    addChain: (chainSpec: string, jsonRpcCallback?: JsonRpcCallback) =>
      internalAddChain(false, chainSpec, jsonRpcCallback),
    addWellKnownChain: (
      name: WellKnownChain,
      jsonRpcCallback?: JsonRpcCallback,
    ) => internalAddChain(true, name, jsonRpcCallback),
  }
}
