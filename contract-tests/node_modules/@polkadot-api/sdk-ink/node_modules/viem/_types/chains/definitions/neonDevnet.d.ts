export declare const neonDevnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Neonscan";
            readonly url: "https://devnet.neonscan.org";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 205206112;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 245022926;
    name: "Neon EVM DevNet";
    nativeCurrency: {
        readonly name: "NEON";
        readonly symbol: "NEON";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://devnet.neonevm.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=neonDevnet.d.ts.map