export declare const sapphireTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Oasis Explorer";
            readonly url: "https://explorer.oasis.io/testnet/sapphire";
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
    id: 23295;
    name: "Oasis Sapphire Testnet";
    nativeCurrency: {
        readonly name: "Sapphire Test Rose";
        readonly symbol: "TEST";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet.sapphire.oasis.dev"];
            readonly webSocket: readonly ["wss://testnet.sapphire.oasis.dev/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "sapphire-testnet";
};
//# sourceMappingURL=sapphireTestnet.d.ts.map