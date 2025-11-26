export declare const haustTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Haust Network Testnet Explorer";
            readonly url: "https://explorer-testnet.haust.app";
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
    id: 1523903251;
    name: "Haust Network Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "HAUST";
        readonly symbol: "HAUST";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-testnet.haust.app"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=haustTestnet.d.ts.map