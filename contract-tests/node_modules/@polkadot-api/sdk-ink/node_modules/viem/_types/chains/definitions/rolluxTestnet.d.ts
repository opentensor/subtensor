export declare const rolluxTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "RolluxTestnetExplorer";
            readonly url: "https://rollux.tanenbaum.io";
            readonly apiUrl: "https://rollux.tanenbaum.io/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 1813675;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 57000;
    name: "Rollux Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Syscoin";
        readonly symbol: "SYS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-tanenbaum.rollux.com/"];
            readonly webSocket: readonly ["wss://rpc-tanenbaum.rollux.com/wss"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=rolluxTestnet.d.ts.map