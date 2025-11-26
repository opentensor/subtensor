export declare const songbird: {
    blockExplorers: {
        readonly default: {
            readonly name: "Songbird Explorer";
            readonly url: "https://songbird-explorer.flare.network";
            readonly apiUrl: "https://songbird-explorer.flare.network/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 13382504;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 19;
    name: "Songbird Canary-Network";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Songbird";
        readonly symbol: "SGB";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://songbird-api.flare.network/ext/C/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=songbird.d.ts.map