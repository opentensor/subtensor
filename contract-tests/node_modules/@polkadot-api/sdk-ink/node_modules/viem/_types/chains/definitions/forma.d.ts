export declare const forma: {
    blockExplorers: {
        readonly default: {
            readonly name: "Forma Explorer";
            readonly url: "https://explorer.forma.art";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xd53C6FFB123F7349A32980F87faeD8FfDc9ef079";
            readonly blockCreated: 252705;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 984122;
    name: "Forma";
    nativeCurrency: {
        readonly symbol: "TIA";
        readonly name: "TIA";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.forma.art"];
            readonly webSocket: readonly ["wss://ws.forma.art"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "forma";
};
//# sourceMappingURL=forma.d.ts.map