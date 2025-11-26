export declare const root: {
    blockExplorers: {
        readonly default: {
            readonly name: "Rootscan";
            readonly url: "https://rootscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xc9C2E2429AeC354916c476B30d729deDdC94988d";
            readonly blockCreated: 9218338;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 7668;
    name: "The Root Network";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "XRP";
        readonly symbol: "XRP";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://root.rootnet.live/archive"];
            readonly webSocket: readonly ["wss://root.rootnet.live/archive/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=root.d.ts.map