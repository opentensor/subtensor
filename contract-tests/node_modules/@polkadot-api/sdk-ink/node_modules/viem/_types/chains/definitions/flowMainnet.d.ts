export declare const flowMainnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Mainnet Explorer";
            readonly url: "https://evm.flowscan.io";
        };
    };
    blockTime: 800;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 6205;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 747;
    name: "Flow EVM Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Flow";
        readonly symbol: "FLOW";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.evm.nodes.onflow.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=flowMainnet.d.ts.map