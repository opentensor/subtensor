export declare const nearTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "NEAR Explorer";
            readonly url: "https://eth-explorer-testnet.near.org";
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
    id: 398;
    name: "NEAR Protocol Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "NEAR";
        readonly symbol: "NEAR";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://eth-rpc.testnet.near.org"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=nearTestnet.d.ts.map