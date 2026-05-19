import { devnet, MultiAddress } from "@polkadot-api/descriptors";
import { getInkClient, InkClient, } from "@polkadot-api/ink-contracts";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import * as assert from "assert";
import fs from "fs";
import { Binary, TypedApi } from "polkadot-api";
import { contracts } from "../.papi/descriptors";
import { convertPublicKeyToSs58 } from "../src/address-utils";
import { tao } from "../src/balance-math";
import { getBalance, getDevnetApi, getRandomSubstrateKeypair, getSignerFromKeypair, waitForTransactionWithRetry } from "../src/substrate";
import { addNewSubnetwork, burnedRegister, forceSetBalanceToSs58Address, sendWasmContractExtrinsic, setAdminFreezeWindow, setTargetRegistrationsPerInterval, startCall } from "../src/subtensor";

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

    async function getContractStake(): Promise<bigint> {
        const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            netuid,
        ))?.stake

        assert.ok(stake !== undefined)
        return stake as bigint
    }

    async function getContractStakeOnRoot(): Promise<bigint> {
        const stake = (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(
            convertPublicKeyToSs58(hotkey.publicKey),
            contractAddress,
            0,
        ))?.stake

        assert.ok(stake !== undefined)
        return stake as bigint
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

    it("Check add_stake_recycle is not atomic operation", async () => {
        const stakeBefore = await getContractStakeOnRoot()
        const balanceBefore = await getBalance(api, convertPublicKeyToSs58(coldkey.publicKey))
        const contractBalanceBefore = (await api.query.System.Account.getValue(contractAddress)).data.free

        const message = inkClient.message("add_stake_recycle_no_revert")
        const data = message.encode({
            hotkey: Binary.fromBytes(hotkey.publicKey),
            netuid: 0,
            amount: tao(100),
        })

        const signer = getSignerFromKeypair(coldkey)
        await api.tx.Contracts.call({
            value: BigInt(0),
            dest: MultiAddress.Id(contractAddress),
            data: Binary.fromBytes(data.asBytes()),
            gas_limit: { ref_time: BigInt(10_000_000_000), proof_size: BigInt(10_000_000) },
            storage_deposit_limit: BigInt(1_000_000_000),
        }).signAndSubmit(signer)

        const stakeAfter = await getContractStakeOnRoot()
        const balanceAfter = await getBalance(api, convertPublicKeyToSs58(coldkey.publicKey))
        const contractBalanceAfter = (await api.query.System.Account.getValue(contractAddress)).data.free

        console.log("stake:    ", stakeBefore, "->", stakeAfter)
        console.log("contract: ", contractBalanceBefore, "->", contractBalanceAfter)
        console.log("coldkey:  ", balanceBefore, "->", balanceAfter)

        assert.ok(balanceBefore - balanceAfter < 10_000_000)

        // BUG: root stake credited even though chain ext returned CannotBurnOrRecycleOnRootSubnet.
        assert.ok(
            stakeAfter > stakeBefore,
            `BUG: not atomic root stake credited: before=${stakeBefore}, after=${stakeAfter}`,
        )

        // BUG: TAO withdrawn from the CONTRACT account (env.caller()).
        const contractDebit = contractBalanceBefore - contractBalanceAfter
        assert.ok(
            contractDebit >= tao(100) / BigInt(2),
            `BUG: not atomic contract debited: debit=${contractDebit}`,
        )
    })
});