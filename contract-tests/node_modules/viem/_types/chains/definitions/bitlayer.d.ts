export declare const bitlayer: {
    blockExplorers: {
        readonly default: {
            readonly name: "bitlayer mainnet scan";
            readonly url: "https://www.btrscan.com";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0x5B256fE9e993902eCe49D138a5b1162cBb529474";
            readonly blockCreated: 2421963;
        };
    };
    id: 200901;
    name: "Bitlayer Mainnet";
    nativeCurrency: {
        readonly name: "BTC";
        readonly symbol: "BTC";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.bitlayer.org"];
            readonly webSocket: readonly ["wss://ws.bitlayer.org"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=bitlayer.d.ts.map