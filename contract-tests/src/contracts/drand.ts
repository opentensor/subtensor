export const IDRAND_ADDRESS = "0x0000000000000000000000000000000000000811";

export const IDrandABI = [
    {
        "type": "function",
        "name": "getLastStoredRound",
        "inputs": [],
        "outputs": [{ "name": "", "type": "uint64", "internalType": "uint64" }],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "getOldestStoredRound",
        "inputs": [],
        "outputs": [{ "name": "", "type": "uint64", "internalType": "uint64" }],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "getPulse",
        "inputs": [{ "name": "round", "type": "uint64", "internalType": "uint64" }],
        "outputs": [
            {
                "name": "",
                "type": "tuple",
                "internalType": "struct IDrand.DrandPulse",
                "components": [
                    { "name": "randomness", "type": "bytes32", "internalType": "bytes32" },
                    { "name": "signature", "type": "bytes", "internalType": "bytes" }
                ]
            }
        ],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "getCurrentRandomness",
        "inputs": [],
        "outputs": [{ "name": "", "type": "bytes32", "internalType": "bytes32" }],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "getBeaconConfig",
        "inputs": [],
        "outputs": [
            {
                "name": "",
                "type": "tuple",
                "internalType": "struct IDrand.BeaconConfig",
                "components": [
                    { "name": "genesisTime", "type": "uint32", "internalType": "uint32" },
                    { "name": "period", "type": "uint32", "internalType": "uint32" },
                    { "name": "publicKey", "type": "bytes", "internalType": "bytes" },
                    { "name": "chainHash", "type": "bytes32", "internalType": "bytes32" },
                    { "name": "groupHash", "type": "bytes32", "internalType": "bytes32" },
                    { "name": "schemeId", "type": "bytes", "internalType": "bytes" },
                    { "name": "beaconId", "type": "bytes", "internalType": "bytes" },
                    { "name": "isExplicitlyConfigured", "type": "bool", "internalType": "bool" }
                ]
            }
        ],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "getHasMigrationRun",
        "inputs": [{ "name": "migrationName", "type": "string", "internalType": "string" }],
        "outputs": [{ "name": "", "type": "bool", "internalType": "bool" }],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "getNextUnsignedAt",
        "inputs": [],
        "outputs": [{ "name": "", "type": "uint64", "internalType": "uint64" }],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "getPalletVersion",
        "inputs": [],
        "outputs": [{ "name": "", "type": "uint16", "internalType": "uint16" }],
        "stateMutability": "view"
    }
] as const;
