export declare const cornTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Corn Testnet Explorer";
            readonly url: "https://testnet.cornscan.io";
            readonly apiUrl: "https://api.routescan.io/v2/network/testnet/evm/21000001/etherscan/api";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 4886;
        };
    };
    id: 21000001;
    name: "Corn Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Bitcorn";
        readonly symbol: "BTCN";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.ankr.com/corn_testnet"];
        };
    };
    sourceId: 11155111;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=cornTestnet.d.ts.map