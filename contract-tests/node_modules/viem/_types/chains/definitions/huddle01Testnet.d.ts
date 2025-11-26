export declare const huddle01Testnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Huddle01 Caldera Explorer";
            readonly url: "https://huddle-testnet.explorer.caldera.xyz";
            readonly apiUrl: "https://huddle-testnet.explorer.caldera.xyz/api";
        };
    };
    contracts?: import("../index.js").Prettify<{
        [key: string]: import("../../index.js").ChainContract | {
            [sourceId: number]: import("../../index.js").ChainContract | undefined;
        } | undefined;
    } & {
        ensRegistry?: import("../../index.js").ChainContract | undefined;
        ensUniversalResolver?: import("../../index.js").ChainContract | undefined;
        multicall3?: import("../../index.js").ChainContract | undefined;
        universalSignatureVerifier?: import("../../index.js").ChainContract | undefined;
    }> | undefined;
    id: 2524852;
    name: "Huddle01 dRTC Chain Testnet";
    nativeCurrency: {
        readonly name: "Ethereum";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://huddle-testnet.rpc.caldera.xyz/http"];
            readonly webSocket: readonly ["wss://huddle-testnet.rpc.caldera.xyz/ws"];
        };
    };
    sourceId: 421614;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=huddle01Testnet.d.ts.map