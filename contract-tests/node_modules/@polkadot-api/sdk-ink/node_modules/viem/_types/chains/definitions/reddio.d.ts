export declare const reddio: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://reddio.cloud.blockscout.com";
            readonly apiUrl: "https://reddio.cloud.blockscout.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 848849;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 50342;
    name: "Reddio";
    nativeCurrency: {
        readonly name: "Reddio";
        readonly symbol: "RED";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.reddio.com/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=reddio.d.ts.map