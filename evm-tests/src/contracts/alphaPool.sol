// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.2 <0.9.0;

interface IStaking {
    function transferStake(bytes32 coldkey, bytes32 hotkey, uint256 netuid1, uint256 netuid2, uint256 amount) external;
    function moveStake(bytes32 hotkey1, bytes32 hotkey2, uint256 netuid1, uint256 netuid2, uint256 amount) external;
}

contract AlphaPool {
  bytes32 public contract_coldkey;
  bytes32 public contract_hotkey;
  address public constant ISTAKING_V2_ADDRESS = 0x0000000000000000000000000000000000000805;

    mapping (address => mapping(uint256 => uint256)) public alphaBalance;

    constructor(bytes32 _contract_hotkey) {
        contract_hotkey = _contract_hotkey;
    }

    function setContractColdkey(bytes32 _contract_coldkey) public {
        contract_coldkey = _contract_coldkey;
    }

   function depositAlpha(uint256 _netuid, uint256 _alphyAmount, bytes32 _hotkey) public {
        require(contract_hotkey != 0x00, "contract coldkey not set");

        bytes memory data = abi.encodeWithSelector(
            IStaking.transferStake.selector,
            contract_coldkey,
            _hotkey,
            _netuid,
            _netuid,
            _alphyAmount
        );
        (bool success, ) = address(ISTAKING_V2_ADDRESS).delegatecall{gas: gasleft()}(data);
        require(success, "user deposit alpha call failed");

        if (_hotkey != contract_hotkey) {
            data = abi.encodeWithSelector(
                IStaking.moveStake.selector,
                _hotkey,
                contract_hotkey,
                _netuid,
                _netuid,
                _alphyAmount
            );
            (success, ) = address(ISTAKING_V2_ADDRESS).call{gas: gasleft()}(data);
            require(success, "user deposit, move stake call failed");
        }

        alphaBalance[msg.sender][_netuid] += _alphyAmount;
    }

    function withdrawAlpha(uint256 _netuid, uint256 _alphyAmount, bytes32 _user_coldkey) public {
        require(contract_hotkey != 0x00, "contract coldkey not set");
        require(alphaBalance[msg.sender][_netuid] >= _alphyAmount, "user withdraw, insufficient alpha balance");

        alphaBalance[msg.sender][_netuid] -= _alphyAmount;

        bytes memory data = abi.encodeWithSelector(
            IStaking.transferStake.selector,
            _user_coldkey,
            contract_hotkey,
            _netuid,
            _netuid,
            _alphyAmount
        );
        (bool success, ) = address(ISTAKING_V2_ADDRESS).call{gas: gasleft()}(data);
        require(success, "user withdraw alpha call failed");
    }
}
