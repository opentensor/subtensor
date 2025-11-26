export declare const worldLand: {
    blockExplorers: {
        readonly default: {
            readonly name: "WorldLand Scan";
            readonly url: "https://scan.worldland.foundation";
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
    id: 103;
    name: "WorldLand Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "WLC";
        readonly symbol: "WLC";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://seoul.worldland.foundation"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=worldLand.d.ts.map