export declare const plumeSepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://testnet-explorer.plume.org";
            readonly apiUrl: "https://testnet-explorer.plume.org/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 199712;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 98867;
    name: "Plume Testnet";
    nativeCurrency: {
        readonly name: "Plume";
        readonly symbol: "PLUME";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet-rpc.plume.org"];
            readonly webSocket: readonly ["wss://testnet-rpc.plume.org"];
        };
    };
    sourceId: 11155111;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=plumeSepolia.d.ts.map