export const ISTAKING_ADDRESS = "0x0000000000000000000000000000000000000801";
export const ISTAKING_V2_ADDRESS = "0x0000000000000000000000000000000000000805";

export const IStakingABI = [
    {
        inputs: [
            {
                internalType: "bytes32",
                name: "delegate",
                type: "bytes32",
            },
        ],
        name: "addProxy",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "bytes32",
                name: "hotkey",
                type: "bytes32",
            },
            {
                internalType: "uint256",
                name: "netuid",
                type: "uint256",
            },
        ],
        name: "addStake",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "bytes32",
                name: "delegate",
                type: "bytes32",
            },
        ],
        name: "removeProxy",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "bytes32",
                name: "hotkey",
                type: "bytes32",
            },
            {
                internalType: "bytes32",
                name: "coldkey",
                type: "bytes32",
            },
            {
                internalType: "uint256",
                name: "netuid",
                type: "uint256",
            },
        ],
        name: "getStake",
        outputs: [
            {
                internalType: "uint256",
                name: "",
                type: "uint256",
            },
        ],
        stateMutability: "view",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "bytes32",
                name: "hotkey",
                type: "bytes32",
            },
            {
                internalType: "uint256",
                name: "amount",
                type: "uint256",
            },
            {
                internalType: "uint256",
                name: "netuid",
                type: "uint256",
            },
        ],
        name: "removeStake",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
];

export const IStakingV2ABI = [
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "delegate",
                "type": "bytes32"
            }
        ],
        "name": "addProxy",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "addStake",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "getAlphaStakedValidators",
        "outputs": [
            {
                "internalType": "uint256[]",
                "name": "",
                "type": "uint256[]"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "bytes32",
                "name": "coldkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "getStake",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "getTotalAlphaStaked",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "coldkey",
                "type": "bytes32"
            }
        ],
        "name": "getTotalColdkeyStake",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "coldkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "getTotalColdkeyStakeOnSubnet",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            }
        ],
        "name": "getTotalHotkeyStake",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "getNominatorMinRequiredStake",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "delegate",
                "type": "bytes32"
            }
        ],
        "name": "removeProxy",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "removeStake",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "limit_price",
                "type": "uint256"
            },
            {
                "internalType": "bool",
                "name": "allow_partial",
                "type": "bool"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "addStakeLimit",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "limit_price",
                "type": "uint256"
            },
            {
                "internalType": "bool",
                "name": "allow_partial",
                "type": "bool"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "removeStakeLimit",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            },
        ],
        "name": "removeStakeFull",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "limit_price",
                "type": "uint256"
            }
        ],
        "name": "removeStakeFullLimit",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "burnAlpha",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "spenderColdkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "absoluteAmount",
                "type": "uint256"
            }
        ],
        "name": "approve",
        "outputs": [],
        "stateMutability": "",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "sourceColdkey",
                "type": "bytes32"
            },
            {
                "internalType": "bytes32",
                "name": "spenderColdkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "allowance",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "spenderColdkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "increaseAmount",
                "type": "uint256"
            }
        ],
        "name": "increaseAllowance",
        "outputs": [],
        "stateMutability": "",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "spenderColdkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "decreaseAmount",
                "type": "uint256"
            }
        ],
        "name": "decreaseAllowance",
        "outputs": [],
        "stateMutability": "",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "source_coldkey",
                "type": "bytes32"
            },
            {
                "internalType": "bytes32",
                "name": "destination_coldkey",
                "type": "bytes32"
            },
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "origin_netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "destination_netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            }
        ],
        "name": "transferStakeFrom",
        "outputs": [],
        "stateMutability": "",
        "type": "function"
    }
];
