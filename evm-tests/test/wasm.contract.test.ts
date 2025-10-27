import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair, gettInkClient } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { Binary, Enum, PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58 } from "../src/address-utils"
import { raoToEth, tao } from "../src/balance-math"
import { ethers } from "ethers"
import { generateRandomEthersWallet, getPublicClient } from "../src/utils"
import { convertH160ToPublicKey } from "../src/address-utils"
import {
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, addNewSubnetwork, burnedRegister,
    sendProxyCall,
    startCall,
} from "../src/subtensor"
import { ETH_LOCAL_URL } from "../src/config";
import { ISTAKING_ADDRESS, ISTAKING_V2_ADDRESS, IStakingABI, IStakingV2ABI } from "../src/contracts/staking"
import { PublicClient } from "viem";
import { getInkClient, InkClient } from "@polkadot-api/ink-contracts"
import { contracts } from "@polkadot-api/descriptors"
import fs from "fs"

const Determinism = {
    Enforced: Enum('Enforced'),
    Relaxed: Enum('Relaxed')
} as const;

describe("Test neuron precompile add remove stake", () => {

    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const proxy = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    let inkClient: InkClient<contracts>;

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        // init variables got from await and async  
        api = await getDevnetApi()
        inkClient = await gettInkClient()

        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(alice.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(proxy.publicKey))
        let netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)

        console.log("test the case on subnet ", netuid)
    })

    it("Can upload contract", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        if (api === undefined) {
            throw new Error("api is undefined")
        }
        const bytecode = fs.readFileSync("bittensor.wasm")
        const upload = await api.tx.Contracts.upload_code({
            code: Binary.fromBytes(bytecode),
            storage_deposit_limit: BigInt(0),
            determinism: Determinism.Enforced
        })
        // const contract = await inkClient.upload(netuid, "bittensor", "bittensor.json")
        // assert.ok(contract !== undefined)
    })




});