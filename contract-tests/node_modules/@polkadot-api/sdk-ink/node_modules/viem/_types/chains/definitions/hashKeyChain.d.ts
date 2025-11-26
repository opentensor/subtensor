export declare const hashkey: {
    blockExplorers: {
        readonly default: {
            readonly name: "HashKey Chain Explorer";
            readonly url: "https://hashkey.blockscout.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 0;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 177;
    name: "HashKey Chain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "HashKey EcoPoints";
        readonly symbol: "HSK";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.hsk.xyz"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=hashKeyChain.d.ts.map