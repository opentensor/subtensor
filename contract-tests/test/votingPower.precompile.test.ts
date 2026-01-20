import * as assert from "assert";

import { getDevnetApi, getRandomSubstrateKeypair, getAliceSigner, getSignerFromKeypair, waitForTransactionWithRetry } from "../src/substrate"
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils"
import { IVotingPowerABI, IVOTING_POWER_ADDRESS } from "../src/contracts/votingPower"
import { forceSetBalanceToSs58Address, addNewSubnetwork, startCall } from "../src/subtensor";

describe("Test VotingPower Precompile", () => {
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>;

    // sudo account alice as signer
    let alice: PolkadotSigner;

    // init other variable
    let subnetId = 0;

    before(async () => {
        // init variables got from await and async
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner();

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))

        let netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)
        subnetId = netuid
    })

    describe("VotingPower Tracking Status Functions", () => {
        it("isVotingPowerTrackingEnabled returns false by default", async () => {
            const isEnabled = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "isVotingPowerTrackingEnabled",
                args: [subnetId]
            })

            assert.ok(isEnabled !== undefined, "isVotingPowerTrackingEnabled should return a value");
            assert.strictEqual(typeof isEnabled, 'boolean', "isVotingPowerTrackingEnabled should return a boolean");
            // By default, voting power tracking is disabled
            assert.strictEqual(isEnabled, false, "Voting power tracking should be disabled by default");
        });

        it("getVotingPowerDisableAtBlock returns 0 when not scheduled", async () => {
            const disableAtBlock = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "getVotingPowerDisableAtBlock",
                args: [subnetId]
            })

            assert.ok(disableAtBlock !== undefined, "getVotingPowerDisableAtBlock should return a value");
            assert.strictEqual(typeof disableAtBlock, 'bigint', "getVotingPowerDisableAtBlock should return a bigint");
            assert.strictEqual(disableAtBlock, BigInt(0), "Disable at block should be 0 when not scheduled");
        });

        it("getVotingPowerEmaAlpha returns default alpha value", async () => {
            const alpha = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "getVotingPowerEmaAlpha",
                args: [subnetId]
            })

            assert.ok(alpha !== undefined, "getVotingPowerEmaAlpha should return a value");
            assert.strictEqual(typeof alpha, 'bigint', "getVotingPowerEmaAlpha should return a bigint");
            // Default alpha is  0_003_570_000_000_000_000 // 0.00357 * 10^18 = 2 weeks e-folding (time-constant) @ 361
            assert.strictEqual(alpha, BigInt("3570000000000000"), "Default alpha should be 0.00357 * 10^18 (3570000000000000)");
        });
    });

    describe("VotingPower Query Functions", () => {
        it("getVotingPower returns 0 for hotkey without voting power", async () => {
            // Convert hotkey public key to bytes32 format (0x prefixed hex string)
            const hotkeyBytes32 = '0x' + Buffer.from(hotkey.publicKey).toString('hex');

            const votingPower = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "getVotingPower",
                args: [subnetId, hotkeyBytes32 as `0x${string}`]
            })

            assert.ok(votingPower !== undefined, "getVotingPower should return a value");
            assert.strictEqual(typeof votingPower, 'bigint', "getVotingPower should return a bigint");
            // Without voting power tracking enabled, voting power should be 0
            assert.strictEqual(votingPower, BigInt(0), "Voting power should be 0 when tracking is disabled");
        });

        it("getVotingPower returns 0 for unknown hotkey", async () => {
            // Generate a random hotkey that doesn't exist
            const randomHotkey = getRandomSubstrateKeypair();
            const randomHotkeyBytes32 = '0x' + Buffer.from(randomHotkey.publicKey).toString('hex');

            const votingPower = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "getVotingPower",
                args: [subnetId, randomHotkeyBytes32 as `0x${string}`]
            })

            assert.ok(votingPower !== undefined, "getVotingPower should return a value");
            assert.strictEqual(votingPower, BigInt(0), "Voting power should be 0 for unknown hotkey");
        });

        it("getTotalVotingPower returns 0 when no voting power exists", async () => {
            const totalVotingPower = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "getTotalVotingPower",
                args: [subnetId]
            })

            assert.ok(totalVotingPower !== undefined, "getTotalVotingPower should return a value");
            assert.strictEqual(typeof totalVotingPower, 'bigint', "getTotalVotingPower should return a bigint");
            assert.strictEqual(totalVotingPower, BigInt(0), "Total voting power should be 0 when tracking is disabled");
        });
    });

    describe("VotingPower with Tracking Enabled", () => {
        let enabledSubnetId: number;

        before(async () => {
            // Create a new subnet for this test
            const hotkey2 = getRandomSubstrateKeypair();
            const coldkey2 = getRandomSubstrateKeypair();

            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey2.publicKey))
            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey2.publicKey))

            enabledSubnetId = await addNewSubnetwork(api, hotkey2, coldkey2)
            await startCall(api, enabledSubnetId, coldkey2)

            // Enable voting power tracking via sudo
            const internalCall = api.tx.SubtensorModule.enable_voting_power_tracking({ netuid: enabledSubnetId })
            const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })
            await waitForTransactionWithRetry(api, tx, alice)
        });

        it("isVotingPowerTrackingEnabled returns true after enabling", async () => {
            const isEnabled = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "isVotingPowerTrackingEnabled",
                args: [enabledSubnetId]
            })

            assert.strictEqual(isEnabled, true, "Voting power tracking should be enabled");
        });

        it("getVotingPowerDisableAtBlock still returns 0 when enabled but not scheduled for disable", async () => {
            const disableAtBlock = await publicClient.readContract({
                abi: IVotingPowerABI,
                address: toViemAddress(IVOTING_POWER_ADDRESS),
                functionName: "getVotingPowerDisableAtBlock",
                args: [enabledSubnetId]
            })

            assert.strictEqual(disableAtBlock, BigInt(0), "Disable at block should still be 0");
        });
    });

    describe("All precompile functions are accessible", () => {
        it("All VotingPower precompile functions can be called", async () => {
            const hotkeyBytes32 = '0x' + Buffer.from(hotkey.publicKey).toString('hex');

            // Test all five functions
            const results = await Promise.all([
                publicClient.readContract({
                    abi: IVotingPowerABI,
                    address: toViemAddress(IVOTING_POWER_ADDRESS),
                    functionName: "getVotingPower",
                    args: [subnetId, hotkeyBytes32 as `0x${string}`]
                }),
                publicClient.readContract({
                    abi: IVotingPowerABI,
                    address: toViemAddress(IVOTING_POWER_ADDRESS),
                    functionName: "isVotingPowerTrackingEnabled",
                    args: [subnetId]
                }),
                publicClient.readContract({
                    abi: IVotingPowerABI,
                    address: toViemAddress(IVOTING_POWER_ADDRESS),
                    functionName: "getVotingPowerDisableAtBlock",
                    args: [subnetId]
                }),
                publicClient.readContract({
                    abi: IVotingPowerABI,
                    address: toViemAddress(IVOTING_POWER_ADDRESS),
                    functionName: "getVotingPowerEmaAlpha",
                    args: [subnetId]
                }),
                publicClient.readContract({
                    abi: IVotingPowerABI,
                    address: toViemAddress(IVOTING_POWER_ADDRESS),
                    functionName: "getTotalVotingPower",
                    args: [subnetId]
                })
            ]);

            // All functions should return defined values
            results.forEach((result: unknown, index: number) => {
                assert.ok(result !== undefined, `Function ${index} should return a value`);
            });

            // Verify types
            assert.strictEqual(typeof results[0], 'bigint', "getVotingPower should return bigint");
            assert.strictEqual(typeof results[1], 'boolean', "isVotingPowerTrackingEnabled should return boolean");
            assert.strictEqual(typeof results[2], 'bigint', "getVotingPowerDisableAtBlock should return bigint");
            assert.strictEqual(typeof results[3], 'bigint', "getVotingPowerEmaAlpha should return bigint");
            assert.strictEqual(typeof results[4], 'bigint', "getTotalVotingPower should return bigint");
        });
    });
});
