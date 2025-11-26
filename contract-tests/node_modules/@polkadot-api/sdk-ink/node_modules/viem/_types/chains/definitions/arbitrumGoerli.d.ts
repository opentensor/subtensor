export declare const arbitrumGoerli: {
    blockExplorers: {
        readonly default: {
            readonly name: "Arbiscan";
            readonly url: "https://goerli.arbiscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 88114;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 421613;
    name: "Arbitrum Goerli";
    nativeCurrency: {
        readonly name: "Arbitrum Goerli Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://goerli-rollup.arbitrum.io/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=arbitrumGoerli.d.ts.map