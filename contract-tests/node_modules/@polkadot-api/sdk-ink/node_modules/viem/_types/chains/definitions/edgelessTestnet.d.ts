export declare const edgelessTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Edgeless Testnet Explorer";
            readonly url: "https://testnet.explorer.edgeless.network";
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
    id: 202;
    name: "Edgeless Testnet";
    nativeCurrency: {
        readonly name: "Edgeless Wrapped ETH";
        readonly symbol: "EwETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://edgeless-testnet.rpc.caldera.xyz/http"];
            readonly webSocket: readonly ["wss://edgeless-testnet.rpc.caldera.xyz/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=edgelessTestnet.d.ts.map