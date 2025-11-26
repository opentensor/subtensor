export declare const exsatTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "exSat Explorer";
            readonly url: "https://scan-testnet.exsat.network";
            readonly apiUrl: "https://scan-testnet.exsat.network/api";
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
    id: 839999;
    name: "exSat Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BTC";
        readonly symbol: "BTC";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evm-tst3.exsat.network"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=exSatTestnet.d.ts.map