export const IPROXY_ADDRESS = "0x000000000000000000000000000000000000080b";

export const IProxyABI = [
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
                "name": "proxy",
                "type": "bytes32"
            }
        ],
        "name": "killPureProxy",
        "outputs": [],
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
        "name": "proxyCall",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
];
