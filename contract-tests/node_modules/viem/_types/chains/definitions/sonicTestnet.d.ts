export declare const sonicTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Sonic Testnet Explorer";
            readonly url: "https://testnet.soniclabs.com/";
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
    id: 64165;
    name: "Sonic Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Sonic";
        readonly symbol: "S";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.testnet.soniclabs.com"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=sonicTestnet.d.ts.map