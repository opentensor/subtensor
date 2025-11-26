export declare const flame: {
    blockExplorers: {
        readonly default: {
            readonly name: "Flame Explorer";
            readonly url: "https://explorer.flame.astria.org";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 6829148;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 253368190;
    name: "Flame";
    nativeCurrency: {
        readonly symbol: "TIA";
        readonly name: "TIA";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.flame.astria.org"];
            readonly webSocket: readonly ["wss://ws.flame.astria.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "flame";
};
//# sourceMappingURL=flame.d.ts.map