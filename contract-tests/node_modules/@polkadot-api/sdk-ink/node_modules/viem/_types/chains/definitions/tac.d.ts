export declare const tac: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://tac.blockscout.com";
            readonly apiUrl: "https://tac.blockscout.com/api";
        };
        readonly native: {
            readonly name: "TAC Explorer";
            readonly url: "https://explorer.tac.build";
            readonly apiUrl: "https://explorer.tac.build/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 0;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 239;
    name: "TAC";
    nativeCurrency: {
        readonly name: "TAC";
        readonly symbol: "TAC";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.ankr.com/tac"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=tac.d.ts.map