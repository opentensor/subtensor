import * as assert from "assert";

import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { TypedApi } from "polkadot-api";
import { convertH160ToSS58, convertPublicKeyToSs58, } from "../src/address-utils"
import { ethers } from "ethers"
import { INEURON_ADDRESS, INeuronABI } from "../src/contracts/neuron"
import { generateRandomEthersWallet } from "../src/utils"
import {
    forceSetBalanceToSs58Address, forceSetBalanceToEthAddress, addNewSubnetwork, burnedRegister, setCommitRevealWeightsEnabled,
    setWeightsSetRateLimit
} from "../src/subtensor"

describe("Test neuron precompile contract, set weights function", () => {
    // init eth part
    const wallet = generateRandomEthersWallet();

    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    before(async () => {
        api = await getDevnetApi()

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToEthAddress(api, wallet.address)

        const netuid = await addNewSubnetwork(api, hotkey, coldkey)
        console.log("test on subnet ", netuid)

        await burnedRegister(api, netuid, convertH160ToSS58(wallet.address), coldkey)
        const uid = await api.query.SubtensorModule.Uids.getValue(netuid, convertH160ToSS58(wallet.address))
        assert.notEqual(uid, undefined)
        // disable reveal and enable direct set weights
        await setCommitRevealWeightsEnabled(api, netuid, false)
        await setWeightsSetRateLimit(api, netuid, BigInt(0))
    })

    it("Set weights is ok", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const uid = await api.query.SubtensorModule.Uids.getValue(netuid, convertH160ToSS58(wallet.address))

        const contract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet);
        const dests = [1];
        const weights = [2];
        const version_key = 0;

        const tx = await contract.setWeights(netuid, dests, weights, version_key);

        await tx.wait();
        const weightsOnChain = await api.query.SubtensorModule.Weights.getValue(netuid, uid)

        weightsOnChain.forEach((weight, _) => {
            const uidInWeight = weight[0];
            const value = weight[1];
            assert.equal(uidInWeight, uid)
            assert.ok(value > 0)
        });
    })
});