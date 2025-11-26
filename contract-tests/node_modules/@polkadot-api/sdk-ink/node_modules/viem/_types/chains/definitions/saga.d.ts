export declare const saga: {
    blockExplorers: {
        readonly default: {
            readonly name: "Saga Explorer";
            readonly url: "https://sagaevm.sagaexplorer.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x864DDc9B50B9A0dF676d826c9B9EDe9F8913a160";
            readonly blockCreated: 467530;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 5464;
    name: "Saga";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "gas";
        readonly symbol: "GAS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://sagaevm.jsonrpc.sagarpc.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "saga";
};
//# sourceMappingURL=saga.d.ts.map