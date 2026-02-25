pragma solidity ^0.8.0;

address constant IDEPRECATION_REGISTRY_ADDRESS = 0x0000000000000000000000000000000000000810;

/**
 * @title IDeprecationRegistry
 * @dev Precompile at 0x810 that exposes deprecation metadata for precompile functions.
 *      Allows contracts and tooling to discover whether a function at a given precompile
 *      address is deprecated and where to find its replacement.
 */
interface IDeprecationRegistry {
    struct DeprecationInfo {
        bool isDeprecated;
        address newPrecompile;
        bytes32 newSelector;  // 4-byte selector left-aligned in bytes32
        bytes message;        // UTF-8 encoded migration guidance
    }

    /**
     * @dev Returns deprecation info for a specific precompile function.
     * @param precompile The precompile contract address.
     * @param selector The 4-byte function selector left-aligned in bytes32.
     *                 For example, for selector 0xabcd1234, pass 0xabcd1234000...000.
     * @return info The deprecation info. If not deprecated, isDeprecated is false
     *         and all other fields are zero/empty.
     */
    function getDeprecationInfo(
        address precompile,
        bytes32 selector
    ) external view returns (DeprecationInfo memory info);
}
