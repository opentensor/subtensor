export declare const berachainTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Berachain";
            readonly url: "https://artio.beratrail.io";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 866924;
        };
    };
    id: 80085;
    name: "Berachain Artio";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BERA Token";
        readonly symbol: "BERA";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://artio.rpc.berachain.com"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=berachainTestnet.d.ts.map