import * as assert from "assert";

import { getAliceSigner, getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { generateRandomEthersWallet } from "../src/utils";
import { devnet, MultiAddress } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertH160ToPublicKey, convertH160ToSS58, convertPublicKeyToSs58 } from "../src/address-utils"
import { IProxyABI, IPROXY_ADDRESS } from "../src/contracts/proxy"
import { ethers } from 'ethers';
import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address } from "../src/subtensor";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";

import { decodeAddress } from "@polkadot/util-crypto";

async function getTransferCallCode(api: TypedApi<typeof devnet>, receiver: KeyPair, transferAmount: number) {

    const unsignedTx = api.tx.Balances.transfer_keep_alive({
        dest: MultiAddress.Id(convertPublicKeyToSs58(receiver.publicKey)),
        value: BigInt(1000000000),
    });
    const encodedCallDataBytes = await unsignedTx.getEncodedData();

    // encoded call should be 0x050300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d02286bee
    // const transferCall = encodedCallDataBytes

    const data = encodedCallDataBytes.asBytes()

    return [...data]
}

async function getProxies(api: TypedApi<typeof devnet>, address: string) {
    const entries = await api.query.Proxy.Proxies.getEntries()
    const result = []
    for (const entry of entries) {
        const proxyAddress = entry.keyArgs[0]
        const values = entry.value
        const proxies = values[0]
        for (const proxy of proxies) {
            if (proxy.delegate === address) {
                result.push(proxyAddress)
            }
        }
    }
    return result
}

