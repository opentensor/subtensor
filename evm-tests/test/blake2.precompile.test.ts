import { ETH_LOCAL_URL } from '../src/config'
import { getPublicClient } from "../src/utils";
import { toHex, toBytes, PublicClient } from 'viem'
import { blake2AsU8a } from "@polkadot/util-crypto";
import * as assert from "assert";

// Blake2 precompile addresses
const BLAKE2_128_ADDRESS = "0x000000000000000000000000000000000000000a";
const BLAKE2_256_ADDRESS = "0x000000000000000000000000000000000000000b";

// Simple ABI for calling the precompiles - they don't have a formal interface
// We'll use low-level calls to test them directly
describe("Blake2 Precompiles Test", () => {
    let ethClient: PublicClient;

    before(async () => {
        ethClient = await getPublicClient(ETH_LOCAL_URL);
    });

    describe("Blake2-128 Precompile", () => {
        it("should hash empty input correctly", async () => {
            const input = new Uint8Array(0);
            const inputHex = toHex(input);

            // Test Blake2-128 precompile
            const result = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-128 computation
            const expectedHash = blake2AsU8a(input, 128);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-128 Empty input result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-128 hash mismatch for empty input");
        });

        it("should hash short message correctly", async () => {
            const message = "Hello, Blake2!";
            const input = new TextEncoder().encode(message);
            const inputHex = toHex(input);

            // Test Blake2-128 precompile
            const result = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-128 computation
            const expectedHash = blake2AsU8a(input, 128);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-128 Short message result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-128 hash mismatch for short message");
        });

        it("should hash 32-byte input correctly", async () => {
            // Create a 32-byte input (typical hash size)
            const input = new Uint8Array(32);
            for (let i = 0; i < 32; i++) {
                input[i] = i;
            }
            const inputHex = toHex(input);

            // Test Blake2-128 precompile
            const result = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-128 computation
            const expectedHash = blake2AsU8a(input, 128);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-128 32-byte input result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-128 hash mismatch for 32-byte input");
        });

        it("should hash large input correctly", async () => {
            // Create a large input (1KB)
            const input = new Uint8Array(1024);
            for (let i = 0; i < 1024; i++) {
                input[i] = i % 256;
            }
            const inputHex = toHex(input);

            // Test Blake2-128 precompile
            const result = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-128 computation
            const expectedHash = blake2AsU8a(input, 128);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-128 Large input result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-128 hash mismatch for large input");
        });

        it("should produce different hashes for different inputs", async () => {
            const input1 = new TextEncoder().encode("input1");
            const input2 = new TextEncoder().encode("input2");

            const result1 = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: toHex(input1),
            });

            const result2 = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: toHex(input2),
            });

            console.log(`Blake2-128 Input1 hash: ${result1.data}`);
            console.log(`Blake2-128 Input2 hash: ${result2.data}`);

            assert.notEqual(result1.data, result2.data, "Blake2-128 should produce different hashes for different inputs");
        });
    });

    describe("Blake2-256 Precompile", () => {
        it("should hash empty input correctly", async () => {
            const input = new Uint8Array(0);
            const inputHex = toHex(input);

            // Test Blake2-256 precompile
            const result = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-256 computation
            const expectedHash = blake2AsU8a(input, 256);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-256 Empty input result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-256 hash mismatch for empty input");
        });

        it("should hash short message correctly", async () => {
            const message = "Hello, Blake2!";
            const input = new TextEncoder().encode(message);
            const inputHex = toHex(input);

            // Test Blake2-256 precompile
            const result = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-256 computation
            const expectedHash = blake2AsU8a(input, 256);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-256 Short message result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-256 hash mismatch for short message");
        });

        it("should hash 32-byte input correctly", async () => {
            // Create a 32-byte input
            const input = new Uint8Array(32);
            for (let i = 0; i < 32; i++) {
                input[i] = i;
            }
            const inputHex = toHex(input);

            // Test Blake2-256 precompile
            const result = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-256 computation
            const expectedHash = blake2AsU8a(input, 256);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-256 32-byte input result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-256 hash mismatch for 32-byte input");
        });

        it("should hash large input correctly", async () => {
            // Create a large input (1KB)
            const input = new Uint8Array(1024);
            for (let i = 0; i < 1024; i++) {
                input[i] = i % 256;
            }
            const inputHex = toHex(input);

            // Test Blake2-256 precompile
            const result = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain Blake2-256 computation
            const expectedHash = blake2AsU8a(input, 256);
            const expectedHex = toHex(expectedHash);

            console.log(`Blake2-256 Large input result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-256 hash mismatch for large input");
        });

        it("should produce different hashes for different inputs", async () => {
            const input1 = new TextEncoder().encode("input1");
            const input2 = new TextEncoder().encode("input2");

            const result1 = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: toHex(input1),
            });

            const result2 = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: toHex(input2),
            });

            console.log(`Blake2-256 Input1 hash: ${result1.data}`);
            console.log(`Blake2-256 Input2 hash: ${result2.data}`);

            assert.notEqual(result1.data, result2.data, "Blake2-256 should produce different hashes for different inputs");
        });
    });

    describe("Blake2 Comparison Tests", () => {
        it("should produce different hash sizes (128 vs 256 bits)", async () => {
            const message = "Test message for comparison";
            const input = new TextEncoder().encode(message);
            const inputHex = toHex(input);

            const result128 = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: inputHex,
            });

            const result256 = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            console.log(`Blake2-128 result: ${result128.data}`);
            console.log(`Blake2-256 result: ${result256.data}`);

            // Blake2-128 should return 16 bytes (32 hex chars + 0x = 34 chars)
            // Blake2-256 should return 32 bytes (64 hex chars + 0x = 66 chars)
            assert.equal(result128.data?.length, 34, "Blake2-128 should return 16 bytes (32 hex chars + 0x)");
            assert.equal(result256.data?.length, 66, "Blake2-256 should return 32 bytes (64 hex chars + 0x)");

            // The hashes should be different
            assert.notEqual(result128.data, result256.data, "Blake2-128 and Blake2-256 should produce different hashes");
        });

        it("should handle the same input consistently across multiple calls", async () => {
            const message = "Consistency test";
            const input = new TextEncoder().encode(message);
            const inputHex = toHex(input);

            // Test Blake2-128 consistency
            const result128_1 = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: inputHex,
            });

            const result128_2 = await ethClient.call({
                to: BLAKE2_128_ADDRESS,
                data: inputHex,
            });

            // Test Blake2-256 consistency
            const result256_1 = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            const result256_2 = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            console.log(`Blake2-128 call 1: ${result128_1.data}`);
            console.log(`Blake2-128 call 2: ${result128_2.data}`);
            console.log(`Blake2-256 call 1: ${result256_1.data}`);
            console.log(`Blake2-256 call 2: ${result256_2.data}`);

            assert.equal(result128_1.data, result128_2.data, "Blake2-128 should be consistent across calls");
            assert.equal(result256_1.data, result256_2.data, "Blake2-256 should be consistent across calls");
        });

        it("should test address-to-SS58 conversion pattern", async () => {
            // Test the pattern used in convertH160ToPublicKey
            const ethAddress = "0x1234567890123456789012345678901234567890";
            const prefix = "evm:";
            const prefixBytes = new TextEncoder().encode(prefix);
            const addressBytes = toBytes(ethAddress);

            // Combine prefix and address like in the address-utils.ts
            const combined = new Uint8Array(prefixBytes.length + addressBytes.length);
            combined.set(prefixBytes);
            combined.set(addressBytes, prefixBytes.length);

            const inputHex = toHex(combined);

            // Test with Blake2-256 precompile (this is what's used for SS58 conversion)
            const result = await ethClient.call({
                to: BLAKE2_256_ADDRESS,
                data: inputHex,
            });

            // Compare with off-chain computation using the same algorithm
            const expectedHash = blake2AsU8a(combined, 256);
            const expectedHex = toHex(expectedHash);

            console.log(`Address conversion Blake2-256 result: ${result.data}`);
            console.log(`Expected hash: ${expectedHex}`);

            assert.equal(result.data, expectedHex, "Blake2-256 precompile should match off-chain computation for address conversion");
        });
    });
});
