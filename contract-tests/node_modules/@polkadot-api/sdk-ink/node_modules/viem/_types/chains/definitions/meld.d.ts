export declare const meld: {
    blockExplorers: {
        readonly default: {
            readonly name: "MELDscan";
            readonly url: "https://meldscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x769ee5a8e82c15c1b6e358f62ac8eb6e3abe8dc5";
            readonly blockCreated: 360069;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 333000333;
    name: "Meld";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Meld";
        readonly symbol: "MELD";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-1.meld.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=meld.d.ts.map