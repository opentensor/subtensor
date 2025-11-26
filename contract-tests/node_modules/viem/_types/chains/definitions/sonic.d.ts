export declare const sonic: {
    blockExplorers: {
        readonly default: {
            readonly name: "Sonic Explorer";
            readonly url: "https://sonicscan.org/";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 60;
        };
    };
    id: 146;
    name: "Sonic";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Sonic";
        readonly symbol: "S";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.soniclabs.com"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=sonic.d.ts.map