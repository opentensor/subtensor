export declare const lensTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Lens Block Explorer";
            readonly url: "https://block-explorer.testnet.lens.dev";
            readonly apiUrl: "https://block-explorer-api.staging.lens.dev/api";
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
    id: 37111;
    name: "Lens Testnet";
    nativeCurrency: {
        readonly name: "GRASS";
        readonly symbol: "GRASS";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.testnet.lens.dev"];
            readonly webSocket: readonly ["wss://rpc.testnet.lens.dev/ws"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=lensTestnet.d.ts.map