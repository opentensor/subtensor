export declare const humanodeTestnet5: {
    blockExplorers?: {
        [key: string]: {
            name: string;
            url: string;
            apiUrl?: string | undefined;
        };
        default: {
            name: string;
            url: string;
            apiUrl?: string | undefined;
        };
    } | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
        };
    };
    id: 14853;
    name: "Humanode Testnet 5";
    nativeCurrency: {
        readonly name: "HMND";
        readonly symbol: "HMND";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://explorer-rpc-http.testnet5.stages.humanode.io"];
            readonly webSocket: readonly ["wss://explorer-rpc-ws.testnet5.stages.humanode.io"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=humanodeTestnet5.d.ts.map