export declare const storyAeneid: {
    blockExplorers: {
        readonly default: {
            readonly name: "Story Aeneid Explorer";
            readonly url: "https://aeneid.storyscan.io";
            readonly apiUrl: "https://aeneid.storyscan.io/api/v2";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 1792;
        };
        readonly ensRegistry: {
            readonly address: "0x5dC881dDA4e4a8d312be3544AD13118D1a04Cb17";
            readonly blockCreated: 1322033;
        };
        readonly ensUniversalResolver: {
            readonly address: "0x6D3B3F99177FB2A5de7F9E928a9BD807bF7b5BAD";
            readonly blockCreated: 1322097;
        };
    };
    ensTlds: readonly [".ip"];
    id: 1315;
    name: "Story Aeneid";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "IP";
        readonly symbol: "IP";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://aeneid.storyrpc.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "story-aeneid";
};
//# sourceMappingURL=storyAeneid.d.ts.map