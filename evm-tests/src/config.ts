export const ETH_LOCAL_URL = 'http://localhost:9944'
export const SUB_LOCAL_URL = 'ws://localhost:9944'

export const IED25519VERIFY_ADDRESS = "0x0000000000000000000000000000000000000402";
export const IEd25519VerifyABI = [
    {
        inputs: [
            { internalType: "bytes32", name: "message", type: "bytes32" },
            { internalType: "bytes32", name: "publicKey", type: "bytes32" },
            { internalType: "bytes32", name: "r", type: "bytes32" },
            { internalType: "bytes32", name: "s", type: "bytes32" },
        ],
        name: "verify",
        outputs: [{ internalType: "bool", name: "", type: "bool" }],
        stateMutability: "pure",
        type: "function",
    },
];

export const IBALANCETRANSFER_ADDRESS = "0x0000000000000000000000000000000000000800";
export const IBalanceTransferABI = [
    {
        inputs: [
            {
                internalType: "bytes32",
                name: "data",
                type: "bytes32",
            },
        ],
        name: "transfer",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
];