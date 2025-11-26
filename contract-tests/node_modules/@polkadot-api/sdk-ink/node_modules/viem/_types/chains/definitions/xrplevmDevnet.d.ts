export declare const xrplevmDevnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "XRPLEVM Devnet Explorer";
            readonly url: "https://explorer.xrplevm.org/";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x82Cc144D7d0AD4B1c27cb41420e82b82Ad6e9B31";
            readonly blockCreated: 15237286;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1440002;
    name: "XRPL EVM Devnet";
    nativeCurrency: {
        readonly name: "XRP";
        readonly symbol: "XRP";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.xrplevm.org/"];
        };
        readonly public: {
            readonly http: readonly ["https://rpc.xrplevm.org/"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=xrplevmDevnet.d.ts.map