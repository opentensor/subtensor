export declare const lumiaMainnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Lumia Explorer";
            readonly url: "https://explorer.lumia.org/";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 3975939;
        };
    };
    id: 994873017;
    name: "Lumia Mainnet";
    nativeCurrency: {
        readonly name: "Lumia";
        readonly symbol: "LUMIA";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet-rpc.lumia.org"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "LumiaMainnet";
};
//# sourceMappingURL=lumiaMainnet.d.ts.map