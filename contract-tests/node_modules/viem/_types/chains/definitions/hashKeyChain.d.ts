export declare const hashkey: {
    blockExplorers: {
        readonly default: {
            readonly name: "HashKey Chain Explorer";
            readonly url: "https://hashkey.blockscout.com";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 0;
        };
    };
    id: 177;
    name: "HashKey Chain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "HashKey EcoPoints";
        readonly symbol: "HSK";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.hsk.xyz"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=hashKeyChain.d.ts.map