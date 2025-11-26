export declare const iotexTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "IoTeXScan";
            readonly url: "https://testnet.iotexscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xb5cecD6894c6f473Ec726A176f1512399A2e355d";
            readonly blockCreated: 24347592;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 4690;
    name: "IoTeX Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "IoTeX";
        readonly symbol: "IOTX";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://babel-api.testnet.iotex.io"];
            readonly webSocket: readonly ["wss://babel-api.testnet.iotex.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=iotexTestnet.d.ts.map