export declare const dailyNetworkTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Daily Testnet Explorer";
            readonly url: "https://explorer.testnet.dailycrypto.net";
        };
    };
    contracts?: import("../index.js").Prettify<{
        [key: string]: import("../../index.js").ChainContract | {
            [sourceId: number]: import("../../index.js").ChainContract | undefined;
        } | undefined;
    } & {
        ensRegistry?: import("../../index.js").ChainContract | undefined;
        ensUniversalResolver?: import("../../index.js").ChainContract | undefined;
        multicall3?: import("../../index.js").ChainContract | undefined;
        universalSignatureVerifier?: import("../../index.js").ChainContract | undefined;
    }> | undefined;
    id: 825;
    name: "Daily Network Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Daily";
        readonly symbol: "DLY";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.testnet.dailycrypto.net"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=dailyNetworkTestnet.d.ts.map