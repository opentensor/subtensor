export const IPURE_PROXY_ADDRESS = "0x000000000000000000000000000000000000080a";

export const IPureProxyABI = [
    {
        "inputs": [],
        "name": "createPureProxy",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "getPureProxy",
        "outputs": [
            {
                "internalType": "bytes32[]",
                "name": "",
                "type": "bytes32[]"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "real",
                "type": "bytes32"
            },
            {
                "internalType": "uint8[]",
                "name": "call",
                "type": "uint8[]"
            }
        ],
        "name": "pureProxyCall",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
];