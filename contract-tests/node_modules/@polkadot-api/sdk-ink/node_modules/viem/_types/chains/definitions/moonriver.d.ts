export declare const moonriver: {
    blockExplorers: {
        readonly default: {
            readonly name: "Moonscan";
            readonly url: "https://moonriver.moonscan.io";
            readonly apiUrl: "https://api-moonriver.moonscan.io/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 1597904;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1285;
    name: "Moonriver";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "MOVR";
        readonly symbol: "MOVR";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://moonriver.public.blastapi.io"];
            readonly webSocket: readonly ["wss://moonriver.public.blastapi.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=moonriver.d.ts.map