import * as assert from "assert";
import { ethers } from "ethers";
import { generateRandomEthersWallet } from "../src/utils";
import { IADDRESS_MAPPING_ADDRESS, IAddressMappingABI } from "../src/contracts/addressMapping";
import { convertH160ToPublicKey } from "../src/address-utils";
import { u8aToHex } from "@polkadot/util";

describe("Test address mapping precompile", () => {
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();

    it("Address mapping converts H160 to AccountId32 correctly", async () => {
        const contract = new ethers.Contract(
            IADDRESS_MAPPING_ADDRESS,
            IAddressMappingABI,
            wallet1
        );

        // Test with wallet1's address
        const evmAddress = wallet1.address;
        const accountId32 = await contract.addressMapping(evmAddress);
        const expectedAcccountId32 = convertH160ToPublicKey(evmAddress);

        // Verify the result is a valid bytes32 (32 bytes)
        assert.ok(accountId32.length === 66, "AccountId32 should be 32 bytes (66 hex chars with 0x)");
        assert.ok(accountId32.startsWith("0x"), "AccountId32 should start with 0x");

        // Verify it's not all zeros
        assert.notEqual(
            accountId32,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "AccountId32 should not be all zeros"
        );

        console.log("accountId32: {}", accountId32);
        console.log("expectedAcccountId32: {}", expectedAcccountId32);

        assert.equal(accountId32, u8aToHex(expectedAcccountId32), "AccountId32 should be the same as the expected AccountId32");
    });

    it("Address mapping works with different addresses", async () => {
        const contract = new ethers.Contract(
            IADDRESS_MAPPING_ADDRESS,
            IAddressMappingABI,
            wallet1
        );

        // Test with wallet2's address
        const evmAddress1 = wallet1.address;
        const evmAddress2 = wallet2.address;

        const accountId1 = await contract.addressMapping(evmAddress1);
        const accountId2 = await contract.addressMapping(evmAddress2);

        // Different addresses should map to different AccountIds
        assert.notEqual(
            accountId1,
            accountId2,
            "Different EVM addresses should map to different AccountIds"
        );

        // Both should be valid bytes32
        assert.ok(accountId1.length === 66, "AccountId1 should be 32 bytes");
        assert.ok(accountId2.length === 66, "AccountId2 should be 32 bytes");
    });

    it("Address mapping is deterministic", async () => {
        const contract = new ethers.Contract(
            IADDRESS_MAPPING_ADDRESS,
            IAddressMappingABI,
            wallet1
        );

        const evmAddress = wallet1.address;

        // Call multiple times with the same address
        const accountId1 = await contract.addressMapping(evmAddress);
        const accountId2 = await contract.addressMapping(evmAddress);

        // All calls should return the same result
        assert.equal(
            accountId1,
            accountId2,
            "First and second calls should return the same AccountId"
        );
    });
});
