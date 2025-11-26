export declare const mantle: {
    blockExplorers: {
        readonly default: {
            readonly name: "Mantle Explorer";
            readonly url: "https://mantlescan.xyz/";
            readonly apiUrl: "https://api.mantlescan.xyz/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 304717;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 5000;
    name: "Mantle";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "MNT";
        readonly symbol: "MNT";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.mantle.xyz"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=mantle.d.ts.map