export declare const goat: {
    blockExplorers: {
        readonly default: {
            readonly name: "Goat Explorer";
            readonly url: "https://explorer.goat.network";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 0;
        };
    };
    id: 2345;
    name: "GOAT";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Bitcoin";
        readonly symbol: "BTC";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.goat.network"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=goat.d.ts.map