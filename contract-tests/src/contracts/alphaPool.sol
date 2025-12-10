// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.2 <0.9.0;

interface IStaking {
    function transferStake(
        bytes32 coldkey,
        bytes32 hotkey,
        uint256 netuid1,
        uint256 netuid2,
        uint256 amount
    ) external;
    function moveStake(
        bytes32 hotkey1,
        bytes32 hotkey2,
        uint256 netuid1,
        uint256 netuid2,
        uint256 amount
    ) external;
    function getStake(
        bytes32 hotkey,
        bytes32 coldkey,
        uint256 netuid
    ) external view returns (uint256);
}

contract AlphaPool {
    bytes32 public contract_coldkey;
    bytes32 public contract_hotkey;
    address public constant ISTAKING_V2_ADDRESS =
        0x0000000000000000000000000000000000000805;

    mapping(address => mapping(uint256 => uint256)) public alphaBalance;

    constructor(bytes32 _contract_hotkey) {
        contract_hotkey = _contract_hotkey;
    }

    function setContractColdkey(bytes32 _contract_coldkey) public {
        contract_coldkey = _contract_coldkey;
    }

    function getContractStake(uint256 netuid) public view returns (uint256) {
        return
            IStaking(ISTAKING_V2_ADDRESS).getStake(
                contract_hotkey,
                contract_coldkey,
                netuid
            );
    }

    function depositAlpha(
        uint256 _netuid,
        uint256 _alphaAmount,
        bytes32 _hotkey
    ) public {
        require(contract_coldkey != 0x00, "contract coldkey not set");
        uint256 contractStake = getContractStake(_netuid);

        bytes memory data = abi.encodeWithSelector(
            IStaking.transferStake.selector,
            contract_coldkey,
            _hotkey,
            _netuid,
            _netuid,
            _alphaAmount
        );
        (bool success, ) = address(ISTAKING_V2_ADDRESS).delegatecall{
            gas: gasleft()
        }(data);
        require(success, "user deposit alpha call failed");

        uint256 newContractStake = getContractStake(_netuid);

        require(
            newContractStake > contractStake,
            "contract stake decreased after deposit"
        );

        // use the increased stake as the actual alpha amount, for the swap fee in the move stake call
        // the contract will take it and get compensated by laster emission of alpha
        uint256 actualAlphaAmount = newContractStake - contractStake;

        if (_hotkey != contract_hotkey) {
            data = abi.encodeWithSelector(
                IStaking.moveStake.selector,
                _hotkey,
                contract_hotkey,
                _netuid,
                _netuid,
                actualAlphaAmount
            );
            (success, ) = address(ISTAKING_V2_ADDRESS).call{gas: gasleft()}(
                data
            );
            require(success, "user deposit, move stake call failed");
        }

        alphaBalance[msg.sender][_netuid] += actualAlphaAmount;
    }

    function withdrawAlpha(
        uint256 _netuid,
        uint256 _alphaAmount,
        bytes32 _user_coldkey
    ) public {
        require(contract_coldkey != 0x00, "contract coldkey not set");
        require(
            alphaBalance[msg.sender][_netuid] >= _alphaAmount,
            "user withdraw, insufficient alpha balance"
        );
        uint256 contractStake = getContractStake(_netuid);

        alphaBalance[msg.sender][_netuid] -= _alphaAmount;

        bytes memory data = abi.encodeWithSelector(
            IStaking.transferStake.selector,
            _user_coldkey,
            contract_hotkey,
            _netuid,
            _netuid,
            _alphaAmount
        );
        (bool success, ) = address(ISTAKING_V2_ADDRESS).call{gas: gasleft()}(
            data
        );

        uint256 newContractStake = getContractStake(_netuid);
        require(
            newContractStake < contractStake,
            "contract stake increased after withdraw"
        );
        require(success, "user withdraw alpha call failed");
    }
}
