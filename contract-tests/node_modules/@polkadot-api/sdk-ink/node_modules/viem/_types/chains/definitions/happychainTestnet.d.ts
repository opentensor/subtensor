export declare const happychainTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Happy Chain Testnet Explorer";
            readonly url: "https://explorer.testnet.happy.tech";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 1;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 216;
    name: "Happychain Testnet";
    nativeCurrency: {
        readonly symbol: "HAPPY";
        readonly name: "HAPPY";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.testnet.happy.tech/http"];
            readonly webSocket: readonly ["wss://rpc.testnet.happy.tech/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=happychainTestnet.d.ts.map