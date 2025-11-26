export declare const peaq: {
    blockExplorers: {
        readonly default: {
            readonly name: "Subscan";
            readonly url: "https://peaq.subscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 3566354;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 3338;
    name: "Peaq";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "peaq";
        readonly symbol: "PEAQ";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://quicknode1.peaq.xyz", "https://quicknode2.peaq.xyz", "https://quicknode3.peaq.xyz"];
            readonly webSocket: readonly ["wss://quicknode1.peaq.xyz", "wss://quicknode2.peaq.xyz", "wss://quicknode3.peaq.xyz"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=peaq.d.ts.map