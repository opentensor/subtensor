export declare const eon: {
    blockExplorers: {
        readonly default: {
            readonly name: "EON Explorer";
            readonly url: "https://eon-explorer.horizenlabs.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {};
    ensTlds?: readonly string[] | undefined;
    id: 7332;
    name: "Horizen EON";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "ZEN";
        readonly symbol: "ZEN";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://eon-rpc.horizenlabs.io/ethv1"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=eon.d.ts.map