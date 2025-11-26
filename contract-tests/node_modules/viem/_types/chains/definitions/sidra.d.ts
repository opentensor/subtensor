export declare const sidraChain: {
    blockExplorers: {
        readonly default: {
            readonly name: "Sidra Chain Explorer";
            readonly url: "https://ledger.sidrachain.com";
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
    id: 97453;
    name: "Sidra Chain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Sidra Digital Asset";
        readonly symbol: "SDA";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://node.sidrachain.com"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=sidra.d.ts.map