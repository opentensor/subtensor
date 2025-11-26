export declare const syscoinTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "SyscoinTestnetExplorer";
            readonly url: "https://tanenbaum.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 271288;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 5700;
    name: "Syscoin Tanenbaum Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Syscoin";
        readonly symbol: "SYS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.tanenbaum.io"];
            readonly webSocket: readonly ["wss://rpc.tanenbaum.io/wss"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=syscoinTestnet.d.ts.map