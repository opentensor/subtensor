import * as assert from "assert";
import { getAliceSigner, getDevnetApi } from "../src/substrate"
import {  generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { IDISPATCH_ADDRESS, ISTORAGE_QUERY_ADDRESS, ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { hexToNumber, PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58 } from "../src/address-utils"
import { forceSetBalanceToEthAddress, setMaxChildkeyTake } from "../src/subtensor";

describe("Test the dispatch precompile", () => {
    let publicClient: PublicClient;
    const wallet1 = generateRandomEthersWallet();
    let api: TypedApi<typeof devnet>
    let alice: PolkadotSigner;

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner()
        await forceSetBalanceToEthAddress(api, wallet1.address)
    })

    it("Dispatch transfer call via precompile contract works correctly", async () => {
        // 0x050300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d02286bee
        // call for transfer 1 token to alice
        const transferAmount = BigInt(1000000000);
        const transferCall = "0x050300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d02286bee"
        const aliceBalance =  (await api.query.System.Account.getValue( convertPublicKeyToSs58(alice.publicKey))).data.free
        const txResponse = await wallet1.sendTransaction({
            to: IDISPATCH_ADDRESS,
            data: transferCall,
        })
        await txResponse.wait()

        const aliceBalanceAfterTransfer =  (await api.query.System.Account.getValue( convertPublicKeyToSs58(alice.publicKey))).data.free

        assert.equal(aliceBalance + transferAmount, aliceBalanceAfterTransfer)
    })

    

    it("Storage query call via precompile contract works correctly", async () => {
        let maxChildkeyTake = 257;
        await setMaxChildkeyTake(api, maxChildkeyTake)
        // 0x658faa385070e074c85bf6b568cf0555f14f14d903e9994045ff1902b3e513dc
        // key for min child key take

        const storageQuery = "0x658faa385070e074c85bf6b568cf0555dba018859cab7e989f77669457b394be"

        api.query.SubtensorModule.MaxChildkeyTake.getValue();
        const rawCallResponse = await publicClient.call({
            to: ISTORAGE_QUERY_ADDRESS,
            data: storageQuery,
        })
        const rawResultData = rawCallResponse.data;
        if (rawResultData === undefined) {
            throw new Error("rawResultData is undefined");
        }
        let value = hexToNumber(rawResultData);
        assert.equal(value, maxChildkeyTake, "value should be 257")
    })
});
