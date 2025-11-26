export declare const dailyNetwork: {
    blockExplorers: {
        readonly default: {
            readonly name: "Daily Mainnet Explorer";
            readonly url: "https://explorer.mainnet.dailycrypto.net";
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
    id: 824;
    name: "Daily Network Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Daily";
        readonly symbol: "DLY";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.mainnet.dailycrypto.net"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=dailyNetwork.d.ts.map