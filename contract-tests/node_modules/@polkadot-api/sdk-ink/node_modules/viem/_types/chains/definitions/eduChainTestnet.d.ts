export declare const eduChainTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "EDU Chain Testnet Explorer";
            readonly url: "https://opencampus-codex.blockscout.com";
            readonly apiUrl: "https://opencampus-codex.blockscout.com/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 15514133;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 656476;
    name: "EDU Chain Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "EDU";
        readonly symbol: "EDU";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.open-campus-codex.gelato.digital/"];
            readonly webSocket: readonly ["wss://ws.open-campus-codex.gelato.digital"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=eduChainTestnet.d.ts.map