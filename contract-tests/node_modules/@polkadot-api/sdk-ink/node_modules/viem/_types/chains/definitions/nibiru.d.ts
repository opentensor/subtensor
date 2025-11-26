export declare const nibiru: {
    blockExplorers: {
        readonly default: {
            readonly name: "NibiScan";
            readonly url: "https://nibiscan.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 19587573;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 6900;
    name: "Nibiru";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "NIBI";
        readonly symbol: "NIBI";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evm-rpc.nibiru.fi"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=nibiru.d.ts.map