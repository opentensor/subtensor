export declare const saga: {
    blockExplorers: {
        readonly default: {
            readonly name: "Saga Explorer";
            readonly url: "https://sagaevm-5464-1.sagaexplorer.io";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0x864DDc9B50B9A0dF676d826c9B9EDe9F8913a160";
            readonly blockCreated: 467530;
        };
    };
    id: 5464;
    name: "Saga";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "gas";
        readonly symbol: "GAS";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["http://sagaevm-5464-1.jsonrpc.sagarpc.io"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "saga";
};
//# sourceMappingURL=saga.d.ts.map