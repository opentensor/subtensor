import * as assert from "assert";

import { getAliceSigner, getDevnetApi } from "../src/substrate"
import { generateRandomEthersWallet, generateRandomEthWallet } from "../src/utils";
import { devnet, MultiAddress } from "@polkadot-api/descriptors"
import { hexToU8a } from "@polkadot/util";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, ss58ToEthAddress } from "../src/address-utils"
import { IPureProxyABI, IPURE_PROXY_ADDRESS } from "../src/contracts/pureProxy"
import { keccak256, ethers } from 'ethers';
import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address } from "../src/subtensor";
import { Signer } from "@polkadot/api/types";
import { decodeAddress } from "@polkadot/util-crypto";

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

        const entries = await api.query.Proxy.Proxies.getEntries();
        const proxiesArray: string[] = [];
        let index = 0;
        while (index < entries.length) {
            const proxy = entries[index];
            proxiesArray.push(proxy.keyArgs[0].toString())
            index++;
        }

        const tx = await contract.createPureProxy()
        await tx.wait()

        const entriesAfterCall = await api.query.Proxy.Proxies.getEntries();
        const proxiesArrayAfterCall = [];

        index = 0;
        while (index < entriesAfterCall.length) {
            const proxy = entriesAfterCall[index];
            proxiesArrayAfterCall.push(proxy.keyArgs[0].toString())
            index++;
        }

        const newProxy = proxiesArrayAfterCall.filter(proxy => !proxiesArray.includes(proxy))
        // at least one proxy should be created
        assert.equal(newProxy.length, 1, "newProxy should be 1")

        await forceSetBalanceToSs58Address(api, newProxy[0])

        const publicKey = decodeAddress(newProxy[0])
        const callCode = await getTransferCallCode(api, alice)
        const tx2 = await contract.pureProxyCall(publicKey, callCode)
        await tx2.wait()
    })
});
