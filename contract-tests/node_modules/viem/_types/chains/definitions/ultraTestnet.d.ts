export declare const ultraTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Ultra EVM Testnet Explorer";
            readonly url: "https://evmexplorer.testnet.ultra.io";
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
    id: 18881;
    name: "Ultra EVM Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Ultra Token";
        readonly symbol: "UOS";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evm.test.ultra.eosusa.io"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=ultraTestnet.d.ts.map