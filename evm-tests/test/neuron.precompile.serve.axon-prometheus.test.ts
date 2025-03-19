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
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    const wallet3 = generateRandomEthersWallet();

    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        // init variables got from await and async
        const subClient = await getClient(SUB_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner();

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(alice.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToEthAddress(api, wallet1.address)
        await forceSetBalanceToEthAddress(api, wallet2.address)
        await forceSetBalanceToEthAddress(api, wallet3.address)
        let netuid = await addNewSubnetwork(api, hotkey, coldkey)

        console.log("test the case on subnet ", netuid)

        await burnedRegister(api, netuid, convertH160ToSS58(wallet1.address), coldkey)
        await burnedRegister(api, netuid, convertH160ToSS58(wallet2.address), coldkey)
        await burnedRegister(api, netuid, convertH160ToSS58(wallet3.address), coldkey)
    })

    it("Serve Axon", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const version = 0;
        const ip = 1;
        const port = 2;
        const ipType = 4;
        const protocol = 0;
        const placeholder1 = 8;
        const placeholder2 = 9;

        const contract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet1);

        const tx = await contract.serveAxon(
            netuid,
            version,
            ip,
            port,
            ipType,
            protocol,
            placeholder1,
            placeholder2
        );
        await tx.wait();

        const axon = await api.query.SubtensorModule.Axons.getValue(
            netuid,
            convertH160ToSS58(wallet1.address)
        )
        assert.notEqual(axon?.block, undefined)
        assert.equal(axon?.version, version)
        assert.equal(axon?.ip, ip)
        assert.equal(axon?.port, port)
        assert.equal(axon?.ip_type, ipType)
        assert.equal(axon?.protocol, protocol)
        assert.equal(axon?.placeholder1, placeholder1)
        assert.equal(axon?.placeholder2, placeholder2)
    });

    it("Serve Axon TLS", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const version = 0;
        const ip = 1;
        const port = 2;
        const ipType = 4;
        const protocol = 0;
        const placeholder1 = 8;
        const placeholder2 = 9;
        // certificate length is 65
        const certificate = new Uint8Array([
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,
            39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
            57, 58, 59, 60, 61, 62, 63, 64, 65,
        ]);

        const contract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet2);

        const tx = await contract.serveAxonTls(
            netuid,
            version,
            ip,
            port,
            ipType,
            protocol,
            placeholder1,
            placeholder2,
            certificate
        );
        await tx.wait();

        const axon = await api.query.SubtensorModule.Axons.getValue(
            netuid,
            convertH160ToSS58(wallet2.address))

        assert.notEqual(axon?.block, undefined)
        assert.equal(axon?.version, version)
        assert.equal(axon?.ip, ip)
        assert.equal(axon?.port, port)
        assert.equal(axon?.ip_type, ipType)
        assert.equal(axon?.protocol, protocol)
        assert.equal(axon?.placeholder1, placeholder1)
        assert.equal(axon?.placeholder2, placeholder2)
    });

    it("Serve Prometheus", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const version = 0;
        const ip = 1;
        const port = 2;
        const ipType = 4;

        const contract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet3);

        const tx = await contract.servePrometheus(
            netuid,
            version,
            ip,
            port,
            ipType
        );
        await tx.wait();

        const prometheus = (
            await api.query.SubtensorModule.Prometheus.getValue(
                netuid,
                convertH160ToSS58(wallet3.address)
            )
        )

        assert.notEqual(prometheus?.block, undefined)
        assert.equal(prometheus?.version, version)
        assert.equal(prometheus?.ip, ip)
        assert.equal(prometheus?.port, port)
        assert.equal(prometheus?.ip_type, ipType)
    });
});