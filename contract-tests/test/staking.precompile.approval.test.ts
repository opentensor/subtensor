import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58 } from "../src/address-utils"
import { raoToEth, tao } from "../src/balance-math"
import { ethers } from "ethers"
import { generateRandomEthersWallet, getPublicClient } from "../src/utils"
import { convertH160ToPublicKey } from "../src/address-utils"
import {
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, addNewSubnetwork, burnedRegister,
    sendProxyCall,
    startCall,
    getStake,
} from "../src/subtensor"
import { ETH_LOCAL_URL } from "../src/config";
import { ISTAKING_ADDRESS, ISTAKING_V2_ADDRESS, IStakingABI, IStakingV2ABI } from "../src/contracts/staking"
import { PublicClient } from "viem";

describe("Test approval in staking precompile", () => {
    // init eth part
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    let publicClient: PublicClient;
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const proxy = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>
    let stakeNetuid: number;

    let expectedAllowance = BigInt(0);

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        // init variables got from await and async
        api = await getDevnetApi()

        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(alice.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(proxy.publicKey))
        await forceSetBalanceToEthAddress(api, wallet1.address)
        await forceSetBalanceToEthAddress(api, wallet2.address)
        let netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)

        console.log("test the case on subnet ", netuid)

        await burnedRegister(api, netuid, convertH160ToSS58(wallet1.address), coldkey)
        await burnedRegister(api, netuid, convertH160ToSS58(wallet2.address), coldkey)

        // add stake as wallet1
        {
            stakeNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
            // the unit in V2 is RAO, not ETH
            let stakeBalance = tao(20)
            const stakeBefore = await getStake(api, convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet1.address), stakeNetuid)
            const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);
            const tx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), stakeNetuid)
            await tx.wait()

            const stakeFromContract = BigInt(
                await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), stakeNetuid)
            );

            assert.ok(stakeFromContract > stakeBefore)
            const stakeAfter = await getStake(api, convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet1.address), stakeNetuid)
            assert.ok(stakeAfter > stakeBefore)
        }
    })

    it("Can't transfer from account without approval", async () => {
        try {
            // wallet2 tries to transfer from wallet1
            const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);
            const tx = await contract.transferStakeFrom(
                wallet1.address, // source
                wallet2.address, // destination
                hotkey.publicKey,
                stakeNetuid,
                stakeNetuid,
                1
            )
            await tx.wait();

            assert.fail("should have reverted due to missing allowance");
        } catch (e) {
            assert.equal(e.reason, "trying to spend more than allowed", "wrong revert message");
        }
    })

    it("Can approve some amount", async () => {
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);

        {
            let allowance = BigInt(
                await contract.allowance(
                    wallet1.address, // source
                    wallet2.address, // spender
                    stakeNetuid,
                )
            );
            assert.equal(allowance, expectedAllowance, "default allowance should be 0");
        }

        {
            const tx = await contract.approve(
                wallet2.address, // spender
                stakeNetuid,
                tao(10)
            )
            await tx.wait();

            expectedAllowance += BigInt(tao(10));

            let allowance = BigInt(
                await contract.allowance(
                    wallet1.address, // source
                    wallet2.address, // spender
                    stakeNetuid,
                )
            );
            assert.equal(allowance, expectedAllowance, "should have set allowance");
        }
    })

    it("Can now use transfer from", async () => {
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);

        // wallet2 transfer from wallet1
        const tx = await contract.transferStakeFrom(
            wallet1.address, // source
            wallet2.address, // destination
            hotkey.publicKey,
            stakeNetuid,
            stakeNetuid,
            tao(5)
        )
        await tx.wait();

        expectedAllowance -= BigInt(tao(5));

        {
            let allowance = BigInt(
                await contract.allowance(
                    wallet1.address, // source
                    wallet2.address, // spender
                    stakeNetuid,
                )
            );
            assert.equal(allowance, expectedAllowance, "allowance should now be 500");
        }
    })

    it("Can't use transfer from with amount too high", async () => {
        try {
            // wallet2 tries to transfer from wallet1
            const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);
            const tx = await contract.transferStakeFrom(
                wallet1.address, // source
                wallet2.address, // destination
                hotkey.publicKey,
                stakeNetuid,
                stakeNetuid,
                expectedAllowance + BigInt(1)
            )
            await tx.wait();

            assert.fail("should have reverted due to missing allowance");
        } catch (e) {
            assert.equal(e.reason, "trying to spend more than allowed", "wrong revert message");
        }
    })

    it("Approval functions works as expected", async () => {
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);

        {
            const tx = await contract.increaseAllowance(
                wallet2.address, // spender
                stakeNetuid,
                tao(10)
            )
            await tx.wait();

            expectedAllowance += BigInt(tao(10));

            let allowance = BigInt(
                await contract.allowance(
                    wallet1.address, // source
                    wallet2.address, // spender
                    stakeNetuid,
                )
            );
            assert.equal(allowance, expectedAllowance, "allowance have been increased correctly");
        }

        {
            const tx = await contract.decreaseAllowance(
                wallet2.address, // spender
                stakeNetuid,
                tao(2)
            )
            await tx.wait();

            expectedAllowance -= BigInt(tao(2));

            let allowance = BigInt(
                await contract.allowance(
                    wallet1.address, // source
                    wallet2.address, // spender
                    stakeNetuid,
                )
            );
            assert.equal(allowance, expectedAllowance, "allowance have been decreased correctly");
        }

        {
            const tx = await contract.approve(
                wallet2.address, // spender
                stakeNetuid,
                0
            )
            await tx.wait();

            expectedAllowance = BigInt(0);

            let allowance = BigInt(
                await contract.allowance(
                    wallet1.address, // source
                    wallet2.address, // spender
                    stakeNetuid,
                )
            );
            assert.equal(allowance, expectedAllowance, "allowance have been overwritten correctly");
        }
    })
})
