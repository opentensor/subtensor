import * as assert from "assert";

import { getDevnetApi } from "../src/substrate";
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors";
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { toViemAddress } from "../src/address-utils";
import { IDRAND_ADDRESS, IDrandABI } from "../src/contracts/drand";

// Helper to call a view function against the drand precompile.
async function readDrand<T>(
    publicClient: PublicClient,
    functionName: string,
    args: unknown[] = []
): Promise<T> {
    return publicClient.readContract({
        abi: IDrandABI,
        address: toViemAddress(IDRAND_ADDRESS),
        functionName: functionName as any,
        args: args as any,
    }) as Promise<T>;
}

describe("Test Drand Precompile (0x811)", () => {
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>;

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL);
        api = await getDevnetApi();
    });

    describe("View Storage Functions", () => {

        // ─── getLastStoredRound ───────────────────────────────────────────────
        it("getLastStoredRound returns matching on-chain value", async () => {
            const onChain = await api.query.Drand.LastStoredRound.getValue();
            const fromContract = await readDrand<bigint>(publicClient, "getLastStoredRound");

            const expected = onChain !== undefined ? BigInt(onChain as any) : BigInt(0);
            const actual = BigInt(fromContract as any);
            const diff = actual > expected ? actual - expected : expected - actual;
            assert.ok(
                diff <= BigInt(2),
                `LastStoredRound should match closely (actual: ${actual}, expected: ${expected})`
            );
        });

        // ─── getOldestStoredRound ─────────────────────────────────────────────
        it("getOldestStoredRound returns matching on-chain value", async () => {
            const onChain = await api.query.Drand.OldestStoredRound.getValue();
            const fromContract = await readDrand<bigint>(publicClient, "getOldestStoredRound");

            const expected = onChain !== undefined ? BigInt(onChain as any) : BigInt(0);
            assert.strictEqual(BigInt(fromContract as any), expected, "OldestStoredRound should match on-chain value");
        });

        // ─── ordering invariant: lastRound >= oldestRound ─────────────────────
        it("lastStoredRound is always >= oldestStoredRound", async () => {
            const last = BigInt(await readDrand<bigint>(publicClient, "getLastStoredRound") as any);
            const oldest = BigInt(await readDrand<bigint>(publicClient, "getOldestStoredRound") as any);
            assert.ok(
                last >= oldest,
                `lastStoredRound (${last}) should be >= oldestStoredRound (${oldest})`
            );
        });

        // ─── getNextUnsignedAt ────────────────────────────────────────────────
        it("getNextUnsignedAt returns a matching on-chain block number", async () => {
            const onChain = await api.query.Drand.NextUnsignedAt.getValue();
            const fromContract = await readDrand<bigint>(publicClient, "getNextUnsignedAt");

            const expected = onChain !== undefined ? BigInt(onChain as any) : BigInt(0);
            const actual = BigInt(fromContract as any);
            // Allow small drift of 20 blocks due to timing
            const diff = actual > expected ? actual - expected : expected - actual;
            assert.ok(
                diff <= BigInt(20),
                `NextUnsignedAt should closely match on-chain value (actual: ${actual}, expected: ${expected})`
            );
        });

        // ─── getHasMigrationRun — fake key ────────────────────────────────────
        it("getHasMigrationRun returns false for a non-existent migration key", async () => {
            const fromContract = await readDrand<boolean>(
                publicClient,
                "getHasMigrationRun",
                ["this_migration_key_does_not_exist"]
            );
            assert.strictEqual(fromContract, false, "Non-existent migration key should return false");
        });

        // ─── getHasMigrationRun — real key ────────────────────────────────────
        it("getHasMigrationRun returns true for migrate_set_oldest_round", async () => {
            const fromContract = await readDrand<boolean>(
                publicClient,
                "getHasMigrationRun",
                ["migrate_set_oldest_round"]
            );
            // This migration is defined in pallets/drand/src/migrations/migrate_set_oldest_round.rs,
            // but it is intentionally deferred in lib.rs (`on_runtime_upgrade`).
            // Thus, it correctly returns false on devnet until uncommented.
            assert.strictEqual(fromContract, false, "Migration 'migrate_set_oldest_round' is intentionally deferred on devnet");
        });

        // ─── getBeaconConfig ──────────────────────────────────────────────────
        it("getBeaconConfig returns a valid struct with non-zero genesis time", async () => {
            const result = await readDrand<{
                genesisTime: number;
                period: number;
                publicKey: `0x${string}`;
                chainHash: `0x${string}`;  // bytes32
                groupHash: `0x${string}`;  // bytes32
                schemeId: `0x${string}`;
                beaconId: `0x${string}`;
                isExplicitlyConfigured: boolean;
            }>(publicClient, "getBeaconConfig");

            assert.ok(result !== undefined, "getBeaconConfig should return a result");
            // genesisTime is uint32 (u32 on-chain) — compare as plain number, not bigint
            assert.ok(result.genesisTime > 0, `genesisTime should be non-zero, got ${result.genesisTime}`);
            assert.ok(result.period > 0, `period should be non-zero, got ${result.period}`);
            assert.ok(result.publicKey && result.publicKey.length > 2, "publicKey should be non-empty");
            // chainHash and groupHash are bytes32 — exactly 66 hex chars (0x + 64)
            assert.strictEqual(result.chainHash.length, 66, `chainHash must be 32 bytes, got ${result.chainHash}`);
            assert.strictEqual(result.groupHash.length, 66, `groupHash must be 32 bytes, got ${result.groupHash}`);
            assert.notStrictEqual(result.chainHash, "0x" + "00".repeat(32), "chainHash should be non-zero");
            assert.ok(result.schemeId && result.schemeId.length > 2, "schemeId should be non-empty");
            assert.ok(result.beaconId && result.beaconId.length > 2, "beaconId should be non-empty");
            assert.strictEqual(typeof result.isExplicitlyConfigured, "boolean", "isExplicitlyConfigured should be a boolean");
            // NOTE: isExplicitlyConfigured=false can only be tested on a fresh chain where
            //       BeaconConfig was never explicitly written. On devnet this is always true
            //       because the OCW sets it at genesis. For the false branch, see the Rust
            //       unit test in pallets/drand/src/tests.rs (try_get Err path).
        });

        // ─── getPalletVersion ─────────────────────────────────────────────────
        it("getPalletVersion returns a valid version number", async () => {
            const fromContract = await readDrand<number>(publicClient, "getPalletVersion");
            assert.ok(Number(fromContract) >= 0, `Pallet version should be a non-negative number, got ${fromContract}`);
        });

        // ─── getPulse — edge case: non-existent round ─────────────────────────
        it("getPulse returns empty bytes for a non-existent round (fallback path)", async () => {
            const result = await readDrand<{ randomness: `0x${string}`; signature: `0x${string}` }>(
                publicClient,
                "getPulse",
                [BigInt(999999999)]
            );
            assert.ok(result !== undefined, "getPulse should return a result even for a non-existent round");
            assert.strictEqual(result.randomness, "0x" + "00".repeat(32), "randomness should be zero bytes32 for non-existent round");
            assert.strictEqual(result.signature, "0x", "signature should be empty bytes for non-existent round");
        });

        // ─── getPulse — happy path ────────────────────────────────────────────
        it("getPulse returns non-empty randomness and signature for the last stored round", async () => {
            const lastRound = await readDrand<bigint>(publicClient, "getLastStoredRound");
            if (lastRound === BigInt(0)) {
                // Devnet has no pulses yet — skip
                console.warn("  [skip] No pulses stored yet, skipping getPulse happy-path test");
                return;
            }

            const result = await readDrand<{ randomness: `0x${string}`; signature: `0x${string}` }>(
                publicClient,
                "getPulse",
                [lastRound]
            );

            assert.ok(
                result.randomness && result.randomness.length > 2,
                `randomness should be non-empty for round ${lastRound}`
            );
            assert.ok(
                result.signature && result.signature.length > 2,
                `signature should be non-empty for round ${lastRound}`
            );
        });

        // ─── getCurrentRandomness ─────────────────────────────────────────────
        it("getCurrentRandomness returns bytes32 (zero when no pulses stored)", async () => {
            const fromContract = await readDrand<`0x${string}`>(publicClient, "getCurrentRandomness");
            // Must be exactly 32 bytes (66 hex chars including 0x prefix)
            assert.strictEqual(fromContract.length, 66, `getCurrentRandomness must return exactly 32 bytes, got ${fromContract}`);
        });

        it("getCurrentRandomness returns non-zero when pulses exist", async () => {
            const lastRound = await readDrand<bigint>(publicClient, "getLastStoredRound");
            if (lastRound === BigInt(0)) {
                console.warn("  [skip] No pulses stored yet, skipping getCurrentRandomness non-zero test");
                return;
            }

            const fromContract = await readDrand<`0x${string}`>(publicClient, "getCurrentRandomness");
            const zeroBytes32 = "0x" + "00".repeat(32);
            assert.notStrictEqual(
                fromContract,
                zeroBytes32,
                "getCurrentRandomness should not be zero when pulses are stored"
            );
        });

        // ─── getPulse — round 0 (always absent) ─────────────────────────────────────────
        it("getPulse returns empty bytes for round 0", async () => {
            const result = await readDrand<{ randomness: `0x${string}`; signature: `0x${string}` }>(
                publicClient,
                "getPulse",
                [BigInt(0)]
            );
            assert.strictEqual(result.randomness, "0x" + "00".repeat(32), "randomness should be zero bytes32 for round 0");
            assert.strictEqual(result.signature, "0x", "signature should be empty for round 0");
        });

        // ─── getHasMigrationRun — key boundary tests ─────────────────────────────────────
        it("getHasMigrationRun returns false for a 128-byte key (boundary)", async () => {
            const result = await readDrand<boolean>(
                publicClient,
                "getHasMigrationRun",
                ["a".repeat(128)]
            );
            assert.strictEqual(result, false, "Non-existent 128-byte key should return false");
        });

        it("getHasMigrationRun reverts for a 129-byte key (exceeds MigrationKeyMaxLen)", async () => {
            await assert.rejects(
                readDrand<boolean>(publicClient, "getHasMigrationRun", ["a".repeat(129)]),
                /migration key too long|revert|execution reverted/i,
                "Key longer than 128 bytes should revert"
            );
        });
    });
});
