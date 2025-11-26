export declare const monadTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Monad Testnet explorer";
            readonly url: "https://testnet.monadexplorer.com";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 251449;
        };
    };
    id: 10143;
    name: "Monad Testnet";
    nativeCurrency: {
        readonly name: "Testnet MON Token";
        readonly symbol: "MON";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet-rpc.monad.xyz"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=monadTestnet.d.ts.map