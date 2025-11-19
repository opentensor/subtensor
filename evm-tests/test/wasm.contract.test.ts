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

/*
The test file is to verify all the functions in the wasm contract are working correctly.
The test call each function defined in the contract extension.

Current issue:
Can't generate the descriptor for the wasm contract if we add the function to return a complicate struct.
https://github.com/polkadot-api/polkadot-api/issues/1207
*/

describe("Test wasm contract", () => {

    let api: TypedApi<typeof devnet>
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    const hotkey2 = getRandomSubstrateKeypair();
    const coldkey2 = getRandomSubstrateKeypair();

    // set initial netuid to 0 to avoid warning
    let netuid: number = 0;
    // let inkClient: InkClient<typeof contracts.bittensor>;
    let contractAddress: string;
    let inkClient: InkClient<typeof contracts.bittensor>;

    async function addStakeWhenWithoutStake() {
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
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey2.publicKey))
        netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)
        console.log("test the case on subnet ", netuid)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), coldkey)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey2.publicKey), coldkey2)

        await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid + 1, coldkey)
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

    it("Can forward tokens to contract", async () => {
        const balanceBefore = await api.query.System.Account.getValue(
            convertPublicKeyToSs58(hotkey2.publicKey),
        )
        assert.ok(balanceBefore !== undefined)
        console.log("===== balanceBefore", balanceBefore)
        // assert.ok(balanceBefore.data.free > BigInt(0))

        const message = inkClient.message("forward_tokens")
        const amount = tao(1)
        const data = message.encode({
            recipient: Binary.fromBytes(hotkey2.publicKey),
        })
        // Pass native token value as the last parameter
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data, amount)

        const balance = await api.query.System.Account.getValue(
            convertPublicKeyToSs58(hotkey2.publicKey),
        )
        console.log("===== balance", balance)
        assert.ok(balance !== undefined)
        assert.ok(balance.data.free > BigInt(0))
    })


    // it("Can query stake info from contract", async () => {
    //     // const signer = getSignerFromKeypair(coldkey);
    //     const inkClient = getInkClient(contracts.bittensor)
    //     const queryMessage = inkClient.message("get_stake_info_for_hotkey_coldkey_netuid")
    //     const data = queryMessage.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         coldkey: Binary.fromBytes(coldkey.publicKey),
    //         netuid: netuid,
    //     })

    //     const response = await api.tx.Contracts.call({
    //         dest: MultiAddress.Id(contractAddress),
    //         data: Binary.fromBytes(data.asBytes()),
    //         value: BigInt(0),
    //         gas_limit: {
    //             ref_time: BigInt(1000000000),
    //             proof_size: BigInt(10000000),
    //         },
    //         storage_deposit_limit: BigInt(10000000),
    //     })

    //     console.log("===== response", response)

    // })

    // it("Can add stake to contract", async () => {
    //     await addStakeWhenWithoutStake()
    // })

    // it("Can remove stake to contract", async () => {
    //     await addStakeWhenWithoutStake()
    //     const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stake !== undefined)

    //     let amount = stake / BigInt(2)
    //     const message = inkClient.message("remove_stake")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         netuid: netuid,
    //         amount: amount,
    //     })

    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     const stakeAfterAddStake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfterAddStake !== undefined)
    //     assert.ok(stake !== undefined)
    //     assert.ok(stakeAfterAddStake < stake)
    // })

    // it("Can unstake all from contract", async () => {
    //     await addStakeWhenWithoutStake()

    //     // Get stake before unstake_all
    //     const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

    //     // Call unstake_all
    //     const unstakeMessage = inkClient.message("unstake_all")
    //     const unstakeData = unstakeMessage.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, unstakeData)

    //     // Verify stake is now zero
    //     const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfter !== undefined)
    //     assert.equal(stakeAfter, BigInt(0))
    // })

    // it("Can unstake all alpha from contract", async () => {
    //     await addStakeWhenWithoutStake()
    //     // Get stake before unstake_all_alpha
    //     const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

    //     // Call unstake_all_alpha
    //     const message = inkClient.message("unstake_all_alpha")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify stake is now zero
    //     const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfter !== undefined)
    //     assert.equal(stakeAfter, BigInt(0))
    // })

    // it("Can move stake between hotkeys", async () => {
    //     await addStakeWhenWithoutStake()

    //     // Get initial stakes
    //     const originStakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const destStakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey2.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake || BigInt(0)

    //     assert.ok(originStakeBefore !== undefined && originStakeBefore > BigInt(0))

    //     // Move stake
    //     const moveAmount = originStakeBefore / BigInt(2)
    //     const message = inkClient.message("move_stake")
    //     const data = message.encode({
    //         origin_hotkey: Binary.fromBytes(hotkey.publicKey),
    //         destination_hotkey: Binary.fromBytes(hotkey2.publicKey),
    //         origin_netuid: netuid,
    //         destination_netuid: netuid,
    //         amount: moveAmount,
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify stakes changed
    //     const originStakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const destStakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey2.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(originStakeAfter !== undefined)
    //     assert.ok(destStakeAfter !== undefined)
    //     assert.ok(originStakeAfter < originStakeBefore!)
    //     assert.ok(destStakeAfter > destStakeBefore)
    // })

    // it("Can transfer stake between coldkeys", async () => {
    //     await addStakeWhenWithoutStake()

    //     // Get initial stake
    //     const stakeBeforeOrigin = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const stakeBeforeDest = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         convertPublicKeyToSs58(coldkey2.publicKey),
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeBeforeOrigin !== undefined && stakeBeforeOrigin > BigInt(0))
    //     assert.ok(stakeBeforeDest !== undefined)

    //     // Transfer stake
    //     const transferAmount = stakeBeforeOrigin / BigInt(2)
    //     const message = inkClient.message("transfer_stake")
    //     const data = message.encode({
    //         destination_coldkey: Binary.fromBytes(coldkey2.publicKey),
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         origin_netuid: netuid,
    //         destination_netuid: netuid,
    //         amount: transferAmount,
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify stake transferred
    //     const stakeAfterOrigin = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const stakeAfterDest = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         convertPublicKeyToSs58(coldkey2.publicKey),
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfterOrigin !== undefined)
    //     assert.ok(stakeAfterDest !== undefined)
    //     assert.ok(stakeAfterOrigin < stakeBeforeOrigin!)
    //     assert.ok(stakeAfterDest > stakeBeforeDest!)
    // })

    // it("Can swap stake between networks", async () => {
    //     await addStakeWhenWithoutStake()

    //     // Get initial stakes
    //     const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const stakeBefore2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid + 1,
    //     ))?.stake || BigInt(0)

    //     assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

    //     // Swap stake
    //     const swapAmount = stakeBefore / BigInt(2)
    //     const message = inkClient.message("swap_stake")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         origin_netuid: netuid,
    //         destination_netuid: netuid + 1,
    //         amount: swapAmount,
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify stakes swapped
    //     const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const stakeAfter2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid + 1,
    //     ))?.stake

    //     assert.ok(stakeAfter !== undefined)
    //     assert.ok(stakeAfter2 !== undefined)
    //     assert.ok(stakeAfter < stakeBefore)
    //     assert.ok(stakeAfter2 > stakeBefore2)
    // })

    // it("Can add stake with limit", async () => {
    //     await addStakeWhenWithoutStake()
    //     const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeBefore !== undefined)

    //     const message = inkClient.message("add_stake_limit")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         netuid: netuid,
    //         amount: tao(200),
    //         limit_price: tao(100),
    //         allow_partial: false,
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     // Verify stake was added
    //     const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     assert.ok(stakeAfter !== undefined)
    //     assert.ok(stakeAfter > stakeBefore!)
    // })

    // it("Can remove stake with limit", async () => {
    //     await addStakeWhenWithoutStake()
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
    //         limit_price: tao(1),
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
    //     await addStakeWhenWithoutStake()

    //     const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const stakeBefore2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid + 1,
    //     ))?.stake

    //     assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
    //     assert.ok(stakeBefore2 !== undefined)

    //     const message = inkClient.message("swap_stake_limit")
    //     const data = message.encode({
    //         hotkey: Binary.fromBytes(hotkey.publicKey),
    //         origin_netuid: netuid,
    //         destination_netuid: netuid + 1,
    //         amount: stakeBefore / BigInt(2),
    //         limit_price: tao(1),
    //         allow_partial: false,
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid,
    //     ))?.stake

    //     const stakeAfter2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
    //         convertPublicKeyToSs58(hotkey.publicKey),
    //         contractAddress,
    //         netuid + 1,
    //     ))?.stake

    //     assert.ok(stakeAfter !== undefined)
    //     assert.ok(stakeAfter2 !== undefined)
    //     assert.ok(stakeAfter < stakeBefore)
    //     assert.ok(stakeAfter2 > stakeBefore2)
    // })

    // it("Can remove stake full limit", async () => {
    //     await addStakeWhenWithoutStake()

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
    //     const message = inkClient.message("set_coldkey_auto_stake_hotkey")
    //     const data = message.encode({
    //         netuid: netuid,
    //         hotkey: Binary.fromBytes(hotkey2.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

    //     let autoStakeHotkey = await api.query.SubtensorModule.AutoStakeDestination.getValue(
    //         contractAddress,
    //         netuid,
    //     )

    //     assert.ok(autoStakeHotkey !== undefined)
    //     assert.ok(autoStakeHotkey === convertPublicKeyToSs58(hotkey2.publicKey))
    // })

    // it("Can add and remove proxy", async () => {
    //     const message = inkClient.message("add_proxy")
    //     const data = message.encode({
    //         delegate: Binary.fromBytes(hotkey2.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
    //     let proxies = await api.query.Proxy.Proxies.getValue(
    //         contractAddress,
    //     )
    //     assert.ok(proxies !== undefined)
    //     assert.ok(proxies.length > 0 && proxies[0].length > 0)
    //     assert.ok(proxies[0][0].delegate === convertPublicKeyToSs58(hotkey2.publicKey))


    //     const removeMessage = inkClient.message("remove_proxy")
    //     const removeData = removeMessage.encode({
    //         delegate: Binary.fromBytes(hotkey2.publicKey),
    //     })
    //     await sendWasmContractExtrinsic(api, coldkey, contractAddress, removeData)

    //     let proxiesAfterRemove = await api.query.Proxy.Proxies.getValue(
    //         contractAddress,
    //     )
    //     assert.ok(proxiesAfterRemove !== undefined)
    //     assert.ok(proxiesAfterRemove[0].length === 0)
    // })
});