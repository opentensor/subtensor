export declare const etp: {
    blockExplorers: {
        readonly default: {
            readonly name: "ETP Scan";
            readonly url: "https://etpscan.xyz";
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
    id: 20256789;
    name: "ETP Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "ETP Chain Native Token";
        readonly symbol: "ETP";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.etpscan.xyz"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=etp.d.ts.map