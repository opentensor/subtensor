export declare const edgewareTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Edgscan by Bharathcoorg";
            readonly url: "https://testnet.edgscan.live";
            readonly apiUrl: "https://testnet.edgscan.live/api";
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
    id: 2022;
    name: "Beresheet BereEVM Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Testnet EDG";
        readonly symbol: "tEDG";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://beresheet-evm.jelliedowl.net"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=edgewareTestnet.d.ts.map