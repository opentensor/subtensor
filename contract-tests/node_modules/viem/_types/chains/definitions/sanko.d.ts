export declare const sanko: {
    blockExplorers: {
        readonly default: {
            readonly name: "Sanko Explorer";
            readonly url: "https://explorer.sanko.xyz";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 37;
        };
    };
    id: 1996;
    name: "Sanko";
    nativeCurrency: {
        readonly name: "DMT";
        readonly symbol: "DMT";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.sanko.xyz"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=sanko.d.ts.map