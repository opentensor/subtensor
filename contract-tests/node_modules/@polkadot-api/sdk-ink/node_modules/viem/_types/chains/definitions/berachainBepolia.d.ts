export declare const berachainBepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "Berascan";
            readonly url: "https://bepolia.beratrail.io";
        };
    };
    blockTime: 2000;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 0;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 80069;
    name: "Berachain Bepolia";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BERA Token";
        readonly symbol: "BERA";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://bepolia.rpc.berachain.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=berachainBepolia.d.ts.map