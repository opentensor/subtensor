import { getDevnetApi, getRandomSubstrateKeypair, getSignerFromKeypair, waitForTransactionWithRetry } from "../src/substrate"
import { devnet, MultiAddress } from "@polkadot-api/descriptors";
import { Binary, TypedApi } from "polkadot-api";
import * as assert from "assert";
import { contracts } from "../.papi/descriptors";
import { getInkClient, InkClient, } from "@polkadot-api/ink-contracts"
import { forceSetBalanceToSs58Address, startCall, burnedRegister, setTargetRegistrationsPerInterval, setAdminFreezeWindow } from "../src/subtensor";
import fs from "fs"
import { convertPublicKeyToSs58 } from "../src/address-utils";
import { addNewSubnetwork, sendWasmContractExtrinsic } from "../src/subtensor";
import { tao } from "../src/balance-math";
import { KeyPair } from "@polkadot-labs/hdkd-helpers"

const bittensorWasmPath = "./bittensor/target/ink/bittensor.wasm"
const bittensorBytecode = fs.readFileSync(bittensorWasmPath)

describe("Test wasm contract", () => {

    let api: TypedApi<typeof devnet>
    let hotkey: KeyPair;
    let coldkey: KeyPair;

    let hotkey2: KeyPair;
    let coldkey2: KeyPair;

    // set initial netuid to 0 to avoid warning
    let netuid: number = 0;
    let contractAddress = "";
    let inkClient: InkClient<typeof contracts.bittensor>;

    async function addStakeViaContract(addStakeToContract: boolean) {
        if (contractAddress === "") {
            return;
        }

        const amount = tao(100)
        let message
        let dest
        if (addStakeToContract) {
            message = inkClient.message("add_stake")
            dest = contractAddress;
        } else {
            message = inkClient.message("caller_add_stake")
            dest = convertPublicKeyToSs58(coldkey.publicKey);
        }

        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            amount: amount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            dest,
            netuid,
        ))?.stake

        assert.ok(stake !== undefined)
        assert.ok(stake > BigInt(0))
    }

    async function initSecondColdAndHotkey() {
        hotkey2 = getRandomSubstrateKeypair();
        coldkey2 = getRandomSubstrateKeypair();
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey2.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey2.publicKey))
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey2.publicKey), coldkey2)
    }

    before(async () => {
        // init variables got from await and async  
        api = await getDevnetApi()
        await setAdminFreezeWindow(api);

        inkClient = getInkClient(contracts.bittensor)
        hotkey = getRandomSubstrateKeypair();
        coldkey = getRandomSubstrateKeypair();
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))

        netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)
        console.log("test the case on subnet ", netuid)
        await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid + 1, coldkey)
        await setTargetRegistrationsPerInterval(api, netuid)
    })

    beforeEach(async () => {
        hotkey = getRandomSubstrateKeypair();
        coldkey = getRandomSubstrateKeypair();
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), coldkey)

    });

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
    })


    it("Can query stake info from contract", async () => {

        const queryMessage = inkClient.message("get_stake_info_for_hotkey_coldkey_netuid")

        const data = queryMessage.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            coldkey: Binary.fromBytes(coldkey.publicKey),
            netuid: netuid,
        })

        const response = await api.apis.ContractsApi.call(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            BigInt(0),
            undefined,
            undefined,
            Binary.fromBytes(data.asBytes()),
        )

        assert.ok(response.result.success)
        const result = queryMessage.decode(response.result.value).value.value

        if (typeof result === "object" && "hotkey" in result && "coldkey" in result && "netuid" in result && "stake" in result && "locked" in result && "emission" in result && "tao_emission" in result && "drain" in result && "is_registered" in result) {
            assert.equal(result.hotkey, convertPublicKeyToSs58(hotkey.publicKey))
            assert.equal(result.coldkey, convertPublicKeyToSs58(coldkey.publicKey))
            assert.equal(result.netuid, netuid)
            assert.equal(result.is_registered, true)
        } else {
            throw new Error("result is not an object")
        }

    })

    it("Can add stake to contract", async () => {
        await addStakeViaContract(true)
    })

    it("Can remove stake to contract", async () => {
        await addStakeViaContract(true)
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
        await addStakeViaContract(true)
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
    })

    it("Can unstake all alpha from contract", async () => {
        await addStakeViaContract(true)
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
        await addStakeViaContract(true)
        await initSecondColdAndHotkey()
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
        await addStakeViaContract(true)
        await initSecondColdAndHotkey()
        // Get initial stake
        const stakeBeforeOrigin = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        const stakeBeforeDest = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey2.publicKey),
            netuid,
        ))?.stake

        assert.ok(stakeBeforeOrigin !== undefined && stakeBeforeOrigin > BigInt(0))
        assert.ok(stakeBeforeDest !== undefined)

        // Transfer stake
        const transferAmount = stakeBeforeOrigin / BigInt(2)
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
        assert.ok(stakeAfterOrigin < stakeBeforeOrigin!)
        assert.ok(stakeAfterDest > stakeBeforeDest!)
    })

    it("Can swap stake between networks", async () => {
        await addStakeViaContract(true)
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
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeBefore !== undefined)

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
        assert.ok(stakeAfter > stakeBefore!)
    })

    it("Can remove stake with limit", async () => {
        await addStakeViaContract(true)
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

        const message = inkClient.message("remove_stake_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            amount: stakeBefore / BigInt(2),
            limit_price: tao(1),
            allow_partial: false,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeAfter !== undefined)
        assert.ok(stakeAfter < stakeBefore!)
    })

    it("Can swap stake with limit", async () => {
        await addStakeViaContract(true)

        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        const stakeBefore2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid + 1,
        ))?.stake

        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
        assert.ok(stakeBefore2 !== undefined)

        const message = inkClient.message("swap_stake_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid + 1,
            amount: stakeBefore / BigInt(2),
            limit_price: tao(1),
            allow_partial: false,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

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

    it("Can remove stake full limit", async () => {
        await addStakeViaContract(true)
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))

        const message = inkClient.message("remove_stake_full_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: netuid,
            limit_price: tao(60),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stakeAfter !== undefined)
        assert.ok(stakeAfter < stakeBefore!)
    })

    it("Can set coldkey auto stake hotkey", async () => {
        const message = inkClient.message("set_coldkey_auto_stake_hotkey")
        const data = message.encode({
            netuid: netuid,
            hotkey: Binary.fromBytes(hotkey.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)

        let autoStakeHotkey = await api.query.SubtensorModule.AutoStakeDestination.getValue(
            contractAddress,
            netuid,
        )

        assert.ok(autoStakeHotkey !== undefined)
        assert.ok(autoStakeHotkey === convertPublicKeyToSs58(hotkey.publicKey))
    })

    it("Can add and remove proxy", async () => {
        const message = inkClient.message("add_proxy")
        const data = message.encode({
            delegate: Binary.fromBytes(hotkey.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        let proxies = await api.query.Proxy.Proxies.getValue(
            contractAddress,
        )
        assert.ok(proxies !== undefined)
        assert.ok(proxies.length > 0 && proxies[0].length > 0)
        assert.ok(proxies[0][0].delegate === convertPublicKeyToSs58(hotkey.publicKey))


        const removeMessage = inkClient.message("remove_proxy")
        const removeData = removeMessage.encode({
            delegate: Binary.fromBytes(hotkey.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, removeData)

        let proxiesAfterRemove = await api.query.Proxy.Proxies.getValue(
            contractAddress,
        )
        assert.ok(proxiesAfterRemove !== undefined)
        assert.ok(proxiesAfterRemove[0].length === 0)
    })

    it("Can get alpha price", async () => {
        const message = inkClient.message("get_alpha_price")
        const data = message.encode({
            netuid: netuid,
        })

        const response = await api.apis.ContractsApi.call(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            BigInt(0),
            undefined,
            undefined,
            Binary.fromBytes(data.asBytes()),
        )

        assert.ok(response.result.success)
        const result = message.decode(response.result.value).value.value

        assert.ok(result !== undefined)
    })

    it("Can caller add stake (fn 16)", async () => {
        await addStakeViaContract(false)
    })

    it("Can caller remove stake (fn 17)", async () => {
        await addStakeViaContract(false)
        const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stake !== undefined)
        const amount = stake / BigInt(2)
        const message = inkClient.message("caller_remove_stake")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid,
            amount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeAfter !== undefined && stakeAfter < stake!)
    })

    it("Can caller unstake_all (fn 18)", async () => {
        await addStakeViaContract(false)
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
        const message = inkClient.message("caller_unstake_all")
        const data = message.encode({ hotkey: Binary.fromBytes(hotkey.publicKey) })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeAfter !== undefined)
        assert.ok(stakeAfter < stakeBefore!)
    })

    it("Can caller unstake_all_alpha (fn 19)", async () => {
        await addStakeViaContract(false)
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
        const message = inkClient.message("caller_unstake_all_alpha")
        const data = message.encode({ hotkey: Binary.fromBytes(hotkey.publicKey) })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeAfter !== undefined)
        assert.ok(stakeAfter < stakeBefore!)
    })

    it("Can caller move_stake (fn 20)", async () => {
        await addStakeViaContract(false)
        await initSecondColdAndHotkey()
        const originStakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const destStakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey2.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake || BigInt(0)
        assert.ok(originStakeBefore !== undefined && originStakeBefore > BigInt(0))
        const moveAmount = originStakeBefore / BigInt(2)
        const message = inkClient.message("caller_move_stake")
        const data = message.encode({
            origin_hotkey: Binary.fromBytes(hotkey.publicKey),
            destination_hotkey: Binary.fromBytes(hotkey2.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid,
            amount: moveAmount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const originStakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const destStakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey2.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(originStakeAfter !== undefined && destStakeAfter !== undefined)
        assert.ok(originStakeAfter < originStakeBefore!)
        assert.ok(destStakeAfter > destStakeBefore)
    })

    it("Can caller transfer_stake (fn 21)", async () => {
        await addStakeViaContract(false)
        await initSecondColdAndHotkey()
        const stakeBeforeOrigin = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const stakeBeforeDest = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey2.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeBeforeOrigin !== undefined && stakeBeforeOrigin > BigInt(0))
        assert.ok(stakeBeforeDest !== undefined)
        const transferAmount = stakeBeforeOrigin / BigInt(2)
        const message = inkClient.message("caller_transfer_stake")
        const data = message.encode({
            destination_coldkey: Binary.fromBytes(coldkey2.publicKey),
            hotkey: Binary.fromBytes(hotkey.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid,
            amount: transferAmount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfterOrigin = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const stakeAfterDest = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey2.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeAfterOrigin !== undefined && stakeAfterDest !== undefined)
        assert.ok(stakeAfterOrigin < stakeBeforeOrigin!)
        assert.ok(stakeAfterDest > stakeBeforeDest!)
    })

    it("Can caller swap_stake (fn 22)", async () => {
        await addStakeViaContract(false)
        await initSecondColdAndHotkey()
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const stakeBefore2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid + 1,
        ))?.stake || BigInt(0)
        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
        const swapAmount = stakeBefore / BigInt(2)
        const message = inkClient.message("caller_swap_stake")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid + 1,
            amount: swapAmount,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const stakeAfter2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid + 1,
        ))?.stake
        assert.ok(stakeAfter !== undefined && stakeAfter2 !== undefined)
        assert.ok(stakeAfter < stakeBefore)
        assert.ok(stakeAfter2 > stakeBefore2)
    })

    it("Can caller add_stake_limit (fn 23)", async () => {
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeBefore !== undefined)
        const message = inkClient.message("caller_add_stake_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid,
            amount: tao(200),
            limit_price: tao(100),
            allow_partial: false,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeAfter !== undefined && stakeAfter > stakeBefore!)
    })

    it("Can caller remove_stake_limit (fn 24)", async () => {
        await addStakeViaContract(false)
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
        const message = inkClient.message("caller_remove_stake_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid,
            amount: stakeBefore / BigInt(2),
            limit_price: tao(1),
            allow_partial: false,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeAfter !== undefined && stakeAfter < stakeBefore!)
    })

    it("Can caller swap_stake_limit (fn 25)", async () => {
        await addStakeViaContract(false)
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const stakeBefore2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid + 1,
        ))?.stake
        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
        assert.ok(stakeBefore2 !== undefined)
        const message = inkClient.message("caller_swap_stake_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            origin_netuid: netuid,
            destination_netuid: netuid + 1,
            amount: stakeBefore / BigInt(2),
            limit_price: tao(1),
            allow_partial: false,
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        const stakeAfter2 = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid + 1,
        ))?.stake
        assert.ok(stakeAfter !== undefined && stakeAfter2 !== undefined)
        assert.ok(stakeAfter < stakeBefore)
        assert.ok(stakeAfter2 > stakeBefore2!)
    })

    it("Can caller remove_stake_full_limit (fn 26)", async () => {
        await addStakeViaContract(false)
        const stakeBefore = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeBefore !== undefined && stakeBefore > BigInt(0))
        const message = inkClient.message("caller_remove_stake_full_limit")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid,
            limit_price: tao(60),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const stakeAfter = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        ))?.stake
        assert.ok(stakeAfter !== undefined && stakeAfter < stakeBefore!)
    })

    it("Can caller set_coldkey_auto_stake_hotkey (fn 27)", async () => {
        await addStakeViaContract(false)
        await initSecondColdAndHotkey()
        const message = inkClient.message("caller_set_coldkey_auto_stake_hotkey")
        const data = message.encode({
            netuid,
            hotkey: Binary.fromBytes(hotkey2.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, data)
        const autoStakeHotkey = await api.query.SubtensorModule.AutoStakeDestination.getValue(
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid,
        )
        assert.ok(autoStakeHotkey === convertPublicKeyToSs58(hotkey2.publicKey))
    })

    it("Can caller add_proxy and remove_proxy (fn 28-29)", async () => {
        const addMessage = inkClient.message("caller_add_proxy")
        const addData = addMessage.encode({
            delegate: Binary.fromBytes(hotkey2.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, addData)
        let proxies = await api.query.Proxy.Proxies.getValue(convertPublicKeyToSs58(coldkey.publicKey))
        assert.ok(proxies !== undefined && proxies[0].length > 0)
        assert.ok(proxies[0][0].delegate === convertPublicKeyToSs58(hotkey2.publicKey))

        const removeMessage = inkClient.message("caller_remove_proxy")
        const removeData = removeMessage.encode({
            delegate: Binary.fromBytes(hotkey2.publicKey),
        })
        await sendWasmContractExtrinsic(api, coldkey, contractAddress, removeData)
        proxies = await api.query.Proxy.Proxies.getValue(convertPublicKeyToSs58(coldkey.publicKey))
        assert.ok(proxies !== undefined && proxies[0].length === 0)
    })
});