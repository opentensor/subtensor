export declare const graphite: {
    blockExplorers: {
        readonly default: {
            readonly name: "Graphite Spectre";
            readonly url: "https://main.atgraphite.com";
            readonly apiUrl: "https://api.main.atgraphite.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts?: {
        [x: string]: import("../../index.js").ChainContract | {
            [sourceId: number]: import("../../index.js").ChainContract | undefined;
        } | undefined;
        ensRegistry?: import("../../index.js").ChainContract | undefined;
        ensUniversalResolver?: import("../../index.js").ChainContract | undefined;
        multicall3?: import("../../index.js").ChainContract | undefined;
        erc6492Verifier?: import("../../index.js").ChainContract | undefined;
    } | undefined;
    ensTlds?: readonly string[] | undefined;
    id: 440017;
    name: "Graphite Network";
    nativeCurrency: {
        readonly name: "Graphite";
        readonly symbol: "@G";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://anon-entrypoint-1.atgraphite.com"];
            readonly webSocket: readonly ["wss://ws-anon-entrypoint-1.atgraphite.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=graphite.d.ts.map