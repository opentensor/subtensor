export declare const chiliz: {
    blockExplorers: {
        readonly default: {
            readonly name: "Chiliz Explorer";
            readonly url: "https://scan.chiliz.com";
            readonly apiUrl: "https://scan.chiliz.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 8080847;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 88888;
    name: "Chiliz Chain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "CHZ";
        readonly symbol: "CHZ";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.chiliz.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "chiliz-chain";
};
//# sourceMappingURL=chiliz.d.ts.map