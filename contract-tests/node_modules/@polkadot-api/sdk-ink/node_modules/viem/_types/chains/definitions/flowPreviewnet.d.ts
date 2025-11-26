export declare const flowPreviewnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Previewnet Explorer";
            readonly url: "https://previewnet.flowdiver.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 6205;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 646;
    name: "Flow EVM Previewnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Flow";
        readonly symbol: "FLOW";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://previewnet.evm.nodes.onflow.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=flowPreviewnet.d.ts.map