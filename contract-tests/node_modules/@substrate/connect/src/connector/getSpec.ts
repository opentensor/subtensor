import { WellKnownChain } from "../WellKnownChain.js"

const chains: Map<WellKnownChain, Promise<{ chainSpec: string }>> = new Map()

export async function getSpec(chain: string): Promise<string> {
  if (!Object.keys(WellKnownChain).includes(chain))
    throw new Error("Invalid chain name")

  const knownChain = chain as WellKnownChain
  if (!chains.has(knownChain))
    // Dynamic imports needs to be explicit for ParcelJS
    // See https://github.com/parcel-bundler/parcel/issues/125
    switch (knownChain) {
      case WellKnownChain.polkadot: {
        chains.set(
          WellKnownChain.polkadot,
          import("@substrate/connect-known-chains/polkadot"),
        )
        break
      }
      case WellKnownChain.ksmcc3: {
        chains.set(
          WellKnownChain.ksmcc3,
          import("@substrate/connect-known-chains/ksmcc3"),
        )
        break
      }
      case WellKnownChain.westend2: {
        chains.set(
          WellKnownChain.westend2,
          import("@substrate/connect-known-chains/westend2"),
        )
        break
      }
      case WellKnownChain.rococo_v2_2: {
        chains.set(
          WellKnownChain.rococo_v2_2,
          import("@substrate/connect-known-chains/rococo_v2_2"),
        )
        break
      }
    }

  return (await chains.get(knownChain)!).chainSpec
}
