export declare const plumeTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://testnet-explorer.plumenetwork.xyz";
            readonly apiUrl: "https://testnet-explorer.plumenetwork.xyz/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 6022332;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 161221135;
    name: "Plume Testnet (Legacy)";
    nativeCurrency: {
        readonly name: "Plume Sepolia Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet-rpc.plumenetwork.xyz/http"];
            readonly webSocket: readonly ["wss://testnet-rpc.plumenetwork.xyz/ws"];
        };
    };
    sourceId: 11155111;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=plumeTestnet.d.ts.map