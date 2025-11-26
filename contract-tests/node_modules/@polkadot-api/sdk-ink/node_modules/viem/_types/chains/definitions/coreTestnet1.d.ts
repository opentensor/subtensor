export declare const coreTestnet1: {
    blockExplorers: {
        readonly default: {
            readonly name: "Core Testnet";
            readonly url: "https://scan.test.btcs.network";
            readonly apiUrl: "https://api.test.btcs.network/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xCcddF20A1932537123C2E48Bd8e00b108B8f7569";
            readonly blockCreated: 29350509;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1115;
    name: "Core Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "tCore";
        readonly symbol: "TCORE";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.test.btcs.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=coreTestnet1.d.ts.map