export declare const dogechain: {
    blockExplorers: {
        readonly default: {
            readonly name: "DogeChainExplorer";
            readonly url: "https://explorer.dogechain.dog";
            readonly apiUrl: "https://explorer.dogechain.dog/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x68a8609a60a008EFA633dfdec592c03B030cC508";
            readonly blockCreated: 25384031;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 2000;
    name: "Dogechain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Wrapped Dogecoin";
        readonly symbol: "WDOGE";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.dogechain.dog"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=dogechain.d.ts.map