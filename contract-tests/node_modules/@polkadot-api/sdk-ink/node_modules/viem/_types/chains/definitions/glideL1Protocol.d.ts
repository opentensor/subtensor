export declare const glideL1Protocol: {
    blockExplorers: {
        readonly default: {
            readonly name: "Glide Protocol Explore";
            readonly url: "https://blockchain-explorer.glideprotocol.xyz";
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
    id: 251;
    name: "Glide L1 Protocol XP";
    nativeCurrency: {
        readonly name: "GLXP";
        readonly symbol: "GLXP";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-api.glideprotocol.xyz/l1-rpc"];
            readonly webSocket: readonly ["wss://rpc-api.glideprotocol.xyz/l1-rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=glideL1Protocol.d.ts.map