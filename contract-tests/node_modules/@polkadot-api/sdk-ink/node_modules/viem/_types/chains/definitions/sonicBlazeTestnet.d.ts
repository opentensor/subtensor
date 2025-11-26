export declare const sonicBlazeTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Sonic Blaze Testnet Explorer";
            readonly url: "https://testnet.sonicscan.org";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 1100;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 57054;
    name: "Sonic Blaze Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Sonic";
        readonly symbol: "S";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.blaze.soniclabs.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=sonicBlazeTestnet.d.ts.map