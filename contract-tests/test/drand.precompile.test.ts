import * as assert from "assert";

import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate";
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors";
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils";
import { IDrandABI, IDRAND_ADDRESS } from "../src/contracts/drand";
import { forceSetBalanceToSs58Address, addNewSubnetwork, startCall } from "../src/subtensor";

describe("Test Drand Precompile", () => {
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>;

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL);
        api = await getDevnetApi();

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey));
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey));

        const netuid = await addNewSubnetwork(api, hotkey, coldkey);
        await startCall(api, netuid, coldkey);
    });

    describe("Drand Randomness Functions", () => {
        it("getLastStoredRound returns a value", async () => {
            const lastRound = await publicClient.readContract({
                abi: IDrandABI,
                address: toViemAddress(IDRAND_ADDRESS),
                functionName: "getLastStoredRound",
                args: [],
            });

            assert.ok(lastRound !== undefined, "getLastStoredRound should return a value");
            assert.strictEqual(
                typeof lastRound,
                "bigint",
                "getLastStoredRound should return a bigint"
            );
            assert.ok(lastRound >= BigInt(0), "Last stored round should be non-negative");
        });

        it("getRandomness returns bytes32 for a round", async () => {
            const lastRound = await publicClient.readContract({
                abi: IDrandABI,
                address: toViemAddress(IDRAND_ADDRESS),
                functionName: "getLastStoredRound",
                args: [],
            });

            const roundToQuery = lastRound > BigInt(0) ? lastRound : BigInt(1);

            const randomness = await publicClient.readContract({
                abi: IDrandABI,
                address: toViemAddress(IDRAND_ADDRESS),
                functionName: "getRandomness",
                args: [roundToQuery],
            });

            assert.ok(randomness !== undefined, "getRandomness should return a value");
            assert.strictEqual(
                typeof randomness,
                "string",
                "getRandomness should return a hex string (bytes32)"
            );
            assert.strictEqual(
                randomness.length,
                66,
                "bytes32 should be 0x + 64 hex chars"
            );
        });

        it("getRandomness for non-existent round returns zero bytes", async () => {
            // Use a very high round number that will not have a stored pulse
            const nonExistentRound = BigInt(999999999);
            const randomness = await publicClient.readContract({
                abi: IDrandABI,
                address: toViemAddress(IDRAND_ADDRESS),
                functionName: "getRandomness",
                args: [nonExistentRound],
            });

            assert.ok(randomness !== undefined, "getRandomness should return a value");
            const zeroBytes32 = "0x" + "0".repeat(64);
            assert.strictEqual(
                randomness.toLowerCase(),
                zeroBytes32,
                "getRandomness for non-existent round should return zero bytes32"
            );
        });
    });
});
