export declare const alephZero: {
    blockExplorers: {
        readonly default: {
            readonly name: "Aleph Zero EVM Explorer";
            readonly url: "https://evm-explorer.alephzero.org";
            readonly apiUrl: "https://evm-explorer.alephzero.org/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 4603377;
        };
    };
    id: 41455;
    name: "Aleph Zero";
    nativeCurrency: {
        readonly name: "Aleph Zero";
        readonly symbol: "AZERO";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.alephzero.raas.gelato.cloud"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=alephZero.d.ts.map