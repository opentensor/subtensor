export declare const mantraEVM: {
    blockExplorers: {
        readonly default: {
            readonly name: "MANTRA Scan";
            readonly url: "https://mantrascan.io/mainnet";
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
    id: 5888;
    name: "MANTRA EVM";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "OM";
        readonly symbol: "OM";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evm.mantrachain.io"];
            readonly webSocket: readonly ["https://evm.mantrachain.io/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=mantraEVM.d.ts.map