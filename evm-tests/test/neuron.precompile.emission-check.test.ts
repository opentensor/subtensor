import * as assert from "assert";

import { getAliceSigner, getClient, getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { getPublicClient, } from "../src/utils";
import { ETH_LOCAL_URL, SUB_LOCAL_URL, } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, } from "../src/address-utils"
import { ethers } from "ethers"
import { INEURON_ADDRESS, INeuronABI } from "../src/contracts/neuron"
import { generateRandomEthersWallet } from "../src/utils"
import { forceSetBalanceToSs58Address, forceSetBalanceToEthAddress, addNewSubnetwork } from "../src/subtensor"

describe("Test the EVM chain ID", () => {
    // init eth part
    const wallet = generateRandomEthersWallet();

    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const hotkey2 = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>

    // sudo account alice as signer
    let alice: PolkadotSigner;

    before(async () => {
        // init variables got from await and async
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        const subClient = await getClient(SUB_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner();
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey2.publicKey))

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToEthAddress(api, wallet.address)

        const netuid = await addNewSubnetwork(api, hotkey2, coldkey)
        console.log("test on subnet ", netuid)
    })

    it("Burned register and check emission", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const uid = await api.query.SubtensorModule.SubnetworkN.getValue(netuid)
        const contract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet);

        const tx = await contract.burnedRegister(
            netuid,
            hotkey.publicKey
        );
        await tx.wait();

        const uidAfterNew = await api.query.SubtensorModule.SubnetworkN.getValue(netuid)
        assert.equal(uid + 1, uidAfterNew)

        const key = await api.query.SubtensorModule.Keys.getValue(netuid, uid)
        assert.equal(key, convertPublicKeyToSs58(hotkey.publicKey))

        let i = 0;
        while (i < 10) {
            const emission = await api.query.SubtensorModule.PendingEmission.getValue(netuid)

            console.log("emission is ", emission);
            await new Promise((resolve) => setTimeout(resolve, 2000));
            i += 1;
        }
    })
});