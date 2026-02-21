
import * as assert from "assert";
import { getDevnetApi, waitForTransactionWithRetry, getRandomSubstrateKeypair } from "../src/substrate"
import { getPublicClient } from "../src/utils";
import { convertPublicKeyToSs58 } from "../src/address-utils"
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { getPolkadotSigner } from "polkadot-api/signer";
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { forceSetBalanceToSs58Address, forceSetChainID } from "../src/subtensor";

describe("Test the EVM chain ID", () => {
  let ethClient: PublicClient;

  // init substrate part
  const keyPair = getRandomSubstrateKeypair();
  let api: TypedApi<typeof devnet>;

  // Default Subtensor EVM chain id is often 42 or 943 (testnet)
  // But for devnet/local it usually starts at a fixed value.
  let initChainId: bigint;

  before(async () => {
    ethClient = await getPublicClient(ETH_LOCAL_URL);
    api = await getDevnetApi()
    await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(keyPair.publicKey))
    initChainId = await ethClient.getChainId();
  });

  it("EVM chain id update via sudo_set_evm_chain_id is reflected in RPC", async () => {
    const newChainId = BigInt(100);
    
    // Use the helper to set chain ID (which likely uses sudo internally)
    await forceSetChainID(api, newChainId);

    const updatedChainId = await ethClient.getChainId();
    assert.equal(updatedChainId, newChainId, "Chain ID should be updated to 100");

    // Reset back to original
    await forceSetChainID(api, initChainId);
    const finalChainId = await ethClient.getChainId();
    assert.equal(finalChainId, initChainId, "Chain ID should be reset to original value");
  });

  it("EVM chain id remains unchanged if non-sudo user attempts update", async () => {
    const currentChainId = await ethClient.getChainId();
    const targetChainId = BigInt(999);

    // Create a regular user signer (non-sudo)
    let signer = getPolkadotSigner(
      keyPair.publicKey,
      "Sr25519",
      keyPair.sign,
    );

    // Attempt to call sudo_set_evm_chain_id directly without sudo wrapper
    // In Substrate, calling a sudo-protected dispatchable directly usually fails at dispatch
    let tx = api.tx.AdminUtils.sudo_set_evm_chain_id({ chain_id: targetChainId });
    
    try {
        await waitForTransactionWithRetry(api, tx, signer);
    } catch (e) {
        // Expected to fail or just not update
    }

    const checkChainId = await ethClient.getChainId();
    assert.equal(checkChainId, currentChainId, "Chain ID should NOT have changed");
  });
});
