// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

interface ISR25519Verify {
    function verify(
        bytes32 message,
        bytes32 publicKey,
        bytes32 r,
        bytes32 s
    ) external pure returns (bool);
}

interface IED25519Verify {
    function verify(
        bytes32 message,
        bytes32 publicKey,
        bytes32 r,
        bytes32 s
    ) external pure returns (bool);
}

contract PrecompileGas {
    address constant IED25519VERIFY_ADDRESS =
        0x0000000000000000000000000000000000000402;
    address constant ISR25519VERIFY_ADDRESS =
        0x0000000000000000000000000000000000000403;
    IED25519Verify constant ed25519 = IED25519Verify(IED25519VERIFY_ADDRESS);
    ISR25519Verify constant sr25519 = ISR25519Verify(ISR25519VERIFY_ADDRESS);

    event Log(string message);

    bytes32 message =
        0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
    bytes32 publicKey =
        0x0000000000000000000000000000000000000000000000000000000000000000;
    bytes32 r =
        0x0000000000000000000000000000000000000000000000000000000000000000;
    bytes32 s =
        0x0000000000000000000000000000000000000000000000000000000000000000;

    /**
     * @notice Call the precompile using hardcoded signature data
     * @param iterations Number of times to call the precompile
     */
    function callED25519(uint64 iterations) external {
        for (uint64 i = 0; i < iterations; i++) {
            ed25519.verify(message, publicKey, r, s);
        }
        emit Log("callED25519");
    }

    /**
     * @notice Call the precompile using hardcoded signature data
     * @param iterations Number of times to call the precompile
     */
    function callSR25519(uint64 iterations) external {
        for (uint64 i = 0; i < iterations; i++) {
            sr25519.verify(message, publicKey, r, s);
        }
        emit Log("callSR25519");
    }
}
