import { IED25519VERIFY_ADDRESS, IEd25519VerifyABI, ETH_LOCAL_URL } from '../src/config'
import { getPublicClient } from "../src/utils";
import { toHex, toBytes, keccak256, PublicClient } from 'viem'
import { Keyring } from "@polkadot/keyring";
import * as assert from "assert";

describe("Verfication of ed25519 signature", () => {
    // init eth part
    let ethClient: PublicClient;

    before(async () => {
        ethClient = await getPublicClient(ETH_LOCAL_URL);
    });

    it("Verification of ed25519 works", async () => {
        const keyring = new Keyring({ type: "ed25519" });
        const alice = keyring.addFromUri("//Alice");

        // Use this example: https://github.com/gztensor/evm-demo/blob/main/docs/ed25519verify-precompile.md
        // const keyring = new Keyring({ type: "ed25519" });
        // const myAccount = keyring.addFromUri("//Alice");

        //////////////////////////////////////////////////////////////////////
        // Generate a signature

        // Your message to sign
        const message = "Sign this message";
        const messageU8a = new TextEncoder().encode(message);
        const messageHex = toHex(messageU8a); // Convert message to hex string
        const messageHash = keccak256(messageHex); // Hash the message to fit into bytes32
        console.log(`messageHash = ${messageHash}`);
        const hashedMessageBytes = toBytes(messageHash);
        console.log(`hashedMessageBytes = ${hashedMessageBytes}`);

        // Sign the message
        const signature = await alice.sign(hashedMessageBytes);
        console.log(`Signature: ${toHex(signature)}`);

        // Verify the signature locally
        const isValid = alice.verify(
            hashedMessageBytes,
            signature,
            alice.publicKey
        );
        console.log(`Is the signature valid? ${isValid}`);

        //////////////////////////////////////////////////////////////////////
        // Verify the signature using the precompile contract

        const publicKeyBytes = toHex(alice.publicKey);
        console.log(`publicKeyBytes = ${publicKeyBytes}`);

        // Split signture into Commitment (R) and response (s)
        let r = signature.slice(0, 32); // Commitment, a.k.a. "r" - first 32 bytes
        let s = signature.slice(32, 64); // Response, a.k.a. "s" - second 32 bytes
        let rBytes = toHex(r);
        let sBytes = toHex(s);

        const isPrecompileValid = await ethClient.readContract({
            address: IED25519VERIFY_ADDRESS,
            abi: IEd25519VerifyABI,
            functionName: "verify",
            args: [messageHash,
                publicKeyBytes,
                rBytes,
                sBytes]

        });

        console.log(
            `Is the signature valid according to the smart contract? ${isPrecompileValid}`
        );
        assert.equal(isPrecompileValid, true)

        //////////////////////////////////////////////////////////////////////
        // Verify the signature for bad data using the precompile contract

        let brokenHashedMessageBytes = hashedMessageBytes;
        brokenHashedMessageBytes[0] = (brokenHashedMessageBytes[0] + 1) % 0xff;
        const brokenMessageHash = toHex(brokenHashedMessageBytes);
        console.log(`brokenMessageHash = ${brokenMessageHash}`);

        const isPrecompileValidBadData = await ethClient.readContract({
            address: IED25519VERIFY_ADDRESS,
            abi: IEd25519VerifyABI,
            functionName: "verify",
            args: [brokenMessageHash,
                publicKeyBytes,
                rBytes,
                sBytes]

        });

        console.log(
            `Is the signature valid according to the smart contract for broken data? ${isPrecompileValidBadData}`
        );
        assert.equal(isPrecompileValidBadData, false)

        //////////////////////////////////////////////////////////////////////
        // Verify the bad signature for good data using the precompile contract

        let brokenR = r;
        brokenR[0] = (brokenR[0] + 1) % 0xff;
        rBytes = toHex(r);
        const isPrecompileValidBadSignature = await ethClient.readContract({
            address: IED25519VERIFY_ADDRESS,
            abi: IEd25519VerifyABI,
            functionName: "verify",
            args: [messageHash,
                publicKeyBytes,
                rBytes,
                sBytes]

        });

        console.log(
            `Is the signature valid according to the smart contract for broken signature? ${isPrecompileValidBadSignature}`
        );
        assert.equal(isPrecompileValidBadSignature, false)

    });
});