export declare const plume: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://explorer.plumenetwork.xyz";
            readonly apiUrl: "https://explorer.plumenetwork.xyz/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 48577;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 98865;
    name: "Plume (Legacy)";
    nativeCurrency: {
        readonly name: "Plume Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.plumenetwork.xyz"];
            readonly webSocket: readonly ["wss://rpc.plumenetwork.xyz"];
        };
    };
    sourceId: 1;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=plume.d.ts.map