export declare const story: {
    blockExplorers: {
        readonly default: {
            readonly name: "Story explorer";
            readonly url: "https://storyscan.io";
            readonly apiUrl: "https://storyscan.io/api/v2";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 340998;
        };
        readonly ensRegistry: {
            readonly address: "0x5dc881dda4e4a8d312be3544ad13118d1a04cb17";
            readonly blockCreated: 648924;
        };
        readonly ensUniversalResolver: {
            readonly address: "0xddfb18888a9466688235887dec2a10c4f5effee9";
            readonly blockCreated: 649114;
        };
    };
    ensTlds: readonly [".ip"];
    id: 1514;
    name: "Story";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "IP Token";
        readonly symbol: "IP";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.storyrpc.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=story.d.ts.map