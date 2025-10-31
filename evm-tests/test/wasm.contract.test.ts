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

    const hotkey3 = getRandomSubstrateKeypair();
    const coldkey3 = getRandomSubstrateKeypair();

    // set initial netuid to 0 to avoid warning
    let netuid: number = 0;
    // let inkClient: InkClient<typeof contracts.bittensor>;
    let contractAddress: string;
    let inkClient: InkClient<typeof contracts.bittensor>;

    async function addStakeWithoutStake() {
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeBefore !== undefined)
        if (stakeBefore > BigInt(0)) {
            return;
        }

        const amount = tao(100)
        const message = inkClient.message("add_stake")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            amount: amount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stake !== undefined)
        assert.ok(stake > BigInt(0))
    }


    before(async () => {
        // init variables got from await and async  
        api = await getDevnetApi()

        inkClient = getInkClient(contracts.bittensor)

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey2.publicKey))
        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey3.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey2.publicKey))
        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey3.publicKey))
        netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)
        console.log("test the case on subnet ", netuid)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), coldkey)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey2.publicKey), coldkey2)

        await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid + 1, coldkey)
        // await burnedRegister(api, netuid + 1, convertPublicKeyToSs58(hotkey3.publicKey), coldkey3)
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
            value: tao(2000),
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
        await addStakeWithoutStake()
    })

    it("Can remove stake to contract", async () => {
        await addStakeWithoutStake()
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

    it("Can unstake all from contract", async () => {
        await addStakeWithoutStake()

        // Get stake before unstake_all
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

        // Call unstake_all
        const unstakeMessage = inkClient.message("unstake_all")
        const unstakeData = unstakeMessage.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, unstakeData)

        // Verify stake is now zero
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeAfter !== undefined)
        assert.equal(stakeAfter, BigInt(0))

        const stakeMessage = inkClient.message("add_stake")
        const stakeData = stakeMessage.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            amount: tao(800),
        })

        await sendWasmContractExtrinsic(api, coldkey, contractAddress, stakeData)
    })

    it("Can unstake all alpha from contract", async () => {
        await addStakeWithoutStake()
        // Get stake before unstake_all_alpha
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

        // Call unstake_all_alpha
        const message = inkClient.message("unstake_all_alpha")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        // Verify stake is now zero
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeAfter !== undefined)
        assert.equal(stakeAfter, BigInt(0))
    })

    it("Can move stake between hotkeys", async () => {
        await addStakeWithoutStake()

        // Get initial stakes
        const originStakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        const destStakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey2.publicKey),
            contractAddress,
            netuid,
        ))?.stake || BigInt(0)

        assert.ok(originStakeBefore !== undefined && originStakeBefore > BigInt(0))

        // Move stake
        const moveAmount = originStakeBefore / BigInt(2)
        const message = inkClient.message("move_stake")
        const data = message.encode({
            origin_hotkey: Binary.fromBytes(hotkey.publicKey),
            destination_hotkey: Binary.fromBytes(hotkey2.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid,
            amount: moveAmount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        // Verify stakes changed
        const originStakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        const destStakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey2.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(originStakeAfter !== undefined)
        assert.ok(destStakeAfter !== undefined)
        assert.ok(originStakeAfter < originStakeBefore!)
        assert.ok(destStakeAfter > destStakeBefore)
    })

    it("Can transfer stake between coldkeys", async () => {
        await addStakeWithoutStake()

        // Get initial stake
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

        // Transfer stake
        const transferAmount = stakeBefore / BigInt(2)
        const message = inkClient.message("transfer_stake")
        const data = message.encode({
            destination_coldkey: Binary.fromBytes(coldkey2.publicKey),
            hotkey: Binary.fromBytes(hotkey.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid,
            amount: transferAmount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        // Verify stake transferred
        const stakeAfterOrigin = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        const stakeAfterDest = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey2.publicKey),
            netuid,
        ))?.stake

        assert.ok(stakeAfterOrigin !== undefined)
        assert.ok(stakeAfterDest !== undefined)
        assert.ok(stakeAfterOrigin < stakeBefore!)
    })

    it("Can swap stake between networks", async () => {
        await addStakeWithoutStake()

        // Get initial stakes
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        const stakeBefore2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid + 1,
        ))?.stake || BigInt(0)

        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

        // Swap stake
        const swapAmount = stakeBefore / BigInt(2)
        const message = inkClient.message("swap_stake")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid + 1,
            amount: swapAmount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        // Verify stakes swapped
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        const stakeAfter2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid + 1,
        ))?.stake

        assert.ok(stakeAfter !== undefined)
        assert.ok(stakeAfter2 !== undefined)
        assert.ok(stakeAfter < stakeBefore)
        assert.ok(stakeAfter2 > stakeBefore2)
    })

    it("Can add stake with limit", async () => {
        const message = inkClient.message("add_stake_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            amount: tao(200),
            limit_price: tao(100),
            allow_partial: false,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        // Verify stake was added
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeAfter !== undefined)
    })

    // it("Can remove stake with limit", async () => {
    //     let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

    //     // First add stake
    //     const addMessage = inkClient.message("add_stake")
    //     const addData = addMessage.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         netuid: netuid,
    //         amount: tao(300),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, addData)

    //     const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

    //     const message = inkClient.message("remove_stake_limit")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         netuid: netuid,
    //         amount: stakeBefore / BigInt(2),
    //         limit_price: tao(50),
    //         allow_partial: false,
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfter !== undefined)
    //     assert.ok(stakeAfter < stakeBefore!)
    // })

    // it("Can swap stake with limit", async () => {
    //     let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

    //     // Create second network
    //     const hotkey4 = getRandomSubstrateKeypair();
    //     const coldkey4 = getRandomSubstrateKeypair();
    //     await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey4.publicKey))
    //     await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey4.publicKey))

    //     let netuid2 = await addNewSubnetwork(api, hotkey4, coldkey4)
    //     await startCall(api, netuid2, coldkey4)
    //     await burnedRegister(api, netuid2, convertPublicKeyToSs58(hotkey4.publicKey), coldkey4)

    //     // Add stake to origin network
    //     const addMessage = inkClient.message("add_stake")
    //     const addData = addMessage.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         netuid: netuid,
    //         amount: tao(500),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, addData)

    //     const stakeBefore1 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeBefore1 !== undefined && stakeBefore1 > BigInt(0))

    //     const message = inkClient.message("swap_stake_limit")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         origin_netuid: netuid,
    //         destination_netuid: netuid2,
    //         amount: stakeBefore1 / BigInt(2),
    //         limit_price: tao(75),
    //         allow_partial: false,
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     const stakeAfter1 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfter1 !== undefined)
    //     assert.ok(stakeAfter1 < stakeBefore1!)
    // })

    // it("Can remove stake full limit", async () => {
    //     let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

    //     // First add stake
    //     const addMessage = inkClient.message("add_stake")
    //     const addData = addMessage.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         netuid: netuid,
    //         amount: tao(700),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, addData)

    //     const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

    //     const message = inkClient.message("remove_stake_full_limit")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         netuid: netuid,
    //         limit_price: tao(60),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfter !== undefined)
    //     assert.ok(stakeAfter < stakeBefore!)
    // })

    // it("Can set coldkey auto stake hotkey", async () => {
    //     let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

    //     const message = inkClient.message("set_coldkey_auto_stake_hotkey")
    //     const data = message.encode({
    //         netuid: netuid,
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify the call succeeded (no error thrown)
    //     assert.ok(true)
    // })

    // it("Can add proxy", async () => {
    //     const message = inkClient.message("add_proxy")
    //     const data = message.encode({
    //         delegate: Binary.fromBytes(hotkey.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify the call succeeded (no error thrown)
    //     assert.ok(true)
    // })

    // it("Can remove proxy", async () => {
    //     // First add proxy
    //     const addMessage = inkClient.message("add_proxy")
    //     const addData = addMessage.encode({
    //         delegate: Binary.fromBytes(hotkey2.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, addData)

    //     // Then remove proxy
    //     const message = inkClient.message("remove_proxy")
    //     const data = message.encode({
    //         delegate: Binary.fromBytes(hotkey2.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify the call succeeded (no error thrown)
    //     assert.ok(true)
    // })
});