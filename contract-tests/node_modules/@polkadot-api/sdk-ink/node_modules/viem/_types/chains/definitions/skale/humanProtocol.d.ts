export declare const skaleHumanProtocol: {
    blockExplorers: {
        readonly default: {
            readonly name: "SKALE Explorer";
            readonly url: "https://wan-red-ain.explorer.mainnet.skalenodes.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {};
    ensTlds?: readonly string[] | undefined;
    id: 1273227453;
    name: "SKALE | Human Protocol";
    nativeCurrency: {
        readonly name: "sFUEL";
        readonly symbol: "sFUEL";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.skalenodes.com/v1/wan-red-ain"];
            readonly webSocket: readonly ["wss://mainnet.skalenodes.com/v1/ws/wan-red-ain"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../../index.js").ChainSerializers<undefined, import("../../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=humanProtocol.d.ts.map