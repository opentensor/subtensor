import * as assert from "assert";

import { getAliceSigner, getDevnetApi } from "../src/substrate"
import { generateRandomEthersWallet, generateRandomEthWallet } from "../src/utils";
import { devnet, MultiAddress } from "@polkadot-api/descriptors"
import { hexToU8a } from "@polkadot/util";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58 } from "../src/address-utils"
import { IPureProxyABI, IPURE_PROXY_ADDRESS } from "../src/contracts/pureProxy"
import { keccak256, ethers } from 'ethers';
import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, setPureProxyAccount } from "../src/subtensor";

function getPureProxyAccount(address: string) {

    const prefix = new TextEncoder().encode("pureproxy:")

    const addressH160 = hexToU8a(address)

    const data = new Uint8Array([...prefix, ...addressH160]);

    return keccak256(data)
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
        const contract = new ethers.Contract(IPURE_PROXY_ADDRESS, IPureProxyABI, evmWallet)
        const tx = await contract.createPureProxy()
        await tx.wait()

        const proxyAddress = await contract.getPureProxy();

        const expected = getPureProxyAccount(evmWallet.address)
        assert.equal(proxyAddress, expected, "the proxy account not the same as expected")

        const ss58Address = convertPublicKeyToSs58(proxyAddress)

        await forceSetBalanceToSs58Address(api, ss58Address)

        const transferAmount = BigInt(1000000000);

        const unsignedTx = api.tx.Balances.transfer_keep_alive({
            dest: MultiAddress.Id(convertPublicKeyToSs58(alice.publicKey)),
            value: transferAmount,
        });
        const encodedCallDataBytes = await unsignedTx.getEncodedData();

        // encoded call should be 0x050300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d02286bee
        // const transferCall = encodedCallDataBytes

        const data = encodedCallDataBytes.asBytes()

        const tx2 = await contract.pureProxyCall([...data])
        await tx2.wait()

    })

    it("Call createPureProxy, edge cases", async () => {
        const contract = new ethers.Contract(IPURE_PROXY_ADDRESS, IPureProxyABI, evmWallet)
        const proxyAddressBeforeCreate = await contract.getPureProxy();
        assert.equal(proxyAddressBeforeCreate, undefined, "proxy should be undefined before set")

        // use papi to set proxy with wrong mapped account

        try {
            await setPureProxyAccount(api, evmWallet.address, convertPublicKeyToSs58(alice.publicKey))
        } catch (error) {

            if (error instanceof Error) {
                assert.notEqual(error, undefined, "should fail if set proxy again")
            }
        }

        const tx = await contract.createPureProxy()
        await tx.wait()

        const proxyAddress = await contract.getPureProxy();

        const expected = getPureProxyAccount(evmWallet.address)
        assert.equal(proxyAddress, expected, "the proxy account not the same as expected")

        // set the proxy again
        try {
            const tx2 = await contract.createPureProxy()
            await tx2.wait()
        } catch (error) {

            if (error instanceof Error) {
                assert.notEqual(error, undefined, "should fail if set proxy again")
            }
        }

        // call transfer without token
        // call transfer without proxy

        const ss58Address = convertPublicKeyToSs58(proxyAddress)

        await forceSetBalanceToSs58Address(api, ss58Address)

        const transferAmount = BigInt(1000000000);

        const unsignedTx = api.tx.Balances.transfer_keep_alive({
            dest: MultiAddress.Id(convertPublicKeyToSs58(alice.publicKey)),
            value: transferAmount,
        });
        const encodedCallDataBytes = await unsignedTx.getEncodedData();

        // encoded call should be 0x050300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d02286bee
        // const transferCall = encodedCallDataBytes

        const data = encodedCallDataBytes.asBytes()

        const tx3 = await contract.pureProxyCall([...data])
        await tx3.wait()

    })
});
