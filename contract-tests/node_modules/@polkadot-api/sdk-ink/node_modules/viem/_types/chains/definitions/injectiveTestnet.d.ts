export declare const injectiveTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Injective Explorer";
            readonly url: "https://testnet.blockscout.injective.network";
            readonly apiUrl: "https://testnet.blockscout.injective.network/api";
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
    id: 1439;
    name: "Injective Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Injective";
        readonly symbol: "INJ";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://k8s.testnet.json-rpc.injective.network"];
            readonly webSocket: readonly ["wss://k8s.testnet.ws.injective.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=injectiveTestnet.d.ts.map