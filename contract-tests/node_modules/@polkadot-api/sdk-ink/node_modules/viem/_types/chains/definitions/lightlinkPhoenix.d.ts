export declare const lightlinkPhoenix: {
    blockExplorers: {
        readonly default: {
            readonly name: "LightLink Phoenix Explorer";
            readonly url: "https://phoenix.lightlink.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 125499184;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1890;
    name: "LightLink Phoenix Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Ether";
        readonly symbol: "ETH";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://replicator.phoenix.lightlink.io/rpc/v1"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "lightlink-phoenix";
};
//# sourceMappingURL=lightlinkPhoenix.d.ts.map