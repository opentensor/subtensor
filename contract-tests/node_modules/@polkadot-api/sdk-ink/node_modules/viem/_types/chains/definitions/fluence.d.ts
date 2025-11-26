export declare const fluence: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://blockscout.mainnet.fluence.dev";
            readonly apiUrl: "https://blockscout.mainnet.fluence.dev/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 207583;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 9999999;
    name: "Fluence";
    nativeCurrency: {
        readonly name: "FLT";
        readonly symbol: "FLT";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.mainnet.fluence.dev"];
            readonly webSocket: readonly ["wss://ws.mainnet.fluence.dev"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=fluence.d.ts.map