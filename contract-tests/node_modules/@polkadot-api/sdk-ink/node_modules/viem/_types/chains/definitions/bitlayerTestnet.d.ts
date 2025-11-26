export declare const bitlayerTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "bitlayer testnet scan";
            readonly url: "https://testnet.btrscan.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x5B256fE9e993902eCe49D138a5b1162cBb529474";
            readonly blockCreated: 4135671;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 200810;
    name: "Bitlayer Testnet";
    nativeCurrency: {
        readonly name: "BTC";
        readonly symbol: "BTC";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet-rpc.bitlayer.org"];
            readonly webSocket: readonly ["wss://testnet-ws.bitlayer.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=bitlayerTestnet.d.ts.map