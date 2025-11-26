export declare const xrplevmTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "blockscout";
            readonly url: "https://explorer.testnet.xrplevm.org";
            readonly apiUrl: "https://explorer.testnet.xrplevm.org/api/v2";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x82Cc144D7d0AD4B1c27cb41420e82b82Ad6e9B31";
            readonly blockCreated: 492302;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1449000;
    name: "XRPL EVM Testnet";
    nativeCurrency: {
        readonly name: "XRP";
        readonly symbol: "XRP";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.testnet.xrplevm.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=xrplevmTestnet.d.ts.map