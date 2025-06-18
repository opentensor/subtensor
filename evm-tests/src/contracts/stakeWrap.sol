// SPDX-License-Identifier: GPL-3.0
// need use the compiler version 0.8.20 for this contract, otherwise there is an issue
// opcode(94) swap5 not supported.
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
        // can't call precompile like this way, the call never go to runtime precompile
        //Staking(ISTAKING_ADDRESS).addStake(hotkey, amount, netuid);

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
        // can't call precompile like this way, the call never go to runtime precompile
        // Staking(ISTAKING_ADDRESS).addStakeLimit(
        //     hotkey,
        //     amount,
        //     limitPrice,
        //     allowPartial,
        //     netuid
        // );

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
