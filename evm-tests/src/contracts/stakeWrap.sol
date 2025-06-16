// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.2 <0.9.0;

address constant ISTAKING_ADDRESS = 0x0000000000000000000000000000000000000805;

interface Staking {
    function addStakeLimit(
        bytes32 hotkey,
        uint256 amount,
        uint256 limit_price,
        bool allow_partial,
        uint256 netuid
    ) public virtual;

    function addStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) public virtual;
}

contract StakeWrap {
    address ISUBNET_ADDRESS = 0x0000000000000000000000000000000000000805;
    constructor() {}
    receive() external payable {}
    function stake(bytes32 hotkey, uint256 netuid, uint256 amount) external {
        address precompile = address(
            0x0000000000000000000000000000000000000805
        );
        Staking(a).addStake(hotkey, amount, netuid);
    }

    function stakeLimit(
        bytes32 hotkey,
        uint256 netuid,
        uint256 limitPrice,
        uint256 amount,
        bool allowPartial
    ) external {
        address precompile = address(
            0x0000000000000000000000000000000000000805
        );
        (bool success, ) = ISUBNET_ADDRESS.call{gas: gasleft()}(
            abi.encodeWithSelector(
                subnet.setMinDifficulty.selector,
                netuid,
                minDifficulty
            )
        );
        require(success, "setMinDifficulty call failed");
    }
}
