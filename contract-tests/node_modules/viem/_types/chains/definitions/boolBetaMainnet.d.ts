export declare const boolBetaMainnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "BoolScan";
            readonly url: "https://beta-mainnet.boolscan.com/";
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
    id: 11100;
    name: "Bool Beta Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BOL";
        readonly symbol: "BOL";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://beta-rpc-node-http.bool.network"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=boolBetaMainnet.d.ts.map