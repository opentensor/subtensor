export declare const tacSPB: {
    blockExplorers: {
        readonly default: {
            readonly name: "TAC";
            readonly url: "https://spb.explorer.tac.build";
            readonly apiUrl: "https://spb.explorer.tac.build/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 471429;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 2391;
    name: "TAC SPB Testnet";
    nativeCurrency: {
        readonly name: "TAC";
        readonly symbol: "TAC";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://spb.rpc.tac.build"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=tacSPB.d.ts.map