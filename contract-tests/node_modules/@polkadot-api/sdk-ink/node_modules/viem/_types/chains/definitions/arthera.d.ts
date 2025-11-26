export declare const arthera: {
    blockExplorers: {
        readonly default: {
            readonly name: "Arthera EVM Explorer";
            readonly url: "https://explorer.arthera.net";
            readonly apiUrl: "https://explorer.arthera.net/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 4502791;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 10242;
    name: "Arthera";
    nativeCurrency: {
        readonly name: "Arthera";
        readonly symbol: "AA";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.arthera.net"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=arthera.d.ts.map