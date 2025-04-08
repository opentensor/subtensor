import * as assert from "assert";
import { getAliceSigner, getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58 } from "../src/address-utils"
import { Vec, Tuple, VecFixed, u16, u8, u64 } from "@polkadot/types-codec";
import { TypeRegistry } from "@polkadot/types";
import { ethers } from "ethers"
import { INEURON_ADDRESS, INeuronABI } from "../src/contracts/neuron"
import { generateRandomEthersWallet } from "../src/utils"
import { convertH160ToPublicKey } from "../src/address-utils"
import { blake2AsU8a } from "@polkadot/util-crypto"
import {
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, addNewSubnetwork, setCommitRevealWeightsEnabled, setWeightsSetRateLimit, burnedRegister,
    setTempo, setCommitRevealWeightsInterval
} from "../src/subtensor"

// hardcode some values for reveal hash
const uids = [1];
const values = [5];
const salt = [9];
const version_key = 0;

function getCommitHash(netuid: number, address: string) {
    const registry = new TypeRegistry();
    let publicKey = convertH160ToPublicKey(address);

    const tupleData = new Tuple(
        registry,
        [
            VecFixed.with(u8, 32),
            u16,
            Vec.with(u16),
            Vec.with(u16),
            Vec.with(u16),
            u64,
        ],
        [publicKey, netuid, uids, values, salt, version_key]
    );

    const hash = blake2AsU8a(tupleData.toU8a());
    return hash;
}

describe("Test neuron precompile reveal weights", () => {
    // init eth part
    const wallet = generateRandomEthersWallet();

    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        // init variables got from await and async
        api = await getDevnetApi()
        alice = await getAliceSigner();

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(alice.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToEthAddress(api, wallet.address)
        let netuid = await addNewSubnetwork(api, hotkey, coldkey)

        console.log("test the case on subnet ", netuid)

        // enable commit reveal feature
        await setCommitRevealWeightsEnabled(api, netuid, true)
        // set it as 0, we can set the weight anytime
        await setWeightsSetRateLimit(api, netuid, BigInt(0))

        const ss58Address = convertH160ToSS58(wallet.address)
        await burnedRegister(api, netuid, ss58Address, coldkey)

        const uid = await api.query.SubtensorModule.Uids.getValue(
            netuid,
            ss58Address
        )
        // eth wallet account should be the first neuron in the subnet
        assert.equal(uid, uids[0])
    })

    it("EVM neuron commit weights via call precompile", async () => {
        let totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue()
        const subnetId = totalNetworks - 1
        const commitHash = getCommitHash(subnetId, wallet.address)
        const contract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet);
        const tx = await contract.commitWeights(subnetId, commitHash)
        await tx.wait()

        const ss58Address = convertH160ToSS58(wallet.address)

        const weightsCommit = await api.query.SubtensorModule.WeightCommits.getValue(subnetId, ss58Address)
        if (weightsCommit === undefined) {
            throw new Error("submit weights failed")
        }
        assert.ok(weightsCommit.length > 0)
    })

    it("EVM neuron reveal weights via call precompile", async () => {
        let totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue()
        const netuid = totalNetworks - 1
        const contract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet);
        // set tempo or epoch large, then enough time to reveal weight
        await setTempo(api, netuid, 60000)
        // set interval epoch as 0, we can reveal at the same epoch
        await setCommitRevealWeightsInterval(api, netuid, BigInt(0))

        const tx = await contract.revealWeights(
            netuid,
            uids,
            values,
            salt,
            version_key
        );
        await tx.wait()
        const ss58Address = convertH160ToSS58(wallet.address)

        // check the weight commit is removed after reveal successfully
        const weightsCommit = await api.query.SubtensorModule.WeightCommits.getValue(netuid, ss58Address)
        assert.equal(weightsCommit, undefined)

        // check the weight is set after reveal with correct uid
        const neuron_uid = await api.query.SubtensorModule.Uids.getValue(
            netuid,
            ss58Address
        )

        const weights = await api.query.SubtensorModule.Weights.getValue(netuid, neuron_uid)

        if (weights === undefined) {
            throw new Error("weights not available onchain")
        }
        for (const weight of weights) {
            assert.equal(weight[0], neuron_uid)
            assert.ok(weight[1] !== undefined)
        }
    })
});