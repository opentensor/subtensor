export declare const coreTestnet2: {
    blockExplorers: {
        readonly default: {
            readonly name: "Core Testnet2";
            readonly url: "https://scan.test2.btcs.network";
            readonly apiUrl: "https://api.test2.btcs.network/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x3CB285ff3Cd5C7C7e570b1E7DE3De17A0f985e56";
            readonly blockCreated: 3838600;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1114;
    name: "Core Testnet2";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "tCore2";
        readonly symbol: "TCORE2";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.test2.btcs.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=coreTestnet2.d.ts.map