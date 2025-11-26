export declare const corn: {
    blockExplorers: {
        readonly default: {
            readonly name: "Corn Explorer";
            readonly url: "https://cornscan.io";
            readonly apiUrl: "https://api.routescan.io/v2/network/mainnet/evm/21000000/etherscan/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 3228;
        };
    };
    id: 21000000;
    name: "Corn";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Bitcorn";
        readonly symbol: "BTCN";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.ankr.com/corn_maizenet"];
        };
    };
    sourceId: 1;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=corn.d.ts.map