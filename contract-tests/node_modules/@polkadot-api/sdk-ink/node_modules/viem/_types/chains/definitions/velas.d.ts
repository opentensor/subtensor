export declare const velas: {
    blockExplorers: {
        readonly default: {
            readonly name: "Velas Explorer";
            readonly url: "https://evmexplorer.velas.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 55883577;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 106;
    name: "Velas EVM Mainnet";
    nativeCurrency: {
        readonly name: "VLX";
        readonly symbol: "VLX";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evmexplorer.velas.com/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=velas.d.ts.map