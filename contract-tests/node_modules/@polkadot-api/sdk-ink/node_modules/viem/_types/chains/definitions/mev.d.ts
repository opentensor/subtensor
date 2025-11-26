export declare const mev: {
    blockExplorers: {
        readonly default: {
            readonly name: "Explorer";
            readonly url: "https://www.meversescan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 86881340;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 7518;
    name: "MEVerse Chain Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "MEVerse";
        readonly symbol: "MEV";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.meversemainnet.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=mev.d.ts.map