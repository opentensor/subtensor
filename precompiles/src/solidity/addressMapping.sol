// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

address constant IADDRESS_MAPPING_ADDRESS = 0x000000000000000000000000000000000000080C;

interface IAddressMapping {
    /**
     * @dev Converts an Ethereum address (H160) to a Substrate AccountId32 (H256).
     *
     * This function uses the AddressMapping configured in the runtime to convert
     * an Ethereum address to its corresponding Substrate account ID. The mapping
     * is implemented using HashedAddressMapping with Blake2b hashing as configured
     * in the runtime.
     *
     * @param target_address The Ethereum address (20 bytes) to convert.
     * @return The corresponding Substrate AccountId32 (32 bytes).
     */
    function addressMapping(
        address target_address
    ) external payable returns (bytes32);
}
