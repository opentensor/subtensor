export declare const metisSepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "Metis Sepolia Explorer";
            readonly url: "https://sepolia-explorer.metisdevops.link";
            readonly apiUrl: "https://sepolia-explorer.metisdevops.link/api-docs";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 224185;
        };
    };
    id: 59902;
    name: "Metis Sepolia";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Test Metis";
        readonly symbol: "tMETIS";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["wss://metis-sepolia-rpc.publicnode.com", "https://sepolia.metisdevops.link", "https://metis-sepolia-rpc.publicnode.com", "https://metis-sepolia.gateway.tenderly.co"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=metisSepolia.d.ts.map