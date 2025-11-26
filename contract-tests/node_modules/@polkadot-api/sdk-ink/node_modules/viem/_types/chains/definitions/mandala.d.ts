export declare const mandala: {
    blockExplorers: {
        readonly default: {
            readonly name: "Mandala Blockscout";
            readonly url: "https://blockscout.mandala.aca-staging.network";
            readonly apiUrl: "https://blockscout.mandala.aca-staging.network/api";
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
    id: 595;
    name: "Mandala TC9";
    nativeCurrency: {
        readonly name: "Mandala";
        readonly symbol: "mACA";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://eth-rpc-tc9.aca-staging.network"];
            readonly webSocket: readonly ["wss://eth-rpc-tc9.aca-staging.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "mandala";
};
//# sourceMappingURL=mandala.d.ts.map