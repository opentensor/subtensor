export declare const bitkubTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Bitkub Chain Testnet Explorer";
            readonly url: "https://testnet.bkcscan.com";
            readonly apiUrl: "https://testnet.bkcscan.com/api";
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
    id: 25925;
    name: "Bitkub Testnet";
    nativeCurrency: {
        readonly name: "Bitkub Test";
        readonly symbol: "tKUB";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-testnet.bitkubchain.io"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "Bitkub Testnet";
};
//# sourceMappingURL=bitkubTestnet.d.ts.map