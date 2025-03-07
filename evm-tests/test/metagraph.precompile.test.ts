import * as assert from "assert";

import { getAliceSigner, getClient, getDevnetApi, waitForTransactionCompletion, convertPublicKeyToMultiAddress, getRandomSubstrateKeypair, getSignerFromKeypair } from "../src/substrate"
import { getPublicClient, } from "../src/utils";
import { ETH_LOCAL_URL, SUB_LOCAL_URL, } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils"
import { IMetagraphABI, IMETAGRAPH_ADDRESS } from "../src/contracts/metagraph"

describe("Test the EVM chain ID", () => {
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>

    // sudo account alice as signer
    let alice: PolkadotSigner;

    // init other variable
    let subnetId = 0;

    before(async () => {
        // init variables got from await and async
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        const subClient = await getClient(SUB_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner();

        {
            const multiAddress = convertPublicKeyToMultiAddress(hotkey.publicKey)
            const internalCall = api.tx.Balances.force_set_balance({ who: multiAddress, new_free: BigInt(1e12) })
            const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

            await waitForTransactionCompletion(api, tx, alice)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        }

        {
            const multiAddress = convertPublicKeyToMultiAddress(coldkey.publicKey)
            const internalCall = api.tx.Balances.force_set_balance({ who: multiAddress, new_free: BigInt(1e12) })
            const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

            await waitForTransactionCompletion(api, tx, alice)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        }

        const signer = getSignerFromKeypair(coldkey)
        const registerNetworkTx = api.tx.SubtensorModule.register_network({ hotkey: convertPublicKeyToSs58(hotkey.publicKey) })
        await waitForTransactionCompletion(api, registerNetworkTx, signer)
            .then(() => { })
            .catch((error) => { console.log(`transaction error ${error}`) });

        let totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue()
        assert.ok(totalNetworks > 1)
        subnetId = totalNetworks - 1

        let uid_count =
            await api.query.SubtensorModule.SubnetworkN.getValue(subnetId)
        if (uid_count === 0) {
            const tx = api.tx.SubtensorModule.burned_register({ hotkey: convertPublicKeyToSs58(hotkey.publicKey), netuid: subnetId })
            await waitForTransactionCompletion(api, tx, signer)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        }
    })

    it("Metagraph data access via precompile contract is ok", async () => {
        const uid = 0
        const uid_count = await publicClient.readContract({
            abi: IMetagraphABI,
            address: toViemAddress(IMETAGRAPH_ADDRESS),
            functionName: "getUidCount",
            args: [subnetId]
        })
        // back to original value for other tests. and we can run it repeatedly
        assert.ok(uid_count != undefined);

        // const axon = api.query.SubtensorModule.Axons.getValue()

        const axon = await publicClient.readContract({
            abi: IMetagraphABI,
            address: toViemAddress(IMETAGRAPH_ADDRESS),
            functionName: "getAxon",
            args: [subnetId, uid]
        })

        assert.ok(axon != undefined);
        if (axon instanceof Object) {
            assert.ok(axon != undefined);
            if ("block" in axon) {
                assert.ok(axon.block != undefined);
            } else {
                throw new Error("block not included in axon")
            }

            if ("version" in axon) {
                assert.ok(axon.version != undefined);
            } else {
                throw new Error("version not included in axon")
            }

            if ("ip" in axon) {
                assert.ok(axon.ip != undefined);
            } else {
                throw new Error("ip not included in axon")
            }

            if ("port" in axon) {
                assert.ok(axon.port != undefined);
            } else {
                throw new Error("port not included in axon")
            }

            if ("ip_type" in axon) {
                assert.ok(axon.ip_type != undefined);
            } else {
                throw new Error("ip_type not included in axon")
            }

            if ("protocol" in axon) {
                assert.ok(axon.protocol != undefined);
            } else {
                throw new Error("protocol not included in axon")
            }
        }

        const methodList = ["getEmission", "getVtrust", "getValidatorStatus", "getLastUpdate", "getIsActive",
            "getHotkey", "getColdkey"
        ]
        for (const method of methodList) {
            const value = await publicClient.readContract({
                abi: IMetagraphABI,
                address: toViemAddress(IMETAGRAPH_ADDRESS),
                functionName: method,
                args: [subnetId, uid]
            })

            assert.ok(value != undefined);
        }
    });
});