export declare const hashkeyTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "HashKey Chain Testnet explorer";
            readonly url: "https://testnet-explorer.hsk.xyz";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts?: {
        [x: string]: import("../../index.js").ChainContract | {
            [sourceId: number]: import("../../index.js").ChainContract | undefined;
        } | undefined;
        ensRegistry?: import("../../index.js").ChainContract | undefined;
        ensUniversalResolver?: import("../../index.js").ChainContract | undefined;
        multicall3?: import("../../index.js").ChainContract | undefined;
        erc6492Verifier?: import("../../index.js").ChainContract | undefined;
    } | undefined;
    ensTlds?: readonly string[] | undefined;
    id: 133;
    name: "HashKey Chain Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "HashKey EcoPoints";
        readonly symbol: "HSK";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet.hsk.xyz"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=hashkeyChainTestnet.d.ts.map