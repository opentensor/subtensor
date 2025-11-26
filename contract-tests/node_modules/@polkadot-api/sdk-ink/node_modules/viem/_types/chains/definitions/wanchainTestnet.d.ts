export declare const wanchainTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "WanScanTest";
            readonly url: "https://wanscan.org";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0x11c89bF4496c39FB80535Ffb4c92715839CC5324";
            readonly blockCreated: 24743448;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 999;
    name: "Wanchain Testnet";
    nativeCurrency: {
        readonly name: "WANCHAIN";
        readonly symbol: "WANt";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://gwan-ssl.wandevs.org:46891"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=wanchainTestnet.d.ts.map