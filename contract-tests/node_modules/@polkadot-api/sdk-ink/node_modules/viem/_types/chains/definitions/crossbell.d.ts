export declare const crossbell: {
    blockExplorers: {
        readonly default: {
            readonly name: "CrossScan";
            readonly url: "https://scan.crossbell.io";
            readonly apiUrl: "https://scan.crossbell.io/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 38246031;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 3737;
    name: "Crossbell";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "CSB";
        readonly symbol: "CSB";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.crossbell.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=crossbell.d.ts.map