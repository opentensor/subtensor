export declare const plumeDevnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://test-explorer.plumenetwork.xyz";
            readonly apiUrl: "https://test-explorer.plumenetwork.xyz/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 481948;
        };
    };
    id: 98864;
    name: "Plume Devnet";
    nativeCurrency: {
        readonly name: "Plume Sepolia Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://test-rpc.plumenetwork.xyz"];
            readonly webSocket: readonly ["wss://test-rpc.plumenetwork.xyz"];
        };
    };
    sourceId: 11155111;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=plumeDevnet.d.ts.map