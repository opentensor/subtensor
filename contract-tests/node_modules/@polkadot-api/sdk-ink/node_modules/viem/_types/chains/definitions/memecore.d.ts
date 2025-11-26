export declare const memecore: {
    blockExplorers: {
        readonly default: {
            readonly name: "MemeCore Explorer";
            readonly url: "https://memecorescan.io";
            readonly apiUrl: "https://api.memecorescan.io/api";
        };
        readonly okx: {
            readonly name: "MemeCore Explorer";
            readonly url: "https://web3.okx.com/explorer/memecore";
        };
        readonly memecore: {
            readonly name: "MemeCore Explorer";
            readonly url: "https://blockscout.memecore.com";
            readonly apiUrl: "https://blockscout.memecore.com/api";
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
    id: 4352;
    name: "MemeCore";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "M";
        readonly symbol: "M";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.memecore.net"];
            readonly webSocket: readonly ["wss://ws.memecore.net"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=memecore.d.ts.map