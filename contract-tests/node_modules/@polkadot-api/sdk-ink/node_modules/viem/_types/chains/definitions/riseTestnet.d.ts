export declare const riseTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://explorer.testnet.riselabs.xyz/";
            readonly apiUrl: "https://explorer.testnet.riselabs.xyz/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 11155931;
    name: "RISE Testnet";
    nativeCurrency: {
        readonly name: "RISE Testnet Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet.riselabs.xyz"];
            readonly webSocket: readonly ["wss://testnet.riselabs.xyz/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=riseTestnet.d.ts.map