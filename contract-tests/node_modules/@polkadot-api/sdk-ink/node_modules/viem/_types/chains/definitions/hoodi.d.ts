export declare const hoodi: {
    blockExplorers: {
        readonly default: {
            readonly name: "Etherscan";
            readonly url: "https://hoodi.etherscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 2589;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 560048;
    name: "Hoodi";
    nativeCurrency: {
        readonly name: "Hoodi Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.hoodi.ethpandaops.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=hoodi.d.ts.map