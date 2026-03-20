// Drand precompile address: 0x80e = 2062
export const IDRAND_ADDRESS = "0x000000000000000000000000000000000000080e";

export const IDrandABI = [
    {
        inputs: [
            {
                internalType: "uint64",
                name: "round",
                type: "uint64",
            },
        ],
        name: "getRandomness",
        outputs: [
            {
                internalType: "bytes32",
                name: "",
                type: "bytes32",
            },
        ],
        stateMutability: "view",
        type: "function",
    },
    {
        inputs: [],
        name: "getLastStoredRound",
        outputs: [
            {
                internalType: "uint64",
                name: "",
                type: "uint64",
            },
        ],
        stateMutability: "view",
        type: "function",
    },
] as const;
