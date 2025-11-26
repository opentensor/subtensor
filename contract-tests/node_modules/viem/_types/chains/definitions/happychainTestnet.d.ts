export declare const happychainTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Happy Chain Testnet Explorer";
            readonly url: "https://happy-testnet-sepolia.explorer.caldera.xyz/";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 1;
        };
    };
    id: 216;
    name: "Happychain Testnet";
    nativeCurrency: {
        readonly symbol: "HAPPY";
        readonly name: "HAPPY";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://happy-testnet-sepolia.rpc.caldera.xyz/http"];
            readonly webSocket: readonly ["wss://happy-testnet-sepolia.rpc.caldera.xyz/ws"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=happychainTestnet.d.ts.map