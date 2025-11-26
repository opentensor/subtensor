export declare const initVerse: {
    blockExplorers: {
        readonly default: {
            readonly name: "InitVerseScan";
            readonly url: "https://www.iniscan.com";
            readonly apiUrl: "https://explorer-api.inichain.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x83466BE48A067115FFF91f7b892Ed1726d032e47";
            readonly blockCreated: 2318;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 7233;
    name: "InitVerse Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "InitVerse";
        readonly symbol: "INI";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-mainnet.inichain.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=initVerse.d.ts.map