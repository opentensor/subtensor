export declare const somniaTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Somnia Testnet Explorer";
            readonly url: "https://shannon-explorer.somnia.network/";
            readonly apiUrl: "https://shannon-explorer.somnia.network/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x841b8199E6d3Db3C6f264f6C2bd8848b3cA64223";
            readonly blockCreated: 71314235;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 50312;
    name: "Somnia Testnet";
    nativeCurrency: {
        readonly name: "STT";
        readonly symbol: "STT";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://dream-rpc.somnia.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=somniaTestnet.d.ts.map