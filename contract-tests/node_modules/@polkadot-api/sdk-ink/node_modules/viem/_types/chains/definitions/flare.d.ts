export declare const flare: {
    blockExplorers: {
        readonly default: {
            readonly name: "Flare Explorer";
            readonly url: "https://flare-explorer.flare.network";
            readonly apiUrl: "https://flare-explorer.flare.network/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 3002461;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 14;
    name: "Flare Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Flare";
        readonly symbol: "FLR";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://flare-api.flare.network/ext/C/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=flare.d.ts.map