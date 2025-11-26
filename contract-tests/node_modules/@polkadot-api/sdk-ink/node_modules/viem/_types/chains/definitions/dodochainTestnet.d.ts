export declare const dodochainTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "DODOchain Testnet (Sepolia) Explorer";
            readonly url: "https://testnet-scan.dodochain.com";
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
    id: 53457;
    name: "DODOchain Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "DODO";
        readonly symbol: "DODO";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://dodochain-testnet.alt.technology"];
            readonly webSocket: readonly ["wss://dodochain-testnet.alt.technology/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=dodochainTestnet.d.ts.map