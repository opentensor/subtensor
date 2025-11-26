export declare const astarZkyoto: {
    blockExplorers: {
        readonly default: {
            readonly name: "zKyoto Explorer";
            readonly url: "https://zkyoto.explorer.startale.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 196153;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 6038361;
    name: "Astar zkEVM Testnet zKyoto";
    nativeCurrency: {
        readonly name: "Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.startale.com/zkyoto"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "zKyoto";
};
//# sourceMappingURL=astarZkyoto.d.ts.map