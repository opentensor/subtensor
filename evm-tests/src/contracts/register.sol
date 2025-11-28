// SPDX-License-Identifier: MIT

pragma solidity ^0.8.30;

address constant NEURON_PRECOMPILE = 0x0000000000000000000000000000000000000804;

contract Register {
    error InsufficientValue();
    error RefundFailed();

    function burnedRegisterNeuron(
        uint16 netuid,
        bytes32 hotkey
    ) external payable returns (bool) {
        if (msg.value == 0) revert InsufficientValue();

        bytes memory data = abi.encodeWithSelector(
            bytes4(keccak256("burnedRegister(uint16,bytes32)")),
            netuid,
            hotkey
        );

        (bool success, ) = NEURON_PRECOMPILE.call{value: 0, gas: gasleft()}(
            data
        );

        if (!success) {
            revert InsufficientValue();
        }

        // Refund leftover balance
        uint256 leftover = address(this).balance;
        if (leftover > 0) _refund(leftover);

        return true;
    }

    function _refund(uint256 amount) internal {
        (bool sent, ) = payable(msg.sender).call{value: amount}("");
        if (!sent) revert RefundFailed();
    }
}
