export declare const rollux: {
    blockExplorers: {
        readonly default: {
            readonly name: "RolluxExplorer";
            readonly url: "https://explorer.rollux.com";
            readonly apiUrl: "https://explorer.rollux.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 119222;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 570;
    name: "Rollux Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Syscoin";
        readonly symbol: "SYS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.rollux.com"];
            readonly webSocket: readonly ["wss://rpc.rollux.com/wss"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=rollux.d.ts.map