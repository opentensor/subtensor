import * as assert from "assert";

import {
  convertPublicKeyToMultiAddress,
  getAliceSigner,
  getDevnetApi,
  getRandomSubstrateKeypair,
  getSignerFromKeypair,
  waitForTransactionWithRetry,
} from "../src/substrate";
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors";
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, toViemAddress } from "../src/address-utils";
import { IMETAGRAPH_ADDRESS, IMetagraphABI } from "../src/contracts/metagraph";
import {
  addNewSubnetwork,
  burnedRegister,
  forceSetBalanceToSs58Address,
} from "../src/subtensor";

describe("Test the Metagraph precompile", () => {
  // init substrate part
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  let publicClient: PublicClient;

  let api: TypedApi<typeof devnet>;

  // sudo account alice as signer
  let alice: PolkadotSigner;

  // init other variable
  let subnetId = 0;

  before(async () => {
    // init variables got from await and async
    publicClient = await getPublicClient(ETH_LOCAL_URL);
    api = await getDevnetApi();
    alice = await getAliceSigner();

    await forceSetBalanceToSs58Address(
      api,
      convertPublicKeyToSs58(hotkey.publicKey),
    );
    await forceSetBalanceToSs58Address(
      api,
      convertPublicKeyToSs58(coldkey.publicKey),
    );

    const netuid = await addNewSubnetwork(api, hotkey, coldkey);
    console.log("test on subnet ", netuid);
    await burnedRegister(
      api,
      netuid,
      convertPublicKeyToSs58(hotkey.publicKey),
      coldkey,
    );
  });

  it("Metagraph data access via precompile contract is ok", async () => {
    const uid = 0;
    const uid_count = await publicClient.readContract({
      abi: IMetagraphABI,
      address: toViemAddress(IMETAGRAPH_ADDRESS),
      functionName: "getUidCount",
      args: [subnetId],
    });
    // back to original value for other tests. and we can run it repeatedly
    assert.ok(uid_count != undefined);

    // const axon = api.query.SubtensorModule.Axons.getValue()

    const axon = await publicClient.readContract({
      abi: IMetagraphABI,
      address: toViemAddress(IMETAGRAPH_ADDRESS),
      functionName: "getAxon",
      args: [subnetId, uid],
    });

    assert.ok(axon != undefined);
    if (axon instanceof Object) {
      assert.ok(axon != undefined);
      if ("block" in axon) {
        assert.ok(axon.block != undefined);
      } else {
        throw new Error("block not included in axon");
      }

      if ("version" in axon) {
        assert.ok(axon.version != undefined);
      } else {
        throw new Error("version not included in axon");
      }

      if ("ip" in axon) {
        assert.ok(axon.ip != undefined);
      } else {
        throw new Error("ip not included in axon");
      }

      if ("port" in axon) {
        assert.ok(axon.port != undefined);
      } else {
        throw new Error("port not included in axon");
      }

      if ("ip_type" in axon) {
        assert.ok(axon.ip_type != undefined);
      } else {
        throw new Error("ip_type not included in axon");
      }

      if ("protocol" in axon) {
        assert.ok(axon.protocol != undefined);
      } else {
        throw new Error("protocol not included in axon");
      }
    }

    const methodList = [
      "getEmission",
      "getVtrust",
      "getValidatorStatus",
      "getLastUpdate",
      "getIsActive",
      "getHotkey",
      "getColdkey",
    ];
    for (const method of methodList) {
      const value = await publicClient.readContract({
        abi: IMetagraphABI,
        address: toViemAddress(IMETAGRAPH_ADDRESS),
        functionName: method,
        args: [subnetId, uid],
      });

      assert.ok(value != undefined);
    }
  });
});
