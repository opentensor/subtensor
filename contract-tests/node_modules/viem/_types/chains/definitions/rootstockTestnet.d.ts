export declare const rootstockTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "RSK Explorer";
            readonly url: "https://explorer.testnet.rootstock.io";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 2771150;
        };
    };
    id: 31;
    name: "Rootstock Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Rootstock Bitcoin";
        readonly symbol: "tRBTC";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://public-node.testnet.rsk.co"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "rootstock";
};
//# sourceMappingURL=rootstockTestnet.d.ts.map