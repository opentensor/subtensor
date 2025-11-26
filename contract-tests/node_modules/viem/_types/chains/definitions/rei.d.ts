export declare const rei: {
    blockExplorers: {
        readonly default: {
            readonly name: "REI Scan";
            readonly url: "https://scan.rei.network";
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
    id: 47805;
    name: "REI Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "REI";
        readonly symbol: "REI";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.rei.network"];
            readonly webSocket: readonly ["wss://rpc.rei.network"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=rei.d.ts.map