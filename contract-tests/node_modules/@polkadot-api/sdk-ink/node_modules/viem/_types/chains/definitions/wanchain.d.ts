export declare const wanchain: {
    blockExplorers: {
        readonly default: {
            readonly name: "WanScan";
            readonly url: "https://wanscan.org";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcDF6A1566e78EB4594c86Fe73Fcdc82429e97fbB";
            readonly blockCreated: 25312390;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 888;
    name: "Wanchain";
    nativeCurrency: {
        readonly name: "WANCHAIN";
        readonly symbol: "WAN";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://gwan-ssl.wandevs.org:56891", "https://gwan2-ssl.wandevs.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=wanchain.d.ts.map