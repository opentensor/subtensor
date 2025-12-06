// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.7.0 <0.9.0;

contract Withdraw {
    constructor() {}

    function withdraw(uint256 value) public payable {
        payable(msg.sender).transfer(value);
    }

    receive() external payable {}
}
