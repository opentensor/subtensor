export declare const lycan: {
    blockExplorers: {
        readonly default: {
            readonly name: "Lycan Explorer";
            readonly url: "https://explorer.lycanchain.com";
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
    id: 721;
    name: "Lycan";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Lycan";
        readonly symbol: "LYC";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.lycanchain.com", "https://us-east.lycanchain.com", "https://us-west.lycanchain.com", "https://eu-north.lycanchain.com", "https://eu-west.lycanchain.com", "https://asia-southeast.lycanchain.com"];
            readonly webSocket: readonly ["wss://rpc.lycanchain.com", "wss://us-east.lycanchain.com", "wss://us-west.lycanchain.com", "wss://eu-north.lycanchain.com", "wss://eu-west.lycanchain.com", "wss://asia-southeast.lycanchain.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=lycan.d.ts.map