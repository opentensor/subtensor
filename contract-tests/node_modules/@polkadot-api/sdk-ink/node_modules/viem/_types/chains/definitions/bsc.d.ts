export declare const bsc: {
    blockExplorers: {
        readonly default: {
            readonly name: "BscScan";
            readonly url: "https://bscscan.com";
            readonly apiUrl: "https://api.bscscan.com/api";
        };
    };
    blockTime: 750;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 15921452;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 56;
    name: "BNB Smart Chain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BNB";
        readonly symbol: "BNB";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://56.rpc.thirdweb.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=bsc.d.ts.map