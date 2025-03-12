import * as assert from "assert";
import { getAliceSigner, getClient, getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { SUB_LOCAL_URL, } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58 } from "../src/address-utils"
import { ethers } from "ethers"
import { INEURON_ADDRESS, INeuronABI } from "../src/contracts/neuron"
import { generateRandomEthersWallet } from "../src/utils"
import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, addNewSubnetwork, burnedRegister } from "../src/subtensor"

describe("Test neuron precompile Serve Axon Prometheus", () => {
    // init eth part
    // const wallet1 = generateRandomEthersWallet();
    // const wallet2 = generateRandomEthersWallet();
    // const wallet3 = generateRandomEthersWallet();

    // init substrate part

    // const coldkey = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        // init variables got from await and async
        const subClient = await getClient(SUB_LOCAL_URL)
        api = await getDevnetApi()
        // alice = await getAliceSigner();

        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        // await forceSetBalanceToEthAddress(api, wallet1.address)
        // await forceSetBalanceToEthAddress(api, wallet2.address)
        // await forceSetBalanceToEthAddress(api, wallet3.address)


        let index = 0;
        while (index < 30) {
            const hotkey = getRandomSubstrateKeypair();
            const coldkey = getRandomSubstrateKeypair();
            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
            let netuid = await addNewSubnetwork(api, hotkey, coldkey)
        }


    })

    it("Serve Axon", async () => {

    });
});