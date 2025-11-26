export declare const huddle01Mainnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Huddle01 Caldera Explorer";
            readonly url: "https://huddle01.calderaexplorer.xyz";
            readonly apiUrl: "https://huddle01.calderaexplorer.xyz/api";
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
    id: 12323;
    name: "Huddle01 dRTC Chain";
    nativeCurrency: {
        readonly name: "Ethereum";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://huddle01.calderachain.xyz/http"];
            readonly webSocket: readonly ["wss://huddle01.calderachain.xyz/ws"];
        };
    };
    sourceId: 42161;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=huddle01Mainnet.d.ts.map