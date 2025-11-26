export declare const eos: {
    blockExplorers: {
        readonly default: {
            readonly name: "EOS EVM Explorer";
            readonly url: "https://explorer.evm.eosnetwork.com";
            readonly apiUrl: "https://explorer.evm.eosnetwork.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 7943933;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 17777;
    name: "EOS EVM";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "EOS";
        readonly symbol: "EOS";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://api.evm.eosnetwork.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=eos.d.ts.map