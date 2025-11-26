export declare const vana: {
    blockExplorers: {
        readonly default: {
            readonly name: "Vana Block Explorer";
            readonly url: "https://vanascan.io";
            readonly apiUrl: "https://vanascan.io/api";
        };
    };
    blockTime: 6000;
    contracts: {
        readonly multicall3: {
            readonly address: "0xD8d2dFca27E8797fd779F8547166A2d3B29d360E";
            readonly blockCreated: 716763;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1480;
    name: "Vana";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Vana";
        readonly symbol: "VANA";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.vana.org/"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=vana.d.ts.map