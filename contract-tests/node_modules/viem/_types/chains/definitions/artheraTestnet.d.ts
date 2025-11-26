export declare const artheraTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Arthera EVM Explorer";
            readonly url: "https://explorer-test.arthera.net";
            readonly apiUrl: "https://explorer-test.arthera.net/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 22051;
        };
    };
    id: 10243;
    name: "Arthera Testnet";
    nativeCurrency: {
        readonly name: "Arthera";
        readonly symbol: "AA";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-test.arthera.net"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=artheraTestnet.d.ts.map