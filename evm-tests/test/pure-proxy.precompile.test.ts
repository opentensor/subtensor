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
import { Signer } from "@polkadot/api/types";

function getPureProxyAccount(address: string) {

    const prefix = new TextEncoder().encode("pureproxy:")

    const addressH160 = hexToU8a(address)

    const data = new Uint8Array([...prefix, ...addressH160]);

    return keccak256(data)
}

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
        const contract = new ethers.Contract(IPURE_PROXY_ADDRESS, IPureProxyABI, evmWallet)

        const tx = await contract.createPureProxy()
        await tx.wait()
        const proxyAddress = await contract.getPureProxy();

        const expected = getPureProxyAccount(evmWallet.address)
        assert.equal(proxyAddress, expected, "the proxy account not the same as expected")

        const ss58Address = convertPublicKeyToSs58(proxyAddress)

        await forceSetBalanceToSs58Address(api, ss58Address)

        const callCode = await getTransferCallCode(api, alice)
        const tx2 = await contract.pureProxyCall(callCode)
        await tx2.wait()

    })

    it("Call createPureProxy, edge cases", async () => {
        const contract = new ethers.Contract(IPURE_PROXY_ADDRESS, IPureProxyABI, evmWallet2)
        const proxyAddressBeforeCreate = await contract.getPureProxy();
        console.log(proxyAddressBeforeCreate)
        assert.equal(proxyAddressBeforeCreate, "0x0000000000000000000000000000000000000000000000000000000000000000", "proxy should be undefined before set")

        // use papi to set proxy with wrong mapped account
        await setPureProxyAccount(api, evmWallet2.address, convertPublicKeyToSs58(alice.publicKey))
        assert.equal(proxyAddressBeforeCreate, "0x0000000000000000000000000000000000000000000000000000000000000000", "proxy should be zero after wrong signer")

        const callCode = await getTransferCallCode(api, alice)

        // call without proxy
        try {
            const tx = await contract.pureProxyCall(callCode)
            await tx.wait()
        } catch (error) {
            assert.notEqual(error, undefined, "should fail if proxy not set")
        }

        const tx = await contract.createPureProxy()
        await tx.wait()
        const proxyAddress = await contract.getPureProxy();

        const expected = getPureProxyAccount(evmWallet2.address)
        assert.equal(proxyAddress, expected, "the proxy account not the same as expected")

        // set the proxy again
        try {
            const tx = await contract.createPureProxy()
            await tx.wait()
        } catch (error) {
            assert.notEqual(error, undefined, "should fail if set proxy again")
        }

        // send extrinsic without token
        try {
            const tx = await contract.pureProxyCall(callCode)
            await tx.wait()
        } catch (error) {
            assert.notEqual(error, undefined, "should fail if proxy without balance")
        }

        // set balance for proxy account
        const ss58Address = convertPublicKeyToSs58(proxyAddress)
        await forceSetBalanceToSs58Address(api, ss58Address)

        // try proxy call finally
        const tx2 = await contract.pureProxyCall(callCode)
        await tx2.wait()
    })
});
