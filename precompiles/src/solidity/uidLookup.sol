pragma solidity ^0.8.0;

address constant IUID_LOOKUP_ADDRESS = 0x0000000000000000000000000000000000000806;

struct LookupItem {
    uint16 uid;
    uint64 block_associated;
}

interface IUidLookup {
    function uidLookup(
        uint16 netuid,
        address evm_address,
        uint16 limit
    ) external view returns (LookupItem[] memory);
}
