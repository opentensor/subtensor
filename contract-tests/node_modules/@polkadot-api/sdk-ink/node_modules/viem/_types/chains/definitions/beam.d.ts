export declare const beam: {
    blockExplorers: {
        readonly default: {
            readonly name: "Beam Explorer";
            readonly url: "https://subnets.avax.network/beam";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x4956f15efdc3dc16645e90cc356eafa65ffc65ec";
            readonly blockCreated: 1;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 4337;
    name: "Beam";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Beam";
        readonly symbol: "BEAM";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://build.onbeam.com/rpc"];
            readonly webSocket: readonly ["wss://build.onbeam.com/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "beam";
};
//# sourceMappingURL=beam.d.ts.map