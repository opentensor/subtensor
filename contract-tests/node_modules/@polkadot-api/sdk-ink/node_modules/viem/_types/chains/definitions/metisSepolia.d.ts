export declare const metisSepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "Metis Sepolia Explorer";
            readonly url: "https://sepolia-explorer.metisdevops.link";
            readonly apiUrl: "https://sepolia-explorer.metisdevops.link/api-docs";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 224185;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 59902;
    name: "Metis Sepolia";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Test Metis";
        readonly symbol: "tMETIS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://sepolia.metisdevops.link", "https://metis-sepolia-rpc.publicnode.com", "https://metis-sepolia.gateway.tenderly.co"];
            readonly webSocket: readonly ["wss://metis-sepolia-rpc.publicnode.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=metisSepolia.d.ts.map