export declare const lumiaTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Lumia Testnet Explorer";
            readonly url: "https://testnet-explorer.lumia.org/";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 2235063;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1952959480;
    name: "Lumia Testnet";
    nativeCurrency: {
        readonly name: "Lumia";
        readonly symbol: "LUMIA";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet-rpc.lumia.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "LumiaTestnet";
};
//# sourceMappingURL=lumiaTestnet.d.ts.map