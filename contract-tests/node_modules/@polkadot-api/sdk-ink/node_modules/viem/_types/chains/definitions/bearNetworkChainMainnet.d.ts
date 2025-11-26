export declare const bearNetworkChainMainnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "BrnkScan";
            readonly url: "https://brnkscan.bearnetwork.net";
            readonly apiUrl: "https://brnkscan.bearnetwork.net/api";
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
    id: 641230;
    name: "Bear Network Chain Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BearNetworkChain";
        readonly symbol: "BRNKC";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://brnkc-mainnet.bearnetwork.net"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=bearNetworkChainMainnet.d.ts.map