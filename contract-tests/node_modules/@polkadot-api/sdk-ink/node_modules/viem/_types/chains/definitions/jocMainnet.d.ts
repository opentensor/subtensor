export declare const jocMainnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Block Explorer";
            readonly url: "https://explorer.japanopenchain.org";
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
    id: 81;
    name: "Japan Open Chain Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Japan Open Chain Token";
        readonly symbol: "JOC";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc-1.japanopenchain.org:8545", "https://rpc-2.japanopenchain.org:8545", "https://rpc-3.japanopenchain.org"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=jocMainnet.d.ts.map