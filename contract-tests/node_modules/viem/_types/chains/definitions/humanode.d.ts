export declare const humanode: {
    blockExplorers: {
        readonly default: {
            readonly name: "Subscan";
            readonly url: "https://humanode.subscan.io";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 4413097;
        };
    };
    id: 5234;
    name: "Humanode";
    nativeCurrency: {
        readonly name: "HMND";
        readonly symbol: "HMND";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://explorer-rpc-http.mainnet.stages.humanode.io"];
            readonly webSocket: readonly ["wss://explorer-rpc-ws.mainnet.stages.humanode.io"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=humanode.d.ts.map