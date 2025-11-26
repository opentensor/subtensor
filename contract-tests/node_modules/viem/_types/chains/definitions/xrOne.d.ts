export declare const xrOne: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://xr1.calderaexplorer.xyz";
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
    id: 273;
    name: "XR One";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "XR1";
        readonly symbol: "XR1";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://xr1.calderachain.xyz/http"];
            readonly webSocket: readonly ["wss://xr1.calderachain.xyz/ws"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=xrOne.d.ts.map