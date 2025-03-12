export const INEURON_ADDRESS = "0x0000000000000000000000000000000000000804";

export const INeuronABI = [
    {
        inputs: [
            {
                internalType: "uint16",
                name: "netuid",
                type: "uint16",
            },
            {
                internalType: "bytes32",
                name: "commitHash",
                type: "bytes32",
            },
        ],
        name: "commitWeights",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "uint16",
                name: "netuid",
                type: "uint16",
            },
            {
                internalType: "uint16[]",
                name: "uids",
                type: "uint16[]",
            },
            {
                internalType: "uint16[]",
                name: "values",
                type: "uint16[]",
            },
            {
                internalType: "uint16[]",
                name: "salt",
                type: "uint16[]",
            },
            {
                internalType: "uint64",
                name: "versionKey",
                type: "uint64",
            },
        ],
        name: "revealWeights",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "uint16",
                name: "netuid",
                type: "uint16",
            },
            {
                internalType: "uint16[]",
                name: "dests",
                type: "uint16[]",
            },
            {
                internalType: "uint16[]",
                name: "weights",
                type: "uint16[]",
            },
            {
                internalType: "uint64",
                name: "versionKey",
                type: "uint64",
            },
        ],
        name: "setWeights",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "uint16",
                name: "netuid",
                type: "uint16",
            },
            {
                internalType: "uint32",
                name: "version",
                type: "uint32",
            },
            {
                internalType: "uint128",
                name: "ip",
                type: "uint128",
            },
            {
                internalType: "uint16",
                name: "port",
                type: "uint16",
            },
            {
                internalType: "uint8",
                name: "ipType",
                type: "uint8",
            },
            {
                internalType: "uint8",
                name: "protocol",
                type: "uint8",
            },
            {
                internalType: "uint8",
                name: "placeholder1",
                type: "uint8",
            },
            {
                internalType: "uint8",
                name: "placeholder2",
                type: "uint8",
            },
        ],
        name: "serveAxon",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "uint16",
                name: "netuid",
                type: "uint16",
            },
            {
                internalType: "uint32",
                name: "version",
                type: "uint32",
            },
            {
                internalType: "uint128",
                name: "ip",
                type: "uint128",
            },
            {
                internalType: "uint16",
                name: "port",
                type: "uint16",
            },
            {
                internalType: "uint8",
                name: "ipType",
                type: "uint8",
            },
            {
                internalType: "uint8",
                name: "protocol",
                type: "uint8",
            },
            {
                internalType: "uint8",
                name: "placeholder1",
                type: "uint8",
            },
            {
                internalType: "uint8",
                name: "placeholder2",
                type: "uint8",
            },
            {
                internalType: "bytes",
                name: "certificate",
                type: "bytes",
            },
        ],
        name: "serveAxonTls",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "uint16",
                name: "netuid",
                type: "uint16",
            },
            {
                internalType: "uint32",
                name: "version",
                type: "uint32",
            },
            {
                internalType: "uint128",
                name: "ip",
                type: "uint128",
            },
            {
                internalType: "uint16",
                name: "port",
                type: "uint16",
            },
            {
                internalType: "uint8",
                name: "ipType",
                type: "uint8",
            },
        ],
        name: "servePrometheus",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
    {
        inputs: [
            {
                internalType: "uint16",
                name: "netuid",
                type: "uint16",
            },
            {
                internalType: "bytes32",
                name: "hotkey",
                type: "bytes32",
            },
        ],
        name: "burnedRegister",
        outputs: [],
        stateMutability: "payable",
        type: "function",
    },
];