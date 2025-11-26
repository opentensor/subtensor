export declare const geist: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://geist-mainnet.explorer.alchemy.com";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 660735;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 63157;
    name: "Geist Mainnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Aavegotchi GHST Token";
        readonly symbol: "GHST";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://geist-mainnet.g.alchemy.com/public"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=geist.d.ts.map