import * as assert from "assert";
import { getDevnetApi, getAliceSigner, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet, contracts, MultiAddress } from "@polkadot-api/descriptors"
import { Binary, PolkadotSigner, TypedApi } from "polkadot-api";

import { ETH_LOCAL_URL } from "../src/config";
import { ISTAKING_ADDRESS, ISTAKING_V2_ADDRESS, IStakingABI, IStakingV2ABI } from "../src/contracts/staking"
import { getInkClient, InkClient } from "@polkadot-api/ink-contracts"
import fs from "fs"
import { convertPublicKeyToSs58 } from "../src/address-utils";
import { forceSetBalanceToSs58Address } from "../src/subtensor";

const bittensorWasmPath = "./bittensor/target/ink/bittensor.wasm"
const bittensorBytecode = fs.readFileSync(bittensorWasmPath)
const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

describe("Test wasm contract", () => {

    let api: TypedApi<typeof devnet>
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    let inkClient: InkClient<typeof contracts.bittensor>;
    let contractAddress: string;

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        // init variables got from await and async  
        api = await getDevnetApi()
        alice = await getAliceSigner();
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
    })

    it("Can instantiate contract", async () => {
        const signer = await getAliceSigner();
        inkClient = getInkClient(contracts.bittensor)
        const constructor = inkClient.constructor('new')
        const data = constructor.encode()
        const instantiate_with_code = await api.tx.Contracts.instantiate_with_code({
            code: Binary.fromBytes(bittensorBytecode),
            storage_deposit_limit: BigInt(10000000),
            value: BigInt(0),
            gas_limit: {
                ref_time: BigInt(1000000000),
                proof_size: BigInt(1000000),
            },
            data: Binary.fromBytes(data.asBytes()),
            salt: Binary.fromHex("0x"),
        }).signAndSubmit(signer)


        let codeStoredEvents = await api.event.Contracts.Instantiated.filter(instantiate_with_code.events)
        if (codeStoredEvents.length === 0) {
            throw new Error("No events found")
        }
        contractAddress = codeStoredEvents[0].contract

        console.log("===== contractAddress", contractAddress)
    })


    it("Can query stake info from contract", async () => {

    })


});