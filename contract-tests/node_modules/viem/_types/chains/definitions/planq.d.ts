export declare const planq: {
    blockExplorers: {
        readonly default: {
            readonly name: "Planq Explorer";
            readonly url: "https://evm.planq.network";
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
    id: 7070;
    name: "Planq Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "PLQ";
        readonly symbol: "PLQ";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evm-rpc.planq.network"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=planq.d.ts.map