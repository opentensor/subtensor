export declare const fluenceTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://blockscout.testnet.fluence.dev";
            readonly apiUrl: "https://blockscout.testnet.fluence.dev/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 96424;
        };
    };
    id: 52164803;
    name: "Fluence Testnet";
    nativeCurrency: {
        readonly name: "tFLT";
        readonly symbol: "tFLT";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.testnet.fluence.dev"];
            readonly webSocket: readonly ["wss://ws.testnet.fluence.dev"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=fluenceTestnet.d.ts.map