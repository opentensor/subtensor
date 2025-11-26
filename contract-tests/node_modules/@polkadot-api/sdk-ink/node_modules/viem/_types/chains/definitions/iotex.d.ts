export declare const iotex: {
    blockExplorers: {
        readonly default: {
            readonly name: "IoTeXScan";
            readonly url: "https://iotexscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 22163670;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 4689;
    name: "IoTeX";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "IoTeX";
        readonly symbol: "IOTX";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://babel-api.mainnet.iotex.io"];
            readonly webSocket: readonly ["wss://babel-api.mainnet.iotex.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=iotex.d.ts.map