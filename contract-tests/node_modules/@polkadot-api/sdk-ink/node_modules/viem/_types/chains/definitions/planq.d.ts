export declare const planq: {
    blockExplorers: {
        readonly default: {
            readonly name: "Planq Explorer";
            readonly url: "https://evm.planq.network";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 8470015;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 7070;
    name: "Planq Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "PLQ";
        readonly symbol: "PLQ";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://planq-rpc.nodies.app", "https://evm-rpc.planq.network", "https://jsonrpc.planq.nodestake.top"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=planq.d.ts.map