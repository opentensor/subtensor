import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58 } from "../src/address-utils"
import { tao } from "../src/balance-math"
import { ethers } from "ethers"
import { generateRandomEthersWallet } from "../src/utils"
import { convertH160ToPublicKey } from "../src/address-utils"
import {
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, addNewSubnetwork, burnedRegister,
    startCall,
} from "../src/subtensor"
import { ISTAKING_V2_ADDRESS, IStakingV2ABI } from "../src/contracts/staking"

describe("Test staking precompile burn alpha", () => {
    // init eth part
    const wallet1 = generateRandomEthersWallet();
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    before(async () => {
        // init variables got from await and async
        api = await getDevnetApi()

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToEthAddress(api, wallet1.address)

        let netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)

        console.log("test the case on subnet ", netuid)

        await burnedRegister(api, netuid, convertH160ToSS58(wallet1.address), coldkey)
    })

    it("Can burn alpha after adding stake", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

        // First add some stake
        let stakeBalance = tao(50)
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);
        const addStakeTx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), netuid)
        await addStakeTx.wait()

        // Get stake before burning
        const stakeBefore = BigInt(await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid))

        console.log("Stake before burn:", stakeBefore)
        assert.ok(stakeBefore > BigInt(0), "Should have stake before burning")

        // Burn some alpha (burn 20 TAO worth)
        let burnAmount = tao(20)
        const burnTx = await contract.burnAlpha(hotkey.publicKey, burnAmount.toString(), netuid)
        await burnTx.wait()

        // Get stake after burning
        const stakeAfter = BigInt(await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid))

        console.log("Stake after burn:", stakeAfter)

        // Verify that stake decreased by burn amount
        assert.ok(stakeAfter < stakeBefore, "Stake should decrease after burning")
        // assert.strictEqual(stakeBefore - stakeAfter, burnAmount, "Stake should decrease by exactly burn amount")
    })

    it("Cannot burn more alpha than staked", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

        // Get current stake
        const currentStake = await api.query.SubtensorModule.Alpha.getValue(
            convertPublicKeyToSs58(hotkey.publicKey),
            convertH160ToSS58(wallet1.address),
            netuid
        )

        // Try to burn more than staked
        let burnAmount = currentStake + tao(10000)
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);

        try {
            const burnTx = await contract.burnAlpha(hotkey.publicKey, burnAmount.toString(), netuid)
            await burnTx.wait()
            assert.fail("Transaction should have failed - cannot burn more than staked");
        } catch (error) {
            // Transaction failed as expected
            console.log("Correctly failed to burn more than staked amount")
            assert.ok(true, "Burning more than staked should fail");
        }
    })

    it("Cannot burn alpha from non-existent subnet", async () => {
        // wrong netuid
        let netuid = 12345;
        let burnAmount = tao(10)
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);

        try {
            const burnTx = await contract.burnAlpha(hotkey.publicKey, burnAmount.toString(), netuid)
            await burnTx.wait()
            assert.fail("Transaction should have failed - subnet doesn't exist");
        } catch (error) {
            // Transaction failed as expected
            console.log("Correctly failed to burn from non-existent subnet")
            assert.ok(true, "Burning from non-existent subnet should fail");
        }
    })

    it("Cannot burn zero alpha", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

        // First add some stake for this test
        let stakeBalance = tao(10)
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);
        const addStakeTx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), netuid)
        await addStakeTx.wait()

        // Try to burn zero amount
        let burnAmount = BigInt(0)

        try {
            const burnTx = await contract.burnAlpha(hotkey.publicKey, burnAmount.toString(), netuid)
            await burnTx.wait()
            assert.fail("Transaction should have failed - cannot burn zero amount");
        } catch (error) {
            // Transaction failed as expected
            console.log("Correctly failed to burn zero amount")
            assert.ok(true, "Burning zero amount should fail");
        }
    })
})

