export declare const scroll: {
    blockExplorers: {
        readonly default: {
            readonly name: "Scrollscan";
            readonly url: "https://scrollscan.com";
            readonly apiUrl: "https://api.scrollscan.com/api";
        };
    };
    blockTime: 3000;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 14;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 534352;
    name: "Scroll";
    nativeCurrency: {
        readonly name: "Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.scroll.io"];
            readonly webSocket: readonly ["wss://wss-rpc.scroll.io/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=scroll.d.ts.map