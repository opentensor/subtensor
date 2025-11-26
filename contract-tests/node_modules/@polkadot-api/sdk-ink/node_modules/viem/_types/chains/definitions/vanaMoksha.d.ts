export declare const vanaMoksha: {
    blockExplorers: {
        readonly default: {
            readonly name: "Vana Moksha Testnet";
            readonly url: "https://moksha.vanascan.io";
            readonly apiUrl: "https://moksha.vanascan.io/api";
        };
    };
    blockTime: 6000;
    contracts: {
        readonly multicall3: {
            readonly address: "0xD8d2dFca27E8797fd779F8547166A2d3B29d360E";
            readonly blockCreated: 732283;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 14800;
    name: "Vana Moksha Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Vana";
        readonly symbol: "VANA";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.moksha.vana.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=vanaMoksha.d.ts.map