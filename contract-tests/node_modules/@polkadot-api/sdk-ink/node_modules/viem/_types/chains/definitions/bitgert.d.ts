export declare const bitgert: {
    blockExplorers: {
        readonly default: {
            readonly name: "Bitgert Scan";
            readonly url: "https://brisescan.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 2118034;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 32520;
    name: "Bitgert Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Brise";
        readonly symbol: "Brise";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-bitgert.icecreamswap.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=bitgert.d.ts.map