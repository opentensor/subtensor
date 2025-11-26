export declare const alephZeroTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Aleph Zero EVM Testnet explorer";
            readonly url: "https://evm-explorer-testnet.alephzero.org";
            readonly apiUrl: "https://evm-explorer-testnet.alephzero.org/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 2861745;
        };
    };
    id: 2039;
    name: "Aleph Zero Testnet";
    nativeCurrency: {
        readonly name: "TZERO";
        readonly symbol: "TZERO";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.alephzero-testnet.gelato.digital"];
            readonly webSocket: readonly ["wss://ws.alephzero-testnet.gelato.digital"];
        };
    };
    sourceId?: number | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=alephZeroTestnet.d.ts.map