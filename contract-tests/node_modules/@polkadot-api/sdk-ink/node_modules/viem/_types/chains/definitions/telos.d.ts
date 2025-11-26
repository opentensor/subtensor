export declare const telos: {
    blockExplorers: {
        readonly default: {
            readonly name: "Teloscan";
            readonly url: "https://www.teloscan.io/";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 246530709;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 40;
    name: "Telos";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Telos";
        readonly symbol: "TLOS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.telos.net"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=telos.d.ts.map