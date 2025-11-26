export declare const cronoszkEVMTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Cronos zkEVM Testnet Explorer";
            readonly url: "https://explorer.zkevm.cronos.org/testnet";
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
    id: 282;
    name: "Cronos zkEVM Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Cronos zkEVM Test Coin";
        readonly symbol: "zkTCRO";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://testnet.zkevm.cronos.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=cronoszkEVMTestnet.d.ts.map