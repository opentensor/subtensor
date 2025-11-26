export declare const areonNetwork: {
    blockExplorers: {
        readonly default: {
            readonly name: "Areonscan";
            readonly url: "https://areonscan.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 353286;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 463;
    name: "Areon Network";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "AREA";
        readonly symbol: "AREA";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet-rpc.areon.network"];
            readonly webSocket: readonly ["wss://mainnet-ws.areon.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=areonNetwork.d.ts.map