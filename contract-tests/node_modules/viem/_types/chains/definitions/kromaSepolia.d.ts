export declare const kromaSepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "Kroma Sepolia Explorer";
            readonly url: "https://blockscout.sepolia.kroma.network";
            readonly apiUrl: "https://blockscout.sepolia.kroma.network/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 8900914;
        };
    };
    id: 2358;
    name: "Kroma Sepolia";
    nativeCurrency: {
        readonly name: "Sepolia Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://api.sepolia.kroma.network"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=kromaSepolia.d.ts.map