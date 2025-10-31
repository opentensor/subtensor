import { getDevnetApi, getRandomSubstrateKeypair, getSignerFromKeypair, waitForTransactionWithRetry } from "../src/substrate"
import { devnet, MultiAddress } from "@polkadot-api/descriptors";
import { Binary, PolkadotSigner, TypedApi } from "polkadot-api";
import * as assert from "assert";
import { contracts } from "../.papi/descriptors";

import { ETH_LOCAL_URL } from "../src/config";
import { ISTAKING_ADDRESS, ISTAKING_V2_ADDRESS, IStakingABI, IStakingV2ABI } from "../src/contracts/staking"
import { getInkClient, InkClient, } from "@polkadot-api/ink-contracts"
import fs from "fs"
import { convertPublicKeyToSs58 } from "../src/address-utils";
import { addNewSubnetwork, burnedRegister, forceSetBalanceToSs58Address, sendWasmContractExtrinsic, startCall } from "../src/subtensor";
import { tao } from "../src/balance-math";

const bittensorWasmPath = "./bittensor/target/ink/bittensor.wasm"
const bittensorBytecode = fs.readFileSync(bittensorWasmPath)

describe("Test wasm contract", () => {

    let api: TypedApi<typeof devnet>
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    const hotkey2 = getRandomSubstrateKeypair();
    const coldkey2 = getRandomSubstrateKeypair();

    // let inkClient: InkClient<typeof contracts.bittensor>;
    let contractAddress: string;
    let inkClient: InkClient<typeof contracts.bittensor>;

    before(async () => {
        // init variables got from await and async  
        api = await getDevnetApi()

        inkClient = getInkClient(contracts.bittensor)

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey2.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey2.publicKey))
        let netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)

        console.log("test the case on subnet ", netuid)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), coldkey)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey2.publicKey), coldkey2)
    })

    it("Can instantiate contract", async () => {
        const signer = getSignerFromKeypair(coldkey);
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
            throw new Error("No events found after instantiating contract call")
        }
        contractAddress = codeStoredEvents[0].contract

        // transfer 10 Tao to contract then we can stake
        const transfer = await api.tx.Balances.transfer_keep_alive({
            dest: MultiAddress.Id(contractAddress),
            value: tao(1000),
        })
        await waitForTransactionWithRetry(api, transfer, signer)

        console.log("===== contractAddress", contractAddress)
    })


    it("Can query stake info from contract", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        // const signer = getSignerFromKeypair(coldkey);
        // const inkClient = getInkClient(contracts.bittensor)
        // const query = inkClient.message("dummy")
        // const data = query.encode()
        // No parameters needed
        // const queryTx = await api.tx.Contracts.call({
        //     dest: MultiAddress.Id(contractAddress),
        //     data: Binary.fromBytes(data.asBytes()),
        //     value: BigInt(0),
        //     gas_limit: {
        //         ref_time: BigInt(1000000000),
        //         proof_size: BigInt(10000000),
        //     },
        //     storage_deposit_limit: BigInt(10000000),
        // }).signAndSubmit(signer)

        // const response = await api.apis.ContractsApi.call(
        //     convertPublicKeyToSs58(coldkey.publicKey),
        //     contractAddress,
        //     BigInt(0),
        //     {
        //         ref_time: BigInt(1000000000),
        //         proof_size: BigInt(10000000),
        //     },
        //     BigInt(1000000000),
        //     Binary.fromBytes(data.asBytes()),
        //     undefined,
        // )

        // console.log("===== response", response.result.asBytes().toString())

    })

    it("Can add stake to contract", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        let amount = tao(800)

        const message = inkClient.message("add_stake")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            amount: amount,
        })

        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        const stakeAfterAddStake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeAfterAddStake !== undefined)
        assert.ok(stake !== undefined)
        assert.ok(stakeAfterAddStake > stake)
    })

    it("Can remove stake to contract", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stake !== undefined)

        let amount = stake / BigInt(2)
        const message = inkClient.message("remove_stake")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            amount: amount,
        })

        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        const stakeAfterAddStake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeAfterAddStake !== undefined)
        assert.ok(stake !== undefined)
        assert.ok(stakeAfterAddStake < stake)
    })
});