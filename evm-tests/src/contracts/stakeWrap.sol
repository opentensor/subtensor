// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0 <0.8.2;

address constant ISTAKING_ADDRESS = 0x0000000000000000000000000000000000000805;

interface Staking {
    function addStakeLimit(
        bytes32 hotkey,
        uint256 amount,
        uint256 limit_price,
        bool allow_partial,
        uint256 netuid
    ) external;

    function addStake(bytes32 hotkey, uint256 amount, uint256 netuid) external;
}

contract StakeWrap {
    constructor() {}
    receive() external payable {}

    function stake(bytes32 hotkey, uint256 netuid, uint256 amount) external {
        bytes memory data = abi.encodeWithSelector(
            Staking.addStake.selector,
            hotkey,
            amount,
            netuid
        );
        (bool success, ) = ISTAKING_ADDRESS.call{gas: gasleft()}(data);
        require(success, "addStake call failed");
    }

    function stakeLimit(
        bytes32 hotkey,
        uint256 netuid,
        uint256 limitPrice,
        uint256 amount,
        bool allowPartial
    ) external {
        bytes memory data = abi.encodeWithSelector(
            Staking.addStakeLimit.selector,
            hotkey,
            amount,
            limitPrice,
            allowPartial,
            netuid
        );
        (bool success, ) = ISTAKING_ADDRESS.call{gas: gasleft()}(data);
        require(success, "addStakeLimit call failed");
    }
}
