export declare const berachain: {
    blockExplorers: {
        readonly default: {
            readonly name: "Berascan";
            readonly url: "https://berascan.com";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 0;
        };
        readonly ensRegistry: {
            readonly address: "0x5b22280886a2f5e09a49bea7e320eab0e5320e28";
            readonly blockCreated: 877007;
        };
        readonly ensUniversalResolver: {
            readonly address: "0xddfb18888a9466688235887dec2a10c4f5effee9";
            readonly blockCreated: 877008;
        };
    };
    id: 80094;
    name: "Berachain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BERA Token";
        readonly symbol: "BERA";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.berachain.com"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=berachain.d.ts.map