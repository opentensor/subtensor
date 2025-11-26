export declare const moonbeam: {
    blockExplorers: {
        readonly default: {
            readonly name: "Moonscan";
            readonly url: "https://moonscan.io";
            readonly apiUrl: "https://api-moonbeam.moonscan.io/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 609002;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1284;
    name: "Moonbeam";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "GLMR";
        readonly symbol: "GLMR";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://moonbeam.public.blastapi.io"];
            readonly webSocket: readonly ["wss://moonbeam.public.blastapi.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=moonbeam.d.ts.map