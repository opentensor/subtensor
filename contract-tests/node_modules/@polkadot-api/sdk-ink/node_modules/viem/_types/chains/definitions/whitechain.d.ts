export declare const whitechain: {
    blockExplorers: {
        readonly default: {
            readonly name: "Whitechain Explorer";
            readonly url: "https://explorer.whitechain.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 25212237;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1875;
    name: "Whitechain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "WhiteBIT Coin";
        readonly symbol: "WBT";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.whitechain.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=whitechain.d.ts.map