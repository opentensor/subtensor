export declare const thunderCore: {
    blockExplorers: {
        readonly default: {
            readonly name: "ThunderCore Explorer";
            readonly url: "https://explorer-mainnet.thundercore.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 0;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 108;
    name: "ThunderCore Mainnet";
    nativeCurrency: {
        readonly name: "TT";
        readonly symbol: "TT";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet-rpc.thundercore.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=thunderCore.d.ts.map