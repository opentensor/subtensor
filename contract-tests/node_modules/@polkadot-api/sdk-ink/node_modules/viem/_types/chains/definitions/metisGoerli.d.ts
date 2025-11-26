export declare const metisGoerli: {
    blockExplorers: {
        readonly default: {
            readonly name: "Metis Goerli Explorer";
            readonly url: "https://goerli.explorer.metisdevops.link";
            readonly apiUrl: "https://goerli.explorer.metisdevops.link/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 1006207;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 599;
    name: "Metis Goerli";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Metis Goerli";
        readonly symbol: "METIS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://goerli.gateway.metisdevops.link"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=metisGoerli.d.ts.map