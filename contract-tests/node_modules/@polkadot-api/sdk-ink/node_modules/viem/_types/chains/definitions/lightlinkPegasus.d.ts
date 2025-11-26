export declare const lightlinkPegasus: {
    blockExplorers: {
        readonly default: {
            readonly name: "LightLink Pegasus Explorer";
            readonly url: "https://pegasus.lightlink.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 127188532;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1891;
    name: "LightLink Pegasus Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Ether";
        readonly symbol: "ETH";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://replicator.pegasus.lightlink.io/rpc/v1"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "lightlink-pegasus";
};
//# sourceMappingURL=lightlinkPegasus.d.ts.map