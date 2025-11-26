export declare const flareTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Coston2 Explorer";
            readonly url: "https://coston2-explorer.flare.network";
            readonly apiUrl: "https://coston2-explorer.flare.network/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts?: {
        [x: string]: import("../../index.js").ChainContract | {
            [sourceId: number]: import("../../index.js").ChainContract | undefined;
        } | undefined;
        ensRegistry?: import("../../index.js").ChainContract | undefined;
        ensUniversalResolver?: import("../../index.js").ChainContract | undefined;
        multicall3?: import("../../index.js").ChainContract | undefined;
        erc6492Verifier?: import("../../index.js").ChainContract | undefined;
    } | undefined;
    ensTlds?: readonly string[] | undefined;
    id: 114;
    name: "Flare Testnet Coston2";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Coston2 Flare";
        readonly symbol: "C2FLR";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://coston2-api.flare.network/ext/C/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=flareTestnet.d.ts.map