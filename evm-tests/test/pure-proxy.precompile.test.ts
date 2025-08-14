import * as assert from "assert";

import { getAliceSigner, getDevnetApi } from "../src/substrate"
import { generateRandomEthersWallet, generateRandomEthWallet } from "../src/utils";
import { devnet, MultiAddress } from "@polkadot-api/descriptors"
import { hexToU8a } from "@polkadot/util";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58 } from "../src/address-utils"
import { IProxyABI, IPROXY_ADDRESS } from "../src/contracts/proxy"
import { keccak256, ethers } from 'ethers';
import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address } from "../src/subtensor";
import { Signer } from "@polkadot/api/types";

async function getTransferCallCode(api: TypedApi<typeof devnet>, signer: PolkadotSigner) {
    const transferAmount = BigInt(1000000000);

    const unsignedTx = api.tx.Balances.transfer_keep_alive({
        dest: MultiAddress.Id(convertPublicKeyToSs58(signer.publicKey)),
        value: transferAmount,
    });
    const encodedCallDataBytes = await unsignedTx.getEncodedData();

    // encoded call should be 0x050300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d02286bee
    // const transferCall = encodedCallDataBytes

    const data = encodedCallDataBytes.asBytes()

    return [...data]
}

describe("Test pure proxy precompile", () => {
    const evmWallet = generateRandomEthersWallet();
    const evmWallet2 = generateRandomEthersWallet();

    let api: TypedApi<typeof devnet>

    let alice: PolkadotSigner;

    before(async () => {
        api = await getDevnetApi()
        alice = await getAliceSigner();

        await forceSetBalanceToEthAddress(api, evmWallet.address)
        await forceSetBalanceToEthAddress(api, evmWallet2.address)

    })

    it("Call createPureProxy, then use proxy to call transfer", async () => {
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet)
        console.log("evmWallet", evmWallet.address)

        const tx = await contract.createPureProxy()
        const proxyAddress = await tx.wait()
        assert.equal(proxyAddress.length, 1, "proxy should be set")

        const ss58Address = convertPublicKeyToSs58(proxyAddress[0])

        await forceSetBalanceToSs58Address(api, ss58Address)

        const callCode = await getTransferCallCode(api, alice)
        const tx2 = await contract.proxyCall(proxyAddress[0], callCode)
        await tx2.wait()
    })

    it("Call createPureProxy, add multiple proxies", async () => {
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet)

        let proxies = []
        for (let i = 0; i < 10; i++) {
            const tx = await contract.createPureProxy()
            const proxyAddressAfterCreate = await tx.wait()
            assert.equal(proxyAddressAfterCreate.length,  i + 1, "proxy should be set")
            proxies.push(proxyAddressAfterCreate[0])
        }

        const tx = await contract.killPureProxy(proxies[proxies.length - 1])
        await tx.wait()
    })

    it("Call createPureProxy, edge cases", async () => {
        const contract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, evmWallet2)

        const callCode = await getTransferCallCode(api, alice)

        // call without proxy
        try {
            const tx = await contract.proxyCall(callCode)
            await tx.wait()
        } catch (error) {
            assert.notEqual(error, undefined, "should fail if proxy not set")
        }

        const tx = await contract.createPureProxy()
        const proxyAddress = await tx.wait()

        // set the proxy again
        try {
            const tx = await contract.createPureProxy()
            await tx.wait()
        } catch (error) {
            assert.notEqual(error, undefined, "should fail if set proxy again")
        }

        // send extrinsic without token
        try {
            const tx = await contract.proxyCall(callCode)
            await tx.wait()
        } catch (error) {
            assert.notEqual(error, undefined, "should fail if proxy without balance")
        }

        // set balance for proxy account
        const ss58Address = convertPublicKeyToSs58(proxyAddress[0])
        await forceSetBalanceToSs58Address(api, ss58Address)

        // try proxy call finally
        const tx2 = await contract.proxyCall(proxyAddress[0], callCode)
        await tx2.wait()
    })
});
