export declare const cronoszkEVM: {
    blockExplorers: {
        readonly default: {
            readonly name: "Cronos zkEVM (Mainnet) Chain Explorer";
            readonly url: "https://explorer.zkevm.cronos.org";
        };
    };
    contracts: {
        readonly multicall3: {
            readonly address: "0x06f4487d7c4a5983d2660db965cc6d2565e4cfaa";
            readonly blockCreated: 72;
        };
    };
    id: 388;
    name: "Cronos zkEVM Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Cronos zkEVM CRO";
        readonly symbol: "zkCRO";
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://mainnet.zkevm.cronos.org"];
        };
    };
    sourceId?: number | undefined;
    testnet?: boolean | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=cronoszkEVM.d.ts.map