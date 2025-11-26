export declare const pumpfiTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Pumpfi Testnet Scan";
            readonly url: "https://testnetscan.pumpfi.me";
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
    id: 490092;
    name: "Pumpfi Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "PMPT";
        readonly symbol: "PMPT";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc1testnet.pumpfi.me"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=pumpfiTestnet.d.ts.map