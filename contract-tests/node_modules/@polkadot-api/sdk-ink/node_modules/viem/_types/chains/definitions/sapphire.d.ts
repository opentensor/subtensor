export declare const sapphire: {
    blockExplorers: {
        readonly default: {
            readonly name: "Oasis Explorer";
            readonly url: "https://explorer.oasis.io/mainnet/sapphire";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 734531;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 23294;
    name: "Oasis Sapphire";
    nativeCurrency: {
        readonly name: "Sapphire Rose";
        readonly symbol: "ROSE";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://sapphire.oasis.io"];
            readonly webSocket: readonly ["wss://sapphire.oasis.io/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "sapphire";
};
//# sourceMappingURL=sapphire.d.ts.map