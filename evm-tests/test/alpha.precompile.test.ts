import * as assert from "assert";

import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils"
import { IAlphaABI, IALPHA_ADDRESS } from "../src/contracts/alpha"
import { forceSetBalanceToSs58Address, addNewSubnetwork, startCall } from "../src/subtensor";
describe("Test Alpha Precompile", () => {
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>;

    // init other variable
    let subnetId = 0;

    before(async () => {
        // init variables got from await and async
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))

        let netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)

    })

    describe("Alpha Price Functions", () => {
        it("getAlphaPrice returns valid price for subnet", async () => {
            const alphaPrice = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaPrice",
                args: [subnetId]
            })

            assert.ok(alphaPrice !== undefined, "Alpha price should be defined");
            assert.ok(typeof alphaPrice === 'bigint', "Alpha price should be a bigint");
            assert.ok(alphaPrice >= BigInt(0), "Alpha price should be non-negative");
        });


    });
});
