export declare const kaia: {
    blockExplorers: {
        readonly default: {
            readonly name: "KaiaScan";
            readonly url: "https://kaiascan.io";
            readonly apiUrl: "https://api-cypress.klaytnscope.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 96002415;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 8217;
    name: "Kaia";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Kaia";
        readonly symbol: "KAIA";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://public-en.node.kaia.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=kaia.d.ts.map