pragma solidity ^0.8.0;

address constant ISUBTENSOR_BALANCE_TRANSFER_ADDRESS = 0x0000000000000000000000000000000000000800;

interface ISubtensorBalanceTransfer {
    function transfer(bytes32 data) external payable;

    /// @dev Returns the total issuance of the native token.
    function getTotalIssuance() external view returns (uint64);

    /// @dev Returns the free balance of an account.
    /// @param account The account ID (bytes32).
    function getFreeBalance(bytes32 account) external view returns (uint64);
}