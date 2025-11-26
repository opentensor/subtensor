export declare const kroma: {
    blockExplorers: {
        readonly default: {
            readonly name: "Kroma Explorer";
            readonly url: "https://blockscout.kroma.network";
            readonly apiUrl: "https://blockscout.kroma.network/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 16054868;
        };
    };
    id: 255;
    name: "Kroma";
    nativeCurrency: {
        readonly name: "ETH";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://api.kroma.network"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=kroma.d.ts.map