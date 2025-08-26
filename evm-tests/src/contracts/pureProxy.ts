export const IPURE_PROXY_ADDRESS = "0x000000000000000000000000000000000000080b";

export const IPureProxyABI = [
    {
        "inputs": [],
        "name": "createPureProxy",
        "outputs": [
            {
                "internalType": "bytes32",
                "name": "",
                "type": "bytes32"
            }
        ],
        "stateMutability": "nonpayable",
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