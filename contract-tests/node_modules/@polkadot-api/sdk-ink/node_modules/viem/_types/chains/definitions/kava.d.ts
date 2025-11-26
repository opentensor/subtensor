export declare const kava: {
    blockExplorers: {
        readonly default: {
            readonly name: "Kava EVM Explorer";
            readonly url: "https://kavascan.com";
            readonly apiUrl: "https://kavascan.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 3661165;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 2222;
    name: "Kava EVM";
    nativeCurrency: {
        readonly name: "Kava";
        readonly symbol: "KAVA";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evm.kava.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "kava-mainnet";
};
//# sourceMappingURL=kava.d.ts.map