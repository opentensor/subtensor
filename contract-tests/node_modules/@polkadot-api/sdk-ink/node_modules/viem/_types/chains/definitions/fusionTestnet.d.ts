export declare const fusionTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "FSNscan";
            readonly url: "https://testnet.fsnscan.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 10428309;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 46688;
    name: "Fusion Testnet";
    nativeCurrency: {
        readonly name: "Fusion";
        readonly symbol: "FSN";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet.fusionnetwork.io"];
            readonly webSocket: readonly ["wss://testnet.fusionnetwork.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=fusionTestnet.d.ts.map