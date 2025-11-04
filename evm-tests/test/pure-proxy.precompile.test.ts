import * as assert from "assert";

import { getAliceSigner, getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { generateRandomEthersWallet, generateRandomEthWallet } from "../src/utils";
import { devnet, MultiAddress } from "@polkadot-api/descriptors"
import { hexToU8a } from "@polkadot/util";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertH160ToSS58, convertPublicKeyToSs58 } from "../src/address-utils"
import { IProxyABI, IPROXY_ADDRESS } from "../src/contracts/proxy"
import { keccak256, ethers } from 'ethers';
import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address } from "../src/subtensor";
import { Signer } from "@polkadot/api/types";
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
    const evmWallet2 = generateRandomEthersWallet();
    const receiver = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    let alice: PolkadotSigner;

    before(async () => {
        api = await getDevnetApi()
        alice = await getAliceSigner();

        await forceSetBalanceToEthAddress(api, evmWallet.address)
        await forceSetBalanceToEthAddress(api, evmWallet2.address)

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

    it("Call createPureProxy, add kill one", async () => {
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet)
        const type = 0;
        const delay = 0;
        const index = 0;
        const extIndex = 1;

        const proxies = await getProxies(api, convertH160ToSS58(evmWallet.address))
        const length = proxies.length
        const addTx = await contract.createPureProxy(type, delay, index)
        const response = await addTx.wait()
        const createBlockNumber = response.blockNumber

        const currentLength = (await getProxies(api, convertH160ToSS58(evmWallet.address))).length
        assert.equal(currentLength, length + 1, "proxy should be set")

        try {
            const tx = await contract.killPureProxy(decodeAddress(proxies[proxies.length - 1]), type, index,
                createBlockNumber, extIndex)
            await tx.wait()
        } catch (error) {
            console.log("error", error)
        }

        const proxiesAfterRemove = await getProxies(api, convertH160ToSS58(evmWallet.address))
        assert.equal(proxiesAfterRemove.length, 0, "proxies should be removed")
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
});
