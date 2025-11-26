export declare const metachainIstanbul: {
    blockExplorers: {
        readonly default: {
            readonly name: "MetaExplorer";
            readonly url: "https://istanbul-explorer.metachain.dev";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x0000000000000000000000000000000000003001";
            readonly blockCreated: 0;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1453;
    name: "MetaChain Istanbul";
    nativeCurrency: {
        readonly name: "Metatime Coin";
        readonly symbol: "MTC";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://istanbul-rpc.metachain.dev"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=metachainIstanbul.d.ts.map