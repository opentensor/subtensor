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
        stateMutability: "nonpayable",
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
        stateMutability: "nonpayable",
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
        stateMutability: "nonpayable",
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
        "stateMutability": "nonpayable",
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
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "delegate",
                "type": "bytes32"
            }
        ],
        "name": "removeProxy",
        "outputs": [],
        "stateMutability": "nonpayable",
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
        "stateMutability": "nonpayable",
        "type": "function"
    }
];