describe("Test pure proxy precompile", () => {
    const evmWallet = generateRandomEthersWallet();
    // only used for edge case and normal proxy
    const evmWallet2 = generateRandomEthersWallet();
    const evmWallet3 = generateRandomEthersWallet();
    const evmWallet4 = generateRandomEthersWallet();
    const receiver = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    let alice: PolkadotSigner;

    before(async () => {
        api = await getDevnetApi()
        alice = await getAliceSigner();

        await forceSetBalanceToEthAddress(api, evmWallet.address)
        await forceSetBalanceToEthAddress(api, evmWallet2.address)
        await forceSetBalanceToEthAddress(api, evmWallet3.address)
        await forceSetBalanceToEthAddress(api, evmWallet4.address)
    })

    it("Call createPureProxy, then use proxy to call transfer", async () => {
        const proxies = await getProxies(api, convertH160ToSS58(evmWallet.address))
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet)
        console.log("evmWallet", evmWallet.address)

        const type = 0;
        const delay = 0;
        const index = 0;
        const tx = await contract.createPureProxy(type, delay, index)
        const response = await tx.wait()
        console.log("response", response.blockNumber)

        const proxiesAfterAdd = await getProxies(api, convertH160ToSS58(evmWallet.address))

        const length = proxiesAfterAdd.length
        assert.equal(length, proxies.length + 1, "proxy should be set")
        const proxy = proxiesAfterAdd[proxiesAfterAdd.length - 1]

        await forceSetBalanceToSs58Address(api, proxy)
        const balance = (await api.query.System.Account.getValue(convertPublicKeyToSs58(receiver.publicKey))).data.free

        const amount = 1000000000;

        const callCode = await getTransferCallCode(api, receiver, amount)
        const tx2 = await contract.proxyCall(decodeAddress(proxy), [type], callCode)
        await tx2.wait()

        const balanceAfter = (await api.query.System.Account.getValue(convertPublicKeyToSs58(receiver.publicKey))).data.free
        assert.equal(balanceAfter, balance + BigInt(amount), "balance should be increased")
    })

    it("Call createPureProxy, add multiple proxies", async () => {
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet)
        const type = 0;
        const delay = 0;
        const index = 0;
        const proxies = await getProxies(api, convertH160ToSS58(evmWallet.address))
        const length = proxies.length
        for (let i = 0; i < 5; i++) {
            const tx = await contract.createPureProxy(type, delay, index)
            await tx.wait()

            await new Promise(resolve => setTimeout(resolve, 500));
            const currentProxies = await getProxies(api, convertH160ToSS58(evmWallet.address))
            assert.equal(currentProxies.length, length + i + 1, "proxy should be set")
        }
    })

    it("Call createPureProxy, edge cases, call via wrong proxy", async () => {
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet2)
        const amount = 1000000000;
        const callCode = await getTransferCallCode(api, receiver, amount)
        const type = 0;

        // call with wrong proxy
        try {
            const tx = await contract.proxyCall(receiver, [type], callCode)
            await tx.wait()
        } catch (error) {
            assert.notEqual(error, undefined, "should fail if proxy not set")
        }
    })

    it("Call createProxy, then use proxy to call transfer", async () => {
        const proxies = await api.query.Proxy.Proxies.getValue(convertH160ToSS58(evmWallet2.address))
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet2)

        const proxiesFromContract = await contract.getProxies(convertH160ToPublicKey(evmWallet2.address))
        assert.equal(proxiesFromContract.length, proxies[0].length, "proxies length should be equal")

        const type = 0;
        const delay = 0;

        const tx = await contract.addProxy(convertH160ToPublicKey(evmWallet3.address), type, delay)
        await tx.wait()

        const proxiesAfterAdd = await await api.query.Proxy.Proxies.getValue(convertH160ToSS58(evmWallet2.address))
        const proxiesList = proxiesAfterAdd[0].map(proxy => proxy.delegate)

        const proxiesFromContractAfterAdd = await contract.getProxies(convertH160ToPublicKey(evmWallet2.address))

        assert.equal(proxiesFromContractAfterAdd.length, proxiesList.length, "proxy length should be equal")

        for (let index = 0; index < proxiesFromContractAfterAdd.length; index++) {
            const proxyInfo = proxiesFromContractAfterAdd[index]
            let proxySs58 = convertPublicKeyToSs58(proxyInfo[0])
            assert.ok(proxiesList.includes(proxySs58), "proxy should be set")
            if (index === proxiesFromContractAfterAdd.length - 1) {
                assert.equal(Number(proxyInfo[1]), type, "proxy_type should match")
                assert.equal(Number(proxyInfo[2]), delay, "delay should match")
            }
        }

        assert.equal(proxiesList.length, proxies[0].length + 1, "proxy should be set")
        const proxy = proxiesList[proxiesList.length - 1]

        assert.equal(proxy, convertH160ToSS58(evmWallet3.address), "proxy should be set")
        const balance = (await api.query.System.Account.getValue(convertPublicKeyToSs58(receiver.publicKey))).data.free
        const amount = 1000000000;

        const contract2 = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet3)
        const callCode = await getTransferCallCode(api, receiver, amount)
        const tx2 = await contract2.proxyCall(convertH160ToPublicKey(evmWallet2.address), [type], callCode)
        await tx2.wait()

        const balanceAfter = (await api.query.System.Account.getValue(convertPublicKeyToSs58(receiver.publicKey))).data.free
        assert.equal(balanceAfter, balance + BigInt(amount), "balance should be increased")
    })

    it("Call addProxy many times, then check getProxies is correct", async () => {
        const proxies = await api.query.Proxy.Proxies.getValue(convertH160ToSS58(evmWallet4.address))
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet4)
        assert.equal(proxies[0].length, 0, "proxies length should be 0")

        const proxiesFromContract = await contract.getProxies(convertH160ToPublicKey(evmWallet4.address))
        assert.equal(proxiesFromContract.length, proxies[0].length, "proxies length should be equal")

        const type = 1;
        const delay = 2;

        for (let i = 0; i < 5; i++) {
            const evmWallet = generateRandomEthersWallet()
            const tx = await contract.addProxy(convertH160ToPublicKey(evmWallet.address), type, delay)
            await tx.wait()
        }

        const proxiesAfterAdd = await await api.query.Proxy.Proxies.getValue(convertH160ToSS58(evmWallet4.address))
        const proxiesList = proxiesAfterAdd[0].map(proxy => proxy.delegate)

        const proxiesFromContractAfterAdd = await contract.getProxies(convertH160ToPublicKey(evmWallet4.address))

        assert.equal(proxiesFromContractAfterAdd.length, proxiesList.length, "proxy length should be equal")

        for (let index = 0; index < proxiesFromContractAfterAdd.length; index++) {
            const proxyInfo = proxiesFromContractAfterAdd[index]
            let proxySs58 = convertPublicKeyToSs58(proxyInfo[0])
            assert.ok(proxiesList.includes(proxySs58), "proxy should be set")
            assert.equal(Number(proxyInfo[1]), type, "proxy_type should match")
            assert.equal(Number(proxyInfo[2]), delay, "delay should match")
        }
    })
});
