export declare const gravity: {
    blockExplorers: {
        readonly default: {
            readonly name: "Gravity Explorer";
            readonly url: "https://explorer.gravity.xyz";
            readonly apiUrl: "https://explorer.gravity.xyz/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xf8ac4BEB2F75d2cFFb588c63251347fdD629B92c";
            readonly blockCreated: 16851;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1625;
    name: "Gravity Alpha Mainnet";
    nativeCurrency: {
        readonly name: "G";
        readonly symbol: "G";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.gravity.xyz"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=gravity.d.ts.